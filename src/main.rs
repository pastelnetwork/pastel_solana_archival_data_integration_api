mod data_archiver;
mod solana_connector;

use tokio::sync::{mpsc, oneshot};
use std::collections::VecDeque;
use data_archiver::{compress_and_save_data, time_bucket_complete};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use chrono::Utc;
use log::{info, debug, error, LevelFilter};
use std::time::Duration;
use std::fs;
use tokio::signal::unix::{signal, SignalKind};
use sysinfo::{System, SystemExt, DiskExt, CpuExt};
use log4rs::append::rolling_file::{RollingFileAppender, policy::compound};
use log4rs::config::{Appender, Config, Root};
use log4rs::append::console::ConsoleAppender;

const BUFFER_MESSAGE_COUNT: usize = 100_000; // Number of messages in a buffer
const DELAY_IN_SECONDS_AFTER_DISCONNECT: u64 = 1;
const USE_VERBOSE_LOGGING: bool = true;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window_size = 10;
    fs::create_dir_all("log_file_backups")?;
    let fixed_window_roller = compound::roll::fixed_window::FixedWindowRoller::builder()
        .build("log_file_backups/log.{}.gz", window_size)?;
    let size_trigger = compound::trigger::size::SizeTrigger::new(25 * 1024 * 1024);
    let compound_policy = compound::CompoundPolicy::new(Box::new(size_trigger), Box::new(fixed_window_roller));
    let rolling_file = RollingFileAppender::builder()
        .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new("{d} - {l} - {m}\n")))
        .build("pastel_solana_archival_data_ingester.log", Box::new(compound_policy))?;
    let console = ConsoleAppender::builder().build();
    let config = Config::builder()
        .appender(Appender::builder().build("rolling_file", Box::new(rolling_file)))
        .appender(Appender::builder().build("console", Box::new(console)))
        .build(Root::builder().appender("rolling_file").appender("console").build(LevelFilter::Debug))?;
    log4rs::init_config(config)?;
    info!("Starting Pastel Solana Data Ingester...");
    let mut sys = System::new_all();
    let shared_buffer = Arc::new(Mutex::new(VecDeque::new()));
    let shared_buffer_clone = shared_buffer.clone();
    let total_message_size = Arc::new(AtomicUsize::new(0));
    let total_message_count = Arc::new(AtomicUsize::new(0));
    let total_message_size_clone = total_message_size.clone();
    let total_message_count_clone = total_message_count.clone();
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
    let mut term_signal = signal(SignalKind::terminate())?;
    let mut int_signal = signal(SignalKind::interrupt())?;
    tokio::spawn(async move {
        tokio::select! {
            _ = term_signal.recv() => {},
            _ = int_signal.recv() => {},
        }
        shutdown_tx.send(()).ok();
    });
    tokio::spawn(async move {
        loop {
            sys.refresh_cpu();
            sys.refresh_disks();
            let cpu_usage: f32 = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
            let cpu_usage = cpu_usage / sys.cpus().len() as f32;
            let total_buffer_usage = shared_buffer_clone.lock().unwrap().len();
            let disk_usage: u64 = sys.disks().iter().map(|d| d.available_space()).sum();
            let total_message_size = total_message_size.load(Ordering::Relaxed);
            let total_message_count = total_message_count.load(Ordering::Relaxed);
            let average_message_size = if total_message_count > 0 {
                total_message_size / total_message_count
            } else {
                0
            };
            debug!("CPU Usage: {}%, Total Buffer Usage: {}/{}, Disk Available Space: {} bytes, Average Message Size: {} bytes",
                cpu_usage, total_buffer_usage, BUFFER_MESSAGE_COUNT, disk_usage, average_message_size);
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
    loop { // Connection loop
        let (tx, mut rx) = mpsc::channel(BUFFER_MESSAGE_COUNT);
        let (error_tx, mut error_rx) = mpsc::channel(1);
        if solana_connector::SolanaConnector::new(tx, error_tx, USE_VERBOSE_LOGGING).await.is_err() {
            info!("Error initializing Solana connector. Reconnecting...");
            tokio::time::sleep(Duration::from_secs(DELAY_IN_SECONDS_AFTER_DISCONNECT)).await;
            continue;
        }
        let mut last_bucket_start = Utc::now();
        loop { // Processing loop
            tokio::select! {
                Some(msg) = rx.recv() => {
                    let message_size = msg.len();
                    total_message_size_clone.fetch_add(message_size, Ordering::Relaxed);
                    total_message_count_clone.fetch_add(1, Ordering::Relaxed);
                    let mut buffer_guard = shared_buffer.lock().unwrap();
                    buffer_guard.push_back(msg);
                    if time_bucket_complete(last_bucket_start) {
                        info!("Time bucket complete, now compressing data!");
                        let buffer_to_compress: VecDeque<_> = buffer_guard.drain(..).collect();
                        tokio::spawn(async move {
                            if let Err(err) = compress_and_save_data(last_bucket_start, &buffer_to_compress) {
                                error!("Error compressing and saving data: {}", err);
                            }
                        });
                        last_bucket_start = Utc::now(); // Update the last bucket start time
                    }
                }
                _ = error_rx.recv() => {
                    info!("Reconnecting...");
                    tokio::time::sleep(Duration::from_secs(DELAY_IN_SECONDS_AFTER_DISCONNECT)).await;
                    break; // Break from the processing loop to reinitialize the connector
                }
                _ = &mut shutdown_rx => {
                    info!("Gracefully shutting down...");
                    return Ok(());
                }
            }
        }
    }
}