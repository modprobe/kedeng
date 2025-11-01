// use crate::cli::Cli;
// use crate::log;
// use crate::xml_format::{format_xml, highlight_xml, Theme};
// use flate2::bufread::GzDecoder;
// use slog::{error, info, Logger};
// use std::io::Read;
// use std::sync::atomic::AtomicBool;
// use std::sync::atomic::Ordering::SeqCst;
// use std::sync::Arc;
// use std::time::Duration;
// use tokio::io::{stdout, AsyncWriteExt, Stdout};
// use tokio::signal::unix::{signal, SignalKind};
// use tokio::sync::Mutex;
// use tokio::time::sleep;
// use tokio_util::task::TaskTracker;
// use zeromq::{Socket, SocketRecv, ZmqMessage};
//
// pub(crate) async fn main(cli: Arc<Cli>) -> anyhow::Result<()> {
//     let logger = Arc::new(log::get_logger(Arc::clone(&cli).verbose));
//
//     let running = Arc::new(AtomicBool::new(true));
//     let task_tracker = TaskTracker::new();
//     let stdout = Arc::new(Mutex::new(stdout()));
//
//     let running_clone = Arc::clone(&running);
//     let logger_clone = Arc::clone(&logger);
//     tokio::spawn(async move {
//         let mut sigint = signal(SignalKind::interrupt()).unwrap();
//         let mut sigterm = signal(SignalKind::terminate()).unwrap();
//
//         tokio::select! {
//             _  = sigint.recv() => {
//                 info!(logger_clone, "Received SIGINT");
//                 running_clone.store(false, SeqCst);
//             },
//             _  = sigterm.recv() => {
//                 info!(logger_clone, "Received SIGTERM");
//                 running_clone.store(false, SeqCst);
//             },
//         }
//     });
//
//     let (host, port) = if let Some(preset) = &cli.connection.preset {
//         preset.get_connection_details()
//     } else {
//         let host = cli.connection.hostname.clone().unwrap();
//         let port = cli.connection.port.clone().unwrap();
//         (host, port)
//     };
//
//     info!(logger, "Connecting to {}:{}", host, port);
//
//     let mut zmq_socket = zeromq::SubSocket::new();
//     zmq_socket
//         .connect(format!("tcp://{}:{}", host, port).as_str())
//         .await?;
//
//     if let Some(topic) = &cli.topic.clone() {
//         info!(logger, "Subscribing to topic {}", topic);
//         zmq_socket.subscribe(topic.as_str()).await?;
//     }
//
//     while running.load(SeqCst) {
//         tokio::select! {
//             msg_result = zmq_socket.recv() => {
//                 let msg = msg_result?;
//                 let log = Arc::clone(&logger);
//                 let stdout = Arc::clone(&stdout);
//                 let cli = Arc::clone(&cli);
//
//                 task_tracker.spawn(async move {
//                     if let Err(e) = process(msg, &cli.unzip.clone(), stdout, Arc::clone(&log)).await {
//                         error!(Arc::clone(&log), "Error processing message: {}", e);
//                     }
//                 });
//             }
//             _ = sleep(Duration::from_millis(500)) => {
//                 continue;
//             }
//         }
//     }
//
//     Ok(())
// }
//
// async fn process(
//     msg: ZmqMessage,
//     unzip: &bool,
//     stdout: Arc<Mutex<Stdout>>,
//     logger: Arc<Logger>,
// ) -> anyhow::Result<()> {
//     let (source, data) = receive(msg, unzip, Arc::clone(&logger))?;
//     let formatted_data = format_xml(data)?;
//
//     info!(Arc::clone(&logger), "Received message from {}", source);
//
//     let mut stdout_guard = stdout.lock().await;
//
//     let highlighted = highlight_xml(formatted_data, Theme::Light)?;
//     stdout_guard.write_all(highlighted.as_bytes()).await?;
//
//     Ok(())
// }
//
// fn receive(msg: ZmqMessage, unzip: &bool, logger: Arc<Logger>) -> anyhow::Result<(String, String)> {
//     assert_eq!(msg.len(), 2);
//
//     let source = String::from_utf8(msg.get(0).unwrap().to_vec())?;
//     let data = msg.get(1).unwrap().clone();
//
//     let data: String = if *unzip {
//         let mut gz = GzDecoder::new(&data[..]);
//         let mut buf = String::new();
//         gz.read_to_string(&mut buf)?;
//         buf
//     } else {
//         String::from_utf8(data.to_vec())?
//     };
//
//     Ok((source, data))
// }
