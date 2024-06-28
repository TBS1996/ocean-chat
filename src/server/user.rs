use crate::common;

use axum::extract::ws::{Message, WebSocket};
use common::CONFIG;
use common::{Scores, SocketMessage};
use futures_util::StreamExt;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;

use futures_util::SinkExt;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration, Instant};

/// Takes care of sending to and receiving from a websocket.
fn handle_socket(
    socket: WebSocket,
    id: String,
    mut close_signal: oneshot::Receiver<()>,
) -> (Sender<SocketMessage>, Receiver<SocketMessage>) {
    let (x_sender, mut x_receiver) = channel::<SocketMessage>(32);
    let (sender, receiver) = channel(32);

    let mut closed = false;

    tokio::spawn(async move {
        let (mut tx, mut rx) = socket.split();
        let timeout_duration = Duration::from_secs(CONFIG.timeout_secs);
        let timeout = sleep(timeout_duration);
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                Ok(()) = &mut close_signal => {
                    tracing::info!("{}: closing socket", id);
                    break;
                }
                Some(socketmessage) = x_receiver.recv() => {
                    tracing::info!("{:?}", &socketmessage);
                    let _ = tx.send(socketmessage.into_message()).await;
                },

                Some(Ok(msg)) = rx.next() => {
                    tracing::info!("{:?}", &msg);

                    match msg {
                        Message::Close(_) => {
                            tracing::info!("{}: client closed connection", &id);
                            closed = true;

                           // let _ = sender.send(SocketMessage::ConnectionClosed).await;
                           // break;
                        },
                        Message::Binary(bytes) => {
                            if closed {
                                // This shouldn't be possible.
                                tracing::error!("{}: received message after closed client", &id);
                            }

                            match serde_json::from_slice(&bytes) {
                                Ok(SocketMessage::Ping) => {
                                    timeout.as_mut().reset(Instant::now() + timeout_duration);
                                    let x = tx.send(SocketMessage::Ping.into_message()).await;
                                    tracing::info!("sending ping: {:?}", &x);
                                },
                                Ok(socket_message) => {
                                    let _ = sender.send(socket_message).await;
                                },
                                _ => {},
                            }
                        },
                        _ => {},
                    }
                },

                _ = &mut timeout => {
                    tracing::info!("{}: Timeout occurred, closing connection", &id);
                    let _ = sender.send(SocketMessage::ConnectionClosed).await;
                    break;
                }
            }
        }
    });

    (x_sender, receiver)
}

pub struct User {
    pub scores: Scores,
    pub id: String,
    pub con_time: SystemTime,
    pub receiver: Receiver<SocketMessage>,
    pub sender: Sender<SocketMessage>,
    close_signal: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl Drop for User {
    fn drop(&mut self) {
        let id = self.id.clone();
        let sender = self.close_signal.clone();
        tokio::spawn(async move {
            let mut x = sender.lock().await;
            if let Some(sender) = x.take() {
                let res = sender.send(());
                if res.is_err() {
                    tracing::error!("{}: failed to send close signal: {:?}", id, res);
                }
            }
        });
    }
}

impl User {
    pub fn new(scores: Scores, id: String, socket: WebSocket) -> Self {
        tracing::info!("user queued ");
        tracing::info!("!@#!$@@2");
        let con_time = SystemTime::now();

        let (onesend, onerecv) = oneshot::channel();

        let (sender, receiver) = handle_socket(socket, id.clone(), onerecv);

        User {
            scores,
            sender,
            receiver,
            id,
            con_time,
            close_signal: Arc::new(Mutex::new(Some(onesend))),
        }
    }

    pub async fn send(&mut self, msg: SocketMessage) -> Result<(), SendError<SocketMessage>> {
        self.sender.send(msg).await
    }

    pub async fn receive(&mut self) -> Option<SocketMessage> {
        self.receiver.recv().await
    }

    pub fn is_closed(&mut self) -> bool {
        while let Ok(msg) = self.receiver.try_recv() {
            if matches!(msg, SocketMessage::ConnectionClosed) {
                tracing::info!("user closed: {}", &self.id);
                return true;
            }
        }

        false
    }
}
