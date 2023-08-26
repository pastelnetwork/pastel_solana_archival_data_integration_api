mod data_archiver;
mod solana_connector;

use data_archiver::MessageDispatcher;
use log::{info, debug, error, LevelFilter};
use std::time::Duration;
use std::fs;
use tokio::signal::unix::{signal, SignalKind};
use sysinfo::{System, SystemExt, DiskExt, CpuExt};
use log4rs::append::rolling_file::{RollingFileAppender, policy::compound};
use log4rs::config::{Appender, Config, Root};
use log4rs::append::console::ConsoleAppender;
use std::sync::atomic::{Ordering, AtomicBool};
use std::sync::Arc;
use std::fs::{create_dir_all, File, read_dir};
use std::io::{BufReader, Result as IOResult};
use std::path::Path;
use zstd::stream::read::Decoder;

const USE_VERBOSE_LOGGING: bool = true;
const DECOMPRESS_MESSAGE_BLOBS_FOR_DEBUGGING: bool = true; // Change to false to disable
const BUFFER_MESSAGE_COUNT: usize = 100_000;
const MINUTES_PER_BUCKET: i64 = 1;
const MAX_RECONNECT_ATTEMPTS: u64 = 10;
const DISK_SPACE_THRESHOLD: u64 = 1024 * 1024 * 1024; // 1 GB

pub fn decompress_zstd_files() -> IOResult<()> {
    if !DECOMPRESS_MESSAGE_BLOBS_FOR_DEBUGGING {
        return Ok(()); // Skip decompression if disabled
    }
    let output_dir = "extracted_compressed_message_blobs";
    create_dir_all(output_dir)?;
    for entry in read_dir(".")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension() == Some(std::ffi::OsStr::new("zstd")) {
            let file_stem = path.file_stem().unwrap().to_str().unwrap();
            if file_stem.ends_with(".temp") {
                continue; // Skip temporary files
            }
            let output_file_name = format!("{}/{}", output_dir, file_stem);
            // Skip if the output file already exists
            if Path::new(&output_file_name).exists() {
                continue;
            }
            let compressed_file = File::open(&path)?;
            let mut decompressor = Decoder::new(BufReader::new(compressed_file))?;
            let mut output_file = File::create(&output_file_name)?;
            std::io::copy(&mut decompressor, &mut output_file)?;
            info!("Successfully decompressed {} into {}", path.display(), output_file_name);
        }
    }
    Ok(())
}

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
    let insufficient_disk_space = Arc::new(AtomicBool::new(false));
    tokio::spawn({
        let insufficient_disk_space = insufficient_disk_space.clone();
        async move {
            loop {
                sys.refresh_all(); // Refresh the system info
                let cpu_usage: f32 = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
                let cpu_usage = cpu_usage / sys.cpus().len() as f32;
                // Filter disks based on mount point starting with "/dev/"
                let disk_usage: u64 = sys.disks().iter()
                .filter(|d| d.mount_point().to_str().map_or(false, |s| s == "/"))
                .map(|d| d.available_space())
                .sum();
                let disk_usage_gb = disk_usage as f64 / 1024.0 / 1024.0 / 1024.0; // Convert bytes to gigabytes
                debug!("CPU Usage: {}%, Disk Available Space: {} gb", cpu_usage, disk_usage_gb);
                if disk_usage < DISK_SPACE_THRESHOLD {
                    insufficient_disk_space.store(true, Ordering::Relaxed);
                    error!("Insufficient disk space. Exiting...");
                    break;
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    });
    if DECOMPRESS_MESSAGE_BLOBS_FOR_DEBUGGING { // Spawn a task to periodically decompress Zstd files if the option is enabled
        tokio::spawn(async {
            loop {
                if let Err(err) = decompress_zstd_files() {
                    error!("Error decompressing Zstd files: {}", err);
                }
                tokio::time::sleep(Duration::from_secs(60)).await; // Check every 60 seconds
            }
        });
    }
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
    let mut term_signal = signal(SignalKind::terminate())?;
    let mut int_signal = signal(SignalKind::interrupt())?;
    tokio::spawn(async move {
        tokio::select! {
            _ = term_signal.recv() => {},
            _ = int_signal.recv() => {},
        }
        shutdown_tx.send(()).ok();
    });
    let mut reconnect_attempts = 0;
    loop {
        if reconnect_attempts >= MAX_RECONNECT_ATTEMPTS || insufficient_disk_space.load(Ordering::Relaxed) {
            break;
        }
        let (mut message_dispatcher, _) = MessageDispatcher::new(BUFFER_MESSAGE_COUNT, MINUTES_PER_BUCKET);
        if solana_connector::SolanaConnector::new(message_dispatcher.clone_tx(), USE_VERBOSE_LOGGING).await.is_err() {
            info!("Error initializing Solana connector. Reconnecting in {} seconds...", reconnect_attempts + 1);
            reconnect_attempts += 1;
        } else {
            reconnect_attempts = 0;
        }
        loop {
            tokio::select! {
                msg = message_dispatcher.receive_message() => {
                    if let Some(msg) = msg {
                        if let Err(err) = message_dispatcher.process_message(msg) {
                            error!("Error processing message: {}", err);
                        }
                    }
                }
                _ = &mut shutdown_rx => {
                    info!("Gracefully shutting down...");
                    break;
                }
            }
        }
        message_dispatcher.finish_and_create_new()?;
    }
    Ok(())
}
