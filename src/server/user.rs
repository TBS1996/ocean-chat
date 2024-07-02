use crate::common;

use crate::server::StateAction;
use crate::server::StateMessage;
use axum::extract::ws::{Message, WebSocket};
use tokio::sync::mpsc;

use common::CONFIG;
use common::{Scores, SocketMessage};
use futures_util::StreamExt;
use std::time::SystemTime;

use futures_util::SinkExt;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::{sleep, Duration, Instant};

macro_rules! logerr {
    ($result:expr, $action:tt) => {
        match $result {
            Ok(value) => value,
            Err(e) => {
                tracing::error!("{}: line {} -> {:?}", file!(), line!(), e);
                $action;
            }
        }
    };
}

/// Takes care of sending to and receiving from a websocket.
fn handle_socket(
    socket: WebSocket,
    id: String,
    upsender: mpsc::Sender<StateMessage>,
    mut close_signal: mpsc::Receiver<()>,
) -> (Sender<SocketMessage>, Receiver<SocketMessage>) {
    let (x_sender, mut x_receiver) = channel::<SocketMessage>(32);
    let (sender, receiver) = channel(32);

    tokio::spawn(async move {
        let (mut tx, mut rx) = socket.split();
        let timeout_duration = Duration::from_secs(CONFIG.timeout_secs);
        let timeout = sleep(timeout_duration);
        tokio::pin!(timeout);

        loop {
            let close_signal_recv = close_signal.recv();
            tokio::pin!(close_signal_recv);

            tokio::select! {
                Some(()) = &mut close_signal_recv => {
                    tracing::info!("{}: closing socket", id);
                    let msg = StateMessage::new(id.clone(), StateAction::RemoveUser);
                    upsender.send(msg).await.ok();
                    break;
                }
                Some(socketmessage) = x_receiver.recv() => {
                    match socketmessage {
                        SocketMessage::GetStatus => {
                            let (tx_, rx) = tokio::sync::oneshot::channel();
                            let msg = StateMessage::new(id.clone(), StateAction::GetStatus(tx_));
                            logerr!(upsender.send(msg).await, continue);
                            let status = logerr!(rx.await, continue);
                            logerr!(tx.send(SocketMessage::Status(status).into_message()).await, continue);
                        },
                        socketmessage => {
                            tracing::info!("{:?}", &socketmessage);
                            let _ = tx.send(socketmessage.into_message()).await;
                        }
                    }
                },

                Some(Ok(msg)) = rx.next() => {
                    timeout.as_mut().reset(Instant::now() + timeout_duration);

                    match msg {
                        Message::Close(_) => {
                            tracing::info!("{}: client closed connection", &id);
                            let msg = StateMessage::new(id.clone(), StateAction::RemoveUser);
                            upsender.send(msg).await.ok();
                        },
                        Message::Binary(bytes) => {
                            let msg = serde_json::from_slice(&bytes);

                            match msg {
                                Ok(SocketMessage::GetStatus) => {
                                    let (tx_, rx) = tokio::sync::oneshot::channel();
                                    let msg = StateMessage::new(id.clone(), StateAction::GetStatus(tx_));
                                    logerr!(upsender.send(msg).await, continue);
                                    let status = logerr!(rx.await, continue);
                                    logerr!(tx.send(SocketMessage::Status(status).into_message()).await, continue);

                                }
                                Ok(SocketMessage::StateChange(new_state)) => {
                                    let upmsg = StateMessage {id: id.clone(), action: StateAction::StateChange(new_state)};
                                    logerr!(upsender.send(upmsg).await, continue);
                                },
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
                    let msg = StateMessage::new(id.clone(), StateAction::RemoveUser);
                    upsender.send(msg).await.ok();
                    break;
                }
            }
        }
    });

    (x_sender, receiver)
}

#[derive(Debug)]
pub struct User {
    pub id: String,
    pub scores: Scores,
    pub con_time: SystemTime,
    pub sender: Sender<SocketMessage>,
    pub receiver: Receiver<SocketMessage>,
    close_signal: Option<mpsc::Sender<()>>,
}

impl User {
    pub fn new(
        scores: Scores,
        id: String,
        socket: WebSocket,
        tx: mpsc::Sender<StateMessage>,
    ) -> Self {
        tracing::info!("user queued ");
        let con_time = SystemTime::now();

        let (onesend, onerecv) = tokio::sync::mpsc::channel(1);

        let (sender, receiver) = handle_socket(socket, id.clone(), tx, onerecv);

        User {
            scores,
            sender,
            receiver,
            id,
            con_time,
            close_signal: Some(onesend),
        }
    }

    /// Notifies the client of the status of the user.
    ///
    /// Client will ask periodically but this should be called when a new status is registered
    /// just to speed things up.
    pub async fn refresh_status(&mut self) -> Result<(), SendError<SocketMessage>> {
        self.sender.send(SocketMessage::GetStatus).await
    }

    pub async fn send(&mut self, msg: SocketMessage) -> Result<(), SendError<SocketMessage>> {
        self.sender.send(msg).await
    }

    pub async fn receive(&mut self) -> Option<SocketMessage> {
        self.receiver.recv().await
    }

    pub async fn close(&mut self) {
        let id = self.id.clone();
        if let Some(sender) = self.close_signal.take() {
            let res = sender.send(()).await;
            if res.is_err() {
                tracing::error!("{}: failed to send close signal: {:?}", id, res);
            }
        };
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

