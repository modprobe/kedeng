use crate::receiver::ZmqConfig;
use clap::error::ErrorKind;
use clap::error::ErrorKind::{ArgumentConflict, MissingRequiredArgument};
use clap::ValueHint::DirPath;
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "zeroMQ listener tool")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,

    #[command(flatten)]
    pub(crate) connection: Connection,

    #[arg(
        short = 't',
        long = "topic",
        help = "Subscribe to specific topics after connecting"
    )]
    pub(crate) topic: Vec<String>,

    #[arg(short = 'z', long = "unzip", help = "Unzip gzipped messages")]
    pub(crate) unzip: bool,

    #[arg(
        short = 'v',
        long = "verbose",
        default_value = "false",
        help = "Enable verbose logging"
    )]
    pub(crate) verbose: bool,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    #[command(about = "listen for messages and output them either to stdout or to a directory")]
    Listen {
        #[arg(short = 'o', long = "output-dir", help = "Directory in which to save the received messages", value_hint = DirPath)]
        output_dir: Option<PathBuf>,
    },

    #[command(about = "Launch a terminal UI for working with ZMQ messages")]
    Ui,
}

#[derive(Args, Clone)]
#[group(required = true, multiple = false)]
pub(crate) struct Connection {
    #[arg(
        short = 'p',
        long = "preset",
        help = "Use a preset to connect to a ZMQ server"
    )]
    pub(crate) preset: Option<Preset>,

    #[arg(
        short = 'H',
        long = "host",
        requires = "port",
        help = "Hostname of the ZMQ server"
    )]
    pub(crate) hostname: Option<String>,

    #[arg(
        short = 'P',
        long = "port",
        requires = "hostname",
        help = "Port of the ZMQ server"
    )]
    pub(crate) port: Option<u16>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, ValueEnum)]
pub enum Preset {
    BISON,
    Kv78Turbo,
    NSInfoPlus,
    SIRI,
}

impl Preset {
    const HOSTNAME: &'static str = "pubsub.besteffort.ndovloket.nl";

    pub fn zmq_config(&self) -> ZmqConfig {
        ZmqConfig {
            host: Self::HOSTNAME.to_string(),
            port: match self {
                Preset::BISON => 7658,
                Preset::Kv78Turbo => 7817,
                Preset::NSInfoPlus => 7664,
                Preset::SIRI => 7666,
            },
            topics: vec!["/RIG/".into()],
        }
    }
}

impl Connection {
    fn validate(&self) -> Result<(), (ErrorKind, String)> {
        match (&self.preset, &self.hostname, &self.port) {
            (Some(_), None, None) => Ok(()),
            (None, Some(_), Some(_)) => Ok(()),
            (Some(_), Some(_), _) | (Some(_), _, Some(_)) => Err((
                ArgumentConflict,
                "--preset cannot be used with --host or --port".to_string(),
            )),
            (None, Some(_), None) => Err((
                ArgumentConflict,
                "--host requires --port to be specified".to_string(),
            )),
            (None, None, Some(_)) => Err((
                ArgumentConflict,
                "--port requires --host to be specified".to_string(),
            )),
            (None, None, None) => Err((
                MissingRequiredArgument,
                "Either --preset or both --host and --port must be specified".to_string(),
            )),
        }
    }
}

impl Cli {
    pub fn validate(&self) {
        if let Err((error_kind, msg)) = self.connection.validate() {
            Cli::command().error(error_kind, msg).exit();
        }
    }
}
