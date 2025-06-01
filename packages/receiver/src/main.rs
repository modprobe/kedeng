mod instrumentation;

use crate::instrumentation::{get_meter, init_tracer_provider, shutdown_metrics};
use async_nats::jetstream::{stream, Context};
use async_nats::ConnectOptions;
use bytes::Bytes;
use flate2::bufread::GzDecoder;
use opentelemetry::metrics::Counter;
use opentelemetry::KeyValue;
use slog::{debug, error, info, o};
use slog::{Drain, Logger};
use slog_term::{FullFormat, TermDecorator};
use std::env;
use std::error::Error;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use async_nats::jetstream::stream::RetentionPolicy;
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{sleep, Duration};
use tokio_util::task::TaskTracker;
use zeromq::{Socket, SocketRecv, ZmqMessage};

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

async fn process(
    msg: ZmqMessage,
    nats: Arc<Context>,
    logger: Arc<Logger>,
    counter_received: Arc<Counter<u64>>,
    counter_published: Arc<Counter<u64>>,
) -> anyhow::Result<()> {
    assert_eq!(msg.len(), 2);

    let source = String::from_utf8(msg.get(0).unwrap().clone().to_vec())?;
    let source = Source::from_envelope(source.as_str()).unwrap();

    let data = msg.get(1).unwrap().clone();
    let mut gz = GzDecoder::new(&data[..]);
    let mut unzipped_data = String::new();

    gz.read_to_string(&mut unzipped_data)?;

    info!(logger, "Received message"; "envelope" => source.as_envelope());
    debug!(logger, "Decompressed message"; "envelope" => source.name(), "data" => unzipped_data.clone());
    counter_received.add(1, &[KeyValue::new("source", source.as_envelope())]);

    nats.publish(source.name(), Bytes::from(unzipped_data))
        .await?;

    counter_published.add(1, &[KeyValue::new("stream", source.name())]);

    info!(logger, "Published message"; "subject" => source.name());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let logger = Arc::new(get_logger());
    info!(logger, "Starting receiver");

    init_tracer_provider()?;

    let running = Arc::new(AtomicBool::new(true));
    let task_tracker = TaskTracker::new();

    // Set up a shutdown handler
    let running_clone = Arc::clone(&running);
    let logger_clone = Arc::clone(&logger);
    tokio::spawn(async move {
        let mut sigint =
            signal(SignalKind::interrupt()).expect("Failed to register SIGINT handler");
        let mut sigterm =
            signal(SignalKind::terminate()).expect("Failed to register SIGTERM handler");

        tokio::select! {
            _ = sigint.recv() => {
                info!(logger_clone, "Received SIGINT");
                running_clone.store(false, Ordering::SeqCst);
            }
            _ = sigterm.recv() => {
                info!(logger_clone, "Received SIGTERM");
                running_clone.store(false, Ordering::SeqCst);
            }
        }
    });

    let mut zmq_socket = zeromq::SubSocket::new();
    zmq_socket
        .connect(
            env::var("NDOV_ZMQ_URL")
                .expect("NDOV_ZMQ_URL not set")
                .as_str(),
        )
        .await?;

    let nats_options = if env::var("NATS_USER").is_ok() && env::var("NATS_PASSWORD").is_ok() {
        ConnectOptions::with_user_and_password(
            env::var("NATS_USER").unwrap(),
            env::var("NATS_PASSWORD").unwrap(),
        )
    } else {
        ConnectOptions::default()
    };

    let nats_client = nats_options
        .connect(env::var("NATS_HOST").expect("NATS_HOST not set"))
        .await?;

    let nats_jetstream = Arc::new(async_nats::jetstream::new(nats_client));

    for source in [
        Source::DVS,
        Source::DAS,
        Source::RIT,
        // Source::POS,
    ] {
        zmq_socket.subscribe(source.as_envelope()).await?;
        info!(logger, "Subscribed to envelope"; "envelope" => source.as_envelope());

        nats_jetstream
            .get_or_create_stream(stream::Config {
                name: source.name().into(),
                retention: RetentionPolicy::WorkQueue,
                ..Default::default()
            })
            .await?;

        info!(logger, "Created stream"; "stream" => source.name());
    }

    let log_clone = Arc::clone(&logger);
    let counter_received = Arc::new(get_meter().u64_counter("kedeng_messages_received").build());
    let counter_published = Arc::new(get_meter().u64_counter("kedeng_messages_published").build());

    while running.load(Ordering::SeqCst) {
        tokio::select! {
            msg_result = zmq_socket.recv() => {
                let msg = msg_result?;

                let nats = Arc::clone(&nats_jetstream);
                let log = Arc::clone(&log_clone);

                let counter_received = Arc::clone(&counter_received);
                let counter_published = Arc::clone(&counter_published);

                task_tracker.spawn(async move {
                    if let Err(e) = process(msg, nats, log.clone(), counter_received.clone(), counter_published.clone()).await {
                        error!(log, "Error processing message: {}", e);
                    }
                });
            }
            _ = sleep(Duration::from_millis(500)) => {
                continue;
            }
        }
    }

    info!(logger, "Shutting down gracefully");
    task_tracker.close();

    info!(logger, "Waiting for all tasks to complete");
    task_tracker.wait().await;
    shutdown_metrics()?;

    info!(logger, "All tasks completed, shutdown complete");
    Ok(())
}
