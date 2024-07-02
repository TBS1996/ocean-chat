use crate::common::SocketMessage;
use crate::server::User;
use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitStream;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
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
    pub fn clear_user(&mut self, id: &str) {
        let Some(connection_id) = self.user_to_connection.remove(id) else {
            return;
        };

        tracing::info!("User connecting twice: {}", id);

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

    pub async fn clear_user(&self, id: &str) {
        self.inner.lock().await.clear_user(id);
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
    async fn run(self) {
        tracing::info!("communication starting between a pair");

        let left_message_sender = self.left.message_sender.clone();
        let right_message_sender = self.right.message_sender.clone();

        left_message_sender
            .send(SocketMessage::PeerConnected.into())
            .await
            .unwrap();
        right_message_sender
            .send(SocketMessage::PeerConnected.into())
            .await
            .unwrap();

        left_message_sender
            .send(SocketMessage::peer_scores(self.right.scores))
            .await
            .unwrap();
        right_message_sender
            .send(SocketMessage::peer_scores(self.left.scores))
            .await
            .unwrap();

        tokio::spawn(Self::process_messages(
            self.left.socket_receiver,
            right_message_sender,
        ));
        tokio::spawn(Self::process_messages(
            self.right.socket_receiver,
            left_message_sender,
        ));
    }

    async fn process_messages(
        mut user_socket: SplitStream<WebSocket>,
        other_user: Sender<Message>,
    ) {
        while let Some(Ok(msg)) = user_socket.next().await {
            match SocketMessage::from(msg) {
                msg @ SocketMessage::User(_) => {
                    other_user.send(msg.into()).await.unwrap();
                }
                SocketMessage::ConnectionClosed => {
                    other_user
                        .send(SocketMessage::PeerConnectionClosed.into())
                        .await
                        .unwrap();
                }
                _ => {
                    tracing::error!("weird error");
                }
            }
        }
    }
}
