use crate::common::SocketMessage;
use crate::server::User;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::oneshot;
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
    id_to_handle: HashMap<ConnectionId, UserExtractor>,
}

impl Inner {
    /// Removes a user.
    ///
    /// Ensures that when a user is removed, also the connection is closed, and
    /// the other user in the connection is also removed.
    pub fn clear_user(&mut self, id: &str) {
        let Some(connection_id) = self.user_to_connection.remove(id) else {
            return;
        };

        tracing::info!("User connecting twice: {}", id);

        // Removes the connection and aborts the thread.
        if let Some(handle) = self.id_to_handle.remove(&connection_id) {
            tokio::spawn(async move {
                handle.get().await;
            });
        }

        // The connection id consists of the two user ids.
        // So to ensure the other user is also removed we simply call remove
        // on both the IDs that make up the tuple.
        let (left_id, right_id) = connection_id;
        self.user_to_connection.remove(&left_id);
        self.user_to_connection.remove(&right_id);
    }

    pub async fn take_pair(&mut self, id: &str) -> Option<(User, User)> {
        let con_id = self.user_to_connection.remove(id)?;
        let extractor = self.id_to_handle.remove(&con_id)?;
        extractor.get().await
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

    pub async fn clear_user(&self, id: &str) {
        self.inner.lock().await.clear_user(id);
    }

    pub async fn take(&self, id: &str) -> Option<(User, User)> {
        self.inner.lock().await.take_pair(id).await
    }

    pub async fn contains(&self, id: &str) -> bool {
        self.inner.lock().await.user_to_connection.contains_key(id)
    }

    /// Connects two users together for chatting.
    pub async fn connect(&self, left: User, right: User) {
        let mut lock = self.inner.lock().await;
        lock.clear_user(&left.id);
        lock.clear_user(&right.id);

        let con_id = (left.id.clone(), right.id.clone());

        lock.user_to_connection
            .insert(left.id.clone(), con_id.clone());
        lock.user_to_connection
            .insert(right.id.clone(), con_id.clone());

        let extractor = Connection::new(left, right).run();
        lock.id_to_handle.insert(con_id, extractor);
        lock.invariant();
    }
}

#[derive(Debug)]
pub struct UserExtractor {
    handle: JoinHandle<(User, User)>,
    sender: oneshot::Sender<()>,
}

impl UserExtractor {
    pub async fn get(self) -> Option<(User, User)> {
        self.sender.send(()).ok()?;
        self.handle.await.ok()
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
    fn run(self) -> UserExtractor {
        let (t, r) = oneshot::channel();
        let handle = tokio::spawn(async move { self.inner(r).await });
        let f = UserExtractor { handle, sender: t };
        f
    }

    async fn inner(mut self, mut stop_signal: oneshot::Receiver<()>) -> (User, User) {
        tracing::info!("communication starting between a pair");
        let msg = "connected to peer!".to_string();
        let _ = self.right.send(SocketMessage::Info(msg.clone())).await;
        let _ = self.left.send(SocketMessage::Info(msg)).await;
        let _ = self
            .right
            .send(SocketMessage::PeerScores(self.left.scores))
            .await;
        let _ = self
            .left
            .send(SocketMessage::PeerScores(self.right.scores))
            .await;

        loop {
            tokio::select! {
                _ = &mut stop_signal => {
                    break;
                },
                Some(msg) = self.left.receive() => {
                    tracing::info!("{}: {:?}", &self.left.id, &msg);
                    match msg {
                        msg @ SocketMessage::User(_) => {
                            if self.right.send(msg).await.is_err(){
                                tracing::error!("error sending message to: {}", &self.right.id);
                                break;
                            };
                        },
                        msg @ SocketMessage::ConnectionClosed => {
                            let _ = self.right.send(msg).await;
                            break;
                        },
                        _ => {
                            tracing::error!("unexpected message");
                        }
                    };
                },
                Some(msg) = self.right.receive() => {
                    tracing::info!("{}: {:?}", &self.right.id, &msg);
                    match msg {
                        msg @ SocketMessage::User(_) => {
                            if self.left.send(msg).await.is_err(){
                                tracing::error!("error sending message to: {}", &self.left.id);
                                break;
                            };
                        }
                        msg @ SocketMessage::ConnectionClosed => {
                            let _ = self.left.send(msg).await;
                            break;
                        },
                        _ => {
                            tracing::error!("unexpected message");
                        }
                    };
                },
                else => {
                    tracing::error!("weird error");
                    break;
                }
            }
        }

        tracing::info!("closing connection");

        (self.left, self.right)
    }
}
