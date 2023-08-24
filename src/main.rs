mod data_archiver;
mod solana_connector;

use tokio::sync::{mpsc, oneshot};
use std::collections::VecDeque;
use data_archiver::{compress_and_save_data, time_bucket_complete};
use std::sync::{Arc, Mutex};
use chrono::Utc;
use log::{info, debug, error, LevelFilter};
use std::time::Duration;
use std::fs;
use tokio::signal::unix::{signal, SignalKind};
use sysinfo::{System, SystemExt, DiskExt, CpuExt};
use log4rs::append::rolling_file::{RollingFileAppender, policy::compound};
use log4rs::config::{Appender, Config, Root};
use log4rs::append::console::ConsoleAppender;

const BUFFER_COUNT: usize = 8;
const BUFFER_SIZE: usize = 1_073_741_824; // 1GB
const DELAY_IN_SECONDS_AFTER_DISCONNECT: u64 = 1;
const USE_VERBOSE_LOGGING: bool = false;

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
        .build(Root::builder().appender("rolling_file").appender("console").build(LevelFilter::Info))?;
    log4rs::init_config(config)?;
    info!("Starting Pastel Solana Data Ingester...");
    let mut sys = System::new_all();
    let buffers: Vec<_> = (0..BUFFER_COUNT).map(|_| Arc::new(Mutex::new(VecDeque::new()))).collect();
    let shared_buffers = buffers.clone();
    let mut last_bucket_start = Utc::now();
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
    let mut current_buffer_index = 0;
    tokio::spawn(async move {
        loop {
            sys.refresh_cpu();
            sys.refresh_disks();
            let cpu_usage: f32 = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
            let cpu_usage = cpu_usage / sys.cpus().len() as f32;
            let mut total_buffer_usage = 0;
            for buffer in &shared_buffers {
                total_buffer_usage += buffer.lock().unwrap().len();
            }
            let disk_usage: u64 = sys.disks().iter().map(|d| d.available_space()).sum();
            debug!("CPU Usage: {}%, Total Buffer Usage: {}/{}, Disk Available Space: {} bytes",
                cpu_usage, total_buffer_usage, BUFFER_COUNT * BUFFER_SIZE, disk_usage);
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
    loop {
        let (tx, mut rx) = mpsc::channel(BUFFER_SIZE);
        let (error_tx, mut error_rx) = mpsc::channel(1);
        if solana_connector::SolanaConnector::new(tx, error_tx, USE_VERBOSE_LOGGING).await.is_err() {
            info!("Error initializing Solana connector. Reconnecting...");
            tokio::time::sleep(Duration::from_secs(DELAY_IN_SECONDS_AFTER_DISCONNECT)).await;
            continue;
        }
        tokio::select! {
            Some(msg) = rx.recv() => {
                let mut buffer_guard = buffers[current_buffer_index].lock().unwrap();
                buffer_guard.push_back(msg);
                if time_bucket_complete(last_bucket_start) {
                    let buffer_to_compress: Vec<_> = buffer_guard.drain(..).collect();
                    info!("Time bucket {} complete, now compressing data!", last_bucket_start);
                    tokio::spawn(async move {
                        if let Err(err) = compress_and_save_data(last_bucket_start, &buffer_to_compress) {
                            error!("Error compressing and saving data: {}", err);
                        }
                    });
                    last_bucket_start = Utc::now();
                    current_buffer_index = (current_buffer_index + 1) % BUFFER_COUNT;
                }
            }
            _ = error_rx.recv() => {
                info!("Reconnecting...");
                tokio::time::sleep(Duration::from_secs(DELAY_IN_SECONDS_AFTER_DISCONNECT)).await;
            }
            _ = &mut shutdown_rx => {
                info!("Gracefully shutting down...");
                break;
            }
        }
    }
    Ok(())
}
