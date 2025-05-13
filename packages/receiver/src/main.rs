use std::env;
use std::error::Error;
use std::io::Read;
use std::sync::Mutex;

use async_nats::jetstream::stream;
use bytes::Bytes;
use flate2::bufread::GzDecoder;
use slog::debug;
use slog::info;
use slog::o;
use slog::Drain;
use slog::Logger;
use slog_term::{FullFormat, TermDecorator};
use zeromq::Socket;
use zeromq::SocketRecv;

fn get_logger() -> Logger {
    if !atty::is(atty::Stream::Stdout) {
        let drain = Mutex::new(slog_json::Json::default(std::io::stdout())).map(slog::Fuse);
        let drain = slog_async::Async::new(drain).build().fuse();

        return Logger::root(drain, o!());
    }

    let decorator = TermDecorator::new().build();
    let drain = FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    Logger::root(drain, o!())
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
enum Source {
    DVS,
    DAS,
    RIT,
    POS,
}

impl Source {
    fn from_envelope(envelope: &str) -> Option<Self> {
        match envelope {
            "/RIG/InfoPlusDASInterface4" => Some(Source::DAS),
            "/RIG/InfoPlusDVSInterface4" => Some(Source::DVS),
            "/RIG/InfoPlusRITInterface5" => Some(Source::RIT),
            "/RIG/NStreinpositiesInterface5" => Some(Source::POS),
            _ => None,
        }
    }

    fn as_envelope(&self) -> &'static str {
        match self {
            Source::DAS => "/RIG/InfoPlusDASInterface4",
            Source::DVS => "/RIG/InfoPlusDVSInterface4",
            Source::RIT => "/RIG/InfoPlusRITInterface5",
            Source::POS => "/RIG/NStreinpositiesInterface5",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Source::DAS => "DAS",
            Source::DVS => "DVS",
            Source::RIT => "RIT",
            Source::POS => "POS",
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let logger = get_logger();
    info!(logger, "Starting receiver");

    let mut zmq_socket = zeromq::SubSocket::new();
    zmq_socket
        .connect(
            env::var("NDOV_ZMQ_URL")
                .expect("NDOV_ZMQ_URL not set")
                .as_str(),
        )
        .await?;

    let nats_options = async_nats::ConnectOptions::with_user_and_password(
        env::var("NATS_USER").expect("NATS_USER not set"),
        env::var("NATS_PASSWORD").expect("NATS_PASSWORD not set"),
    );

    let nats_client = nats_options
        .connect(env::var("NATS_HOST").expect("NATS_HOST not set"))
        .await?;

    let nats_jetstream = async_nats::jetstream::new(nats_client);

    for source in [Source::DVS, Source::DAS, Source::POS, Source::RIT] {
        zmq_socket.subscribe(source.as_envelope()).await?;
        info!(logger, "Subscribed to envelope"; "envelope" => source.as_envelope());

        nats_jetstream
            .get_or_create_stream(stream::Config {
                name: source.name().into(),
                ..Default::default()
            })
            .await?;

        info!(logger, "Created stream"; "stream" => source.name());
    }

    loop {
        let msg = zmq_socket.recv().await?;
        assert!(msg.len() == 2);

        let source = String::from_utf8(msg.get(0).unwrap().clone().to_vec()).unwrap();
        let source = Source::from_envelope(source.as_str()).unwrap();

        let data = msg.get(1).unwrap().clone();
        let mut gz = GzDecoder::new(&data[..]);
        let mut unzipped_data = String::new();

        gz.read_to_string(&mut unzipped_data).unwrap();

        info!(logger, "Received message"; "envelope" => source.as_envelope());
        debug!(logger, "Decompressed message"; "envelope" => source.name(), "data" => unzipped_data.clone());

        nats_jetstream
            .publish(source.name(), Bytes::from(unzipped_data))
            .await?;

        info!(logger, "Published message"; "subject" => source.name());
    }
}
