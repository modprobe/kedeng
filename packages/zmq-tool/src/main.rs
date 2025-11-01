use crate::cli::{Cli, Command};
use crate::headless::HeadlessMode;
use crate::log::{get_logger, init_logger};
use crate::receiver::{ZmqConfig, ZmqReceiver};
use crate::tui::TuiMode;
use clap::Parser;
use slog::error;
use tokio::signal;

mod cli;
mod headless;
mod listen;
mod log;
mod receiver;
mod tui;
mod xml_format;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.validate();

    init_logger(cli.verbose);

    let zmq_config = if let Some(preset) = &cli.connection.preset {
        match cli.topic.len() {
            0 => preset.zmq_config(),
            _ => ZmqConfig {
                topics: cli.topic.clone(),
                ..preset.zmq_config()
            },
        }
    } else {
        ZmqConfig {
            host: cli.connection.hostname.clone().unwrap(),
            port: cli.connection.port.unwrap(),
            topics: cli.topic.clone(),
        }
    };

    match cli.command {
        Command::Listen { output_dir } => {
            let (mut receiver, subscriber) = ZmqReceiver::new(zmq_config.clone())?;
            let mut headless = HeadlessMode::new(subscriber, output_dir);

            tokio::select! {
                result = receiver.start_receiving() => {
                    if result.is_err() {
                        error!(get_logger(), "{}", result.unwrap_err());
                    } else {
                        error!(get_logger(), "Receiver task stopped unexpectedly");
                    }
                },
                result = headless.run() => {
                    if result.is_err() {
                        error!(get_logger(), "{}", result.unwrap_err());
                    } else {
                        error!(get_logger(), "App task stopped unexpectedly");
                    }
                },
                _ = signal::ctrl_c() => {}
            }
        }
        Command::Ui => {
            let (mut receiver, subscriber) = ZmqReceiver::new(zmq_config.clone())?;
            let tui = TuiMode::new(subscriber);

            tokio::select! {
                _ = receiver.start_receiving() => {},
                _ = tui.run() => {},
                _ = signal::ctrl_c() => {}
            }

            ratatui::restore();
        }
    }

    Ok(())
}
