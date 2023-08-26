use chrono::{Utc, DateTime};
use zstd::stream::write::Encoder;
use std::fs::File;
use std::io::{Write, BufWriter, Result as IOResult};
use log::info;
use num_cpus;
use tokio::sync::mpsc::{Sender, Receiver};

const MAX_VOLUME_SIZE: usize = 100_000_000; // 100MB
const ZSTD_COMPRESSION_LEVEL: i32 = 21;

pub fn time_bucket_complete(last_bucket_start: DateTime<Utc>, minutes_per_bucket: i64) -> bool {
    let now = Utc::now();
    (now - last_bucket_start).num_minutes() >= minutes_per_bucket
}

pub fn generate_file_name(bucket_start_time: DateTime<Utc>, minutes_per_bucket: i64, volume: usize, temp: bool) -> String {
    let from_time = bucket_start_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let to_time = (bucket_start_time + chrono::Duration::minutes(minutes_per_bucket)).format("%Y-%m-%d %H:%M").to_string();
    let suffix = if temp { "temp.zstd" } else { "zstd" };
    format!("solana_data_archive__from_{}_to_{}_UTC__Volume_{}.{}", from_time, to_time, volume, suffix)
}

pub struct MessageDispatcher {
    sender: Sender<String>,
    receiver: Receiver<String>,
    encoder_manager: EncoderManager,
}

impl MessageDispatcher {
    pub fn new(buffer_count: usize, minutes_per_bucket: i64) -> (Self, Sender<()>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(buffer_count);
        let (error_sender, _) = tokio::sync::mpsc::channel(1);
        let encoder_manager = EncoderManager::new(minutes_per_bucket).unwrap();
        (
            Self {
                sender,
                receiver,
                encoder_manager,
            },
            error_sender,
        )
    }

    pub async fn receive_message(&mut self) -> Option<String> {
        self.receiver.recv().await
    }

    pub fn process_message(&mut self, message: String) -> IOResult<()> {
        self.encoder_manager.process_message(message)
    }

    pub fn clone_tx(&self) -> Sender<String> {
        self.sender.clone()
    }

    pub fn finish_and_create_new(&mut self) -> IOResult<()> {
        self.encoder_manager.finish_and_create_new()
    }
}

pub struct EncoderManager {
    encoder: Option<Encoder<'static, BufWriter<File>>>,
    bucket_start_time: DateTime<Utc>,
    minutes_per_bucket: i64,
    volume: usize,
    current_size: usize,
}

impl EncoderManager {
    fn create_new_encoder(&mut self, temp: bool) -> IOResult<()> {
        let file_name = generate_file_name(self.bucket_start_time, self.minutes_per_bucket, self.volume, temp);
        let file = File::create(&file_name)?;
        let buf_writer = BufWriter::new(file);
        let mut encoder = Encoder::new(buf_writer, ZSTD_COMPRESSION_LEVEL)?;
        let num_workers = num_cpus::get() as u32 / 2;
        encoder.multithread(num_workers)?;
        self.encoder = Some(encoder);
        Ok(())
    }

    pub fn new(minutes_per_bucket: i64) -> IOResult<Self> {
        let bucket_start_time = Utc::now();
        let mut manager = Self {
            encoder: None,
            bucket_start_time,
            minutes_per_bucket,
            volume: 1,
            current_size: 0,
        };
        manager.create_new_encoder(true)?;
        Ok(manager)
    }

    pub fn write_message(&mut self, message: String) -> IOResult<()> {
        if let Some(ref mut encoder) = self.encoder {
            let msg_bytes = message.into_bytes();
            encoder.write_all(&msg_bytes)?;
            self.current_size += msg_bytes.len();
        }
        Ok(())
    }

    pub fn nearing_max_volume(&self) -> bool {
        self.current_size >= MAX_VOLUME_SIZE
    }

    pub fn process_message(&mut self, message: String) -> IOResult<()> {
        if self.nearing_max_volume() {
            self.finish_and_create_new()?;
        } else if time_bucket_complete(self.bucket_start_time, self.minutes_per_bucket) {
            self.finish_and_create_new()?;
            self.bucket_start_time = Utc::now();
            self.volume = 1;  // Reset volume to 1 for the new time bucket
        }
        self.write_message(message)?;
        Ok(())
    }

    pub fn finish_and_create_new(&mut self) -> IOResult<()> {
        if let Some(encoder) = self.encoder.take() {
            let start_time = std::time::Instant::now();
            encoder.finish()?;
            let elapsed_time = start_time.elapsed();
            let temp_file_name = generate_file_name(self.bucket_start_time, self.minutes_per_bucket, self.volume, true);
            let final_file_name = generate_file_name(self.bucket_start_time, self.minutes_per_bucket, self.volume, false);
            std::fs::rename(&temp_file_name, &final_file_name)?;
            let compressed_file_size = std::fs::metadata(&final_file_name)?.len() as f64 / 1_048_576.0;
            let uncompressed_file_size = self.current_size as f64 / 1_048_576.0;
            let compression_ratio = compressed_file_size / uncompressed_file_size;
            info!("Compressed and saved data to {} in {:?} seconds! Compressed file size is {:.4}mb, compared to uncompressed file size of {:.4}mb. Compression ratio is {}.", final_file_name, elapsed_time.as_secs(), compressed_file_size, uncompressed_file_size, compression_ratio);
            self.current_size = 0;
            self.create_new_encoder(true)?;
            if self.nearing_max_volume() {
                self.volume += 1;  // Only increment volume if nearing max size
            }
        }
        Ok(())
    }
}    