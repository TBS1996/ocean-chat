use crate::common;

use axum::extract::ws::{Message, WebSocket};
use common::CONFIG;
use common::{Scores, SocketMessage};
use futures::stream::SplitStream;
use futures_util::StreamExt;
use std::time::SystemTime;

use futures_util::SinkExt;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{self, channel, Receiver, Sender};
use tokio::time::{sleep, Duration, Instant};

/// Takes care of sending to and receiving from a websocket.
fn handle_socket(
    socket: WebSocket,
    id: String,
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
                Some(socketmessage) = x_receiver.recv() => {
                    let _ = tx.send(socketmessage.into_message()).await;
                },

                Some(Ok(msg)) = rx.next() => {
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
    pub message_sender: Sender<Message>,
    pub socket_receiver: SplitStream<WebSocket>,
}

impl User {
    pub async fn new(scores: Scores, id: String, socket: WebSocket) -> Self {
        tracing::info!("user queued ");
        let con_time = SystemTime::now();

        let (mut socket_sender, socket_receiver) = socket.split();
        let (message_sender, mut message_receiver) = mpsc::channel::<Message>(32);

        // Writer
        tokio::spawn(async move {
            while let Some(msg) = message_receiver.recv().await {
                socket_sender.feed(msg).await.unwrap();
            }
        });

        let pinger_message_sender = message_sender.clone();
        // Pinger
        tokio::spawn(async move {
            loop {
                match pinger_message_sender.send(Message::Ping(vec![])).await {
                    Ok(_) => {
                        sleep(Duration::new(5, 0)).await;
                    }
                    Err(_) => break,
                }
            }
        });

        User {
            scores,
            id,
            con_time,
            message_sender,
            socket_receiver,
        }
    }

    #[deprecated(note = "Please check for `Messaage::Close` instead")]
    pub fn is_closed(&mut self) -> bool {
        todo!()
    }
}
