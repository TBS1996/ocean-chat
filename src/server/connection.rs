use crate::common::SocketMessage;
use crate::server::User;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

type UserId = String;
type ConnectionId = (UserId, UserId);

/// Ensures the same user is not connected multiple times.
#[derive(Default, Debug, Clone)]
pub struct ConnectionManager {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Default, Debug)]
struct Inner {
    user_to_connection: HashMap<UserId, ConnectionId>,
    id_to_handle: HashMap<ConnectionId, JoinHandle<()>>,
}

impl Inner {
    /// Removes a user.
    ///
    /// Ensures that when a user is removed, also the connection is closed, and
    /// the other user in the connection is also removed.
    fn clear_user(&mut self, user: &User) {
        let Some(connection_id) = self.user_to_connection.remove(&user.id) else {
            return;
        };

        tracing::info!("User connecting twice: {}", &user.id);

        // Removes the connection and aborts the thread.
        if let Some(handle) = self.id_to_handle.remove(&connection_id) {
            handle.abort();
        }

        // The connection id consists of the two user ids.
        // So to ensure the other user is also removed we simply call remove
        // on both the IDs that make up the tuple.
        let (left_id, right_id) = connection_id;
        self.user_to_connection.remove(&left_id);
        self.user_to_connection.remove(&right_id);
    }

    fn debug(&self) {
        tracing::info!("current active connections: {}", self.id_to_handle.len());
        tracing::debug!("{:?}", self);
    }

    // There should be exactly twice as many users as connections.
    fn invariant(&self) {
        if self.user_to_connection.len() != self.id_to_handle.len() * 2 {
            tracing::error!(
                "INVALID STATE: user_to_connection: {}, id_to_handle: {}",
                self.user_to_connection.len(),
                self.id_to_handle.len()
            );
        }
    }
}

impl ConnectionManager {
    /// Returns the quantity of users currently connected.
    pub async fn connected_users_qty(&self) -> usize {
        self.inner.lock().await.id_to_handle.len()
    }

    /// Connects two users together for chatting.
    pub async fn connect(&self, left: User, right: User) {
        let mut lock = self.inner.lock().await;
        lock.clear_user(&left);
        lock.clear_user(&right);

        let con_id = (left.id.clone(), right.id.clone());

        lock.user_to_connection
            .insert(left.id.clone(), con_id.clone());
        lock.user_to_connection
            .insert(right.id.clone(), con_id.clone());

        let handle = tokio::spawn(async move {
            Connection::new(left, right).run().await;
        });
        lock.id_to_handle.insert(con_id, handle);
        lock.invariant();
    }
}

/// Holds the client-server connections between two peers.
struct Connection {
    left: User,
    right: User,
}

impl Connection {
    fn new(left: User, right: User) -> Self {
        Self { left, right }
    }

    /// Handles sending messages from one peer to another.
    async fn run(mut self) {
        tracing::info!("communication starting between a pair");
        let msg = "connected to peer!".to_string();
        let _ = self.right.send(SocketMessage::Info(msg.clone())).await;
        let _ = self.left.send(SocketMessage::Info(msg)).await;

        loop {
            tokio::select! {
                Some(msg) = self.left.receiver.recv() => {
                    if self.right.send(msg).await.is_err(){
                        break;
                    };

                },
                Some(msg) = self.right.receiver.recv() => {
                 if self.left.send(msg).await.is_err() {
                     break;
                 };

                },
                else => {
                    break;
                }
            }
        }

        tracing::info!("closing connection");
    }
}
