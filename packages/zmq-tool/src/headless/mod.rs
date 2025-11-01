use crate::log::get_logger;
use crate::receiver::ZmqMessage;
use crate::xml_format::format_xml;
use anyhow::Result;
use slog::{info, Logger};
use std::path::PathBuf;
use tokio::fs::create_dir_all;
use tokio::sync::broadcast::Receiver;

pub struct HeadlessMode {
    receiver: Receiver<ZmqMessage<String>>,
    output_dir: Option<PathBuf>,

    logger: &'static Logger,
}

impl HeadlessMode {
    pub fn new(receiver: Receiver<ZmqMessage<String>>, output_dir: Option<PathBuf>) -> Self {
        Self {
            receiver,
            output_dir,
            logger: get_logger(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.validate_output_dir().await?;

        while let Ok(message) = self.receiver.recv().await {
            self.handle_message(message).await?;
        }

        Ok(())
    }

    async fn handle_message(&self, message: ZmqMessage<String>) -> Result<()> {
        info!(self.logger, "Received message from {}", message.topic);

        let formatted_payload = format_xml(message.payload)?;

        match &self.output_dir {
            Some(dir) => {
                let sanitized_topic = message.topic.trim_start_matches('/');
                let sanitized_topic = sanitized_topic.replace("/", ".");

                let file_name =
                    format!("{}_{}.xml", message.timestamp.timestamp(), sanitized_topic);
                let file_path = dir.join(file_name);
                tokio::fs::write(file_path, formatted_payload).await?;
            }
            None => {
                info!(self.logger, "{}", formatted_payload);
            }
        }

        Ok(())
    }

    async fn validate_output_dir(&self) -> Result<()> {
        if self.output_dir.is_none() {
            return Ok(());
        }

        let dir = self.output_dir.as_ref().unwrap();
        if dir.exists() && !dir.is_dir() {
            return Err(anyhow::anyhow!(
                "Output directory {} exists but is not a directory",
                dir.display()
            ));
        }

        if !dir.exists() {
            create_dir_all(dir).await?;
        }

        Ok(())
    }
}
