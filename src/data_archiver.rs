use chrono::{Utc, DateTime};
use serde_json::json;
use zstd::stream::encode_all;
use std::fs::File;
use std::io::{Write, BufWriter, Error as IOError};
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use log::info;

const MAX_VOLUME_SIZE: usize = 100_000_000; // 100MB
const ZSTD_COMPRESSION_LEVEL: i32 = 21;

pub fn time_bucket_complete(last_bucket_start: DateTime<Utc>) -> bool {
    let now = Utc::now();
    (now - last_bucket_start).num_minutes() >= 1
}

pub fn generate_file_name(from_time: &str, to_time: &str, volume: usize) -> String {
    format!(
        "solana_data_archive__from_{}_to_{}_UTC__Volume_{}.zstd",
        from_time, to_time, volume
    )
}

pub fn compress_and_save_data(bucket_start_time: DateTime<Utc>, buffer: &[String]) -> Result<(), IOError> {
    let compression_start_time: Instant = Instant::now();
    let volume_counter = AtomicUsize::new(1);
    let from_time = bucket_start_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let to_time = (bucket_start_time + chrono::Duration::minutes(1)).format("%Y-%m-%d %H:%M:%S").to_string();
    buffer.par_chunks(std::cmp::min(MAX_VOLUME_SIZE, buffer.len())).try_for_each(|segment| {
        let segment_json = json!(segment).to_string();
        let compressed_data = encode_all(segment_json.as_bytes(), ZSTD_COMPRESSION_LEVEL).map_err(|e| IOError::new(std::io::ErrorKind::Other, e))?;
        let volume = volume_counter.fetch_add(1, Ordering::Relaxed);
        let file_name = generate_file_name(&from_time, &to_time, volume);
        let file = File::create(&file_name)?;
        let mut tmp_file = BufWriter::new(file);
        tmp_file.write_all(&compressed_data)?;
        tmp_file.flush()?;

        let original_size = segment.iter().map(|s| s.len()).sum::<usize>(); // Correct calculation of original size
        let compressed_file_size = std::fs::metadata(&file_name)?.len();
        let compression_ratio = compressed_file_size as f64 / original_size as f64; // Correct calculation of compression ratio
        let elapsed_time = compression_start_time.elapsed();
        info!("Compressed and saved data to {} in {:?} seconds! Compressed file size is {}mb, compared to uncompressed file size of {}mb. Compression ratio is {}.", file_name, elapsed_time.as_secs(), compressed_file_size / 1_000_000, original_size / 1_000_000, compression_ratio);
        Ok(())
    })
}
