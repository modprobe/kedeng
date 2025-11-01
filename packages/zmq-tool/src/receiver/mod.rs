use crate::log::get_logger;
use anyhow::Result;
use bytes::Bytes;
use chrono::{DateTime, Local};
use slog::{debug, Logger};
use std::io::Read;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use zeromq::{Socket, SocketRecv, SubSocket};

#[derive(Debug, Clone)]
pub struct ZmqMessage<T> {
    pub timestamp: DateTime<Local>,
    pub topic: String,
    pub payload: T,
}

#[derive(Debug, Clone)]
pub struct ZmqConfig {
    pub host: String,
    pub port: u16,
    pub topics: Vec<String>,
}

impl ZmqConfig {
    fn endpoint(&self) -> String {
        format!("tcp://{}:{}", self.host, self.port)
    }
}

pub struct ZmqReceiver<T> {
    socket: SubSocket,
    socket_config: ZmqConfig,
    sender: Sender<ZmqMessage<T>>,

    logger: &'static Logger,
}

impl<T> ZmqReceiver<T>
where
    T: Clone + Send,
{
    pub fn new(config: ZmqConfig) -> Result<(Self, Receiver<ZmqMessage<T>>)> {
        let (sender, receiver) = broadcast::channel(1000);
        let mut socket = SubSocket::new();

        Ok((
            Self {
                socket,
                socket_config: config,
                sender,
                logger: get_logger(),
            },
            receiver,
        ))
    }

    async fn setup_socket(&mut self) -> Result<()> {
        self.socket.connect(&self.socket_config.endpoint()).await?;
        debug!(
            self.logger,
            "Connected to {}",
            self.socket_config.endpoint()
        );

        for topic in &self.socket_config.topics {
            self.socket.subscribe(topic).await?;
            debug!(self.logger, "Subscribed to topic {}", topic);
        }

        Ok(())
    }
}

impl ZmqReceiver<Bytes> {
    pub async fn start_receiving(&mut self) -> Result<()> {
        self.setup_socket().await?;

        loop {
            match self.socket.recv().await {
                Ok(msg) => {
                    let zmq_message = ZmqMessage {
                        timestamp: Local::now(),
                        topic: String::from_utf8(msg.get(0).unwrap().to_vec())?,
                        payload: msg.get(1).unwrap().clone(),
                    };

                    if let Err(e) = self.sender.send(zmq_message) {
                        eprintln!("Error sending message: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                }
            }
        }
    }
}

impl ZmqReceiver<String> {
    pub async fn start_receiving(&mut self) -> Result<()> {
        self.setup_socket().await?;

        loop {
            match self.socket.recv().await {
                Ok(msg) => {
                    let zmq_message = ZmqMessage {
                        timestamp: Local::now(),
                        topic: String::from_utf8(msg.get(0).unwrap().to_vec())?,
                        payload: self.unzip_payload(msg.get(1).unwrap().clone())?,
                    };

                    if let Err(e) = self.sender.send(zmq_message) {
                        eprintln!("Error sending message: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                }
            }
        }
    }

    fn unzip_payload(&self, payload: Bytes) -> Result<String> {
        let mut gz = flate2::read::GzDecoder::new(&payload[..]);
        let mut buf = String::new();
        gz.read_to_string(&mut buf)?;
        Ok(buf)
    }
}
