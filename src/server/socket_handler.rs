use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use futures::{future::BoxFuture, SinkExt};
use futures_util::StreamExt;
use tokio::{
    sync::mpsc::{self, error::SendError, Sender},
    time::sleep,
};

use crate::common::SocketMessage;

pub struct Handler {
    message_sender: Sender<Message>,
}

impl Handler {
    pub async fn start(
        socket: WebSocket,
        handle_socket_read: impl Fn(SocketMessage) -> BoxFuture<'static, ()> + Send + 'static,
    ) -> Self {
        let (mut socket_sender, mut socket_receiver) = socket.split();
        let (message_sender, mut message_receiver) = mpsc::channel::<Message>(32);

        let handler = Handler {
            message_sender: message_sender.clone(),
        };

        // Reader
        tokio::spawn(async move {
            while let Some(Ok(msg)) = socket_receiver.next().await {
                // Spawn another task cause message processing could take some time in the future
                tokio::spawn(handle_socket_read(SocketMessage::from(msg)));
            }
        });

        // Writer
        tokio::spawn(async move {
            while let Some(msg) = message_receiver.recv().await {
                socket_sender.feed(msg).await.unwrap();
            }
        });

        // Pinger
        tokio::spawn(async move {
            loop {
                match message_sender.send(Message::Ping(vec![])).await {
                    Ok(_) => {
                        sleep(Duration::new(5, 0)).await;
                    }
                    Err(_) => break,
                }
            }
        });

        handler
    }

    pub async fn write_socket_message(
        &self,
        message: SocketMessage,
    ) -> Result<(), SendError<Message>> {
        self.message_sender.send(message.into()).await
    }
}
