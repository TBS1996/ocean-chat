use crate::common::SocketMessage;
use crate::server::User;
use axum::extract::ws::Message;
use futures_util::SinkExt;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time;

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
    fn clear_user(&mut self, user: &User) {
        let Some(id) = self.user_to_connection.remove(&user.id) else {
            return;
        };

        tracing::info!("User connecting twice: {}", &user.id);

        if let Some(handle) = self.id_to_handle.remove(&id) {
            handle.abort();
        }
        self.user_to_connection.remove(&id.0);
        self.user_to_connection.remove(&id.1);
    }

    fn debug(&self) {
        tracing::info!("current active connections: {}", self.id_to_handle.len());
        tracing::debug!("{:?}", self);
    }
}

impl ConnectionManager {
    pub async fn connected_users_qty(&self) -> usize {
        self.inner.lock().await.id_to_handle.len()
    }

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

    //  Removes all the pending messages in the streams.
    async fn drain_streams(&mut self) {
        let drain_timeout = time::Duration::from_millis(100);

        while let Ok(Some(_)) = time::timeout(drain_timeout, self.left.socket.recv()).await {}
        while let Ok(Some(_)) = time::timeout(drain_timeout, self.right.socket.recv()).await {}
    }

    /// Handles sending messages from one peer to another.
    async fn run(mut self) {
        tracing::info!("communication starting between a pair");
        let msg = "connected to peer!".to_string();

        self.drain_streams().await;
        let (mut left_tx, mut left_rx) = self.left.socket.split();
        let (mut right_tx, mut right_rx) = self.right.socket.split();

        let _ = right_tx.send(SocketMessage::info_msg(msg.clone())).await;
        let _ = left_tx.send(SocketMessage::info_msg(msg)).await;
        let _ = right_tx
            .send(SocketMessage::peer_scores(self.left.scores))
            .await;
        let _ = left_tx
            .send(SocketMessage::peer_scores(self.right.scores))
            .await;

        loop {
            tokio::select! {
                Some(Ok(msg)) = right_rx.next() => {
                    match msg {
                        Message::Close(_) => {
                            let _ = left_tx.send(SocketMessage::close_connection()).await;
                            break;
                        },
                        Message::Binary(bytes) => {
                            match serde_json::from_slice(&bytes) {
                                Ok(SocketMessage::User(msg)) => {
                                    tracing::info!("right->left: {}", &msg);
                                    if left_tx.send(SocketMessage::user_msg(msg)).await.is_err() {
                                        let _ = left_tx.send(SocketMessage::close_connection()).await;
                                        tracing::error!("Failed to send message to left");
                                        break;
                                    }

                                },
                                _ => {},
                            }
                        },
                        _ => {}
                    }
                }
                Some(Ok(msg)) = left_rx.next() => {
                    match msg {
                        Message::Close(_) => {
                            let _ = right_tx.send(SocketMessage::close_connection()).await;
                            break;
                        },
                        Message::Binary(bytes) => {
                            match serde_json::from_slice(&bytes) {
                                Ok(SocketMessage::User(msg)) => {
                                    tracing::info!("left->right: {}", &msg);
                                    if right_tx.send(SocketMessage::user_msg(msg)).await.is_err() {
                                        let _ = left_tx.send(SocketMessage::close_connection()).await;
                                        tracing::error!("Failed to send message to right");
                                        break;
                                    }

                                },
                                _ => {},
                            }
                        },
                        _ => {}
                    }
                }
                else => {
                    let _ = left_tx.send(SocketMessage::close_connection()).await;
                    let _ = right_tx.send(SocketMessage::close_connection()).await;
                    break;
                }
            }
        }

        let _ = left_tx.close().await;
        let _ = right_tx.close().await;

        tracing::info!("closing connection");
    }
}
