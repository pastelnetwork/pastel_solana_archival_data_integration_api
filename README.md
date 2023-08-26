# Pastel Solana Data Ingester

## Overview

The Pastel Solana Data Ingester is a robust, high-performance data archival tool designed to ingest all network activity on the Solana network. It bundles this data into 1-minute duration files, highly compressed with Zstd, to prepare them for long-term archival storage on the Pastel Network's decentralized Cascade file storage layer. The ingester is written in Rust and leverages asynchronous programming with Tokio and WebSockets for real-time data streaming.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Performance Optimizations](#performance-optimizations)
- [Installation](#installation)
- [Usage](#usage)
- [License](#license)

## Features

1. **Real-time Data Ingestion**: Utilizes the Solana network's real-time WebSockets API to ingest various types of network activity.
2. **Data Compression**: Utilizes Zstd for high-efficiency data compression.
3. **Time Bucketing**: Aggregates data in 1-minute time buckets for more structured data storage.
4. **Multi-threading**: Employs multi-threading to further optimize Zstd compression.
5. **Disk and CPU Monitoring**: Constantly monitors disk space and CPU usage to ensure efficient operation.
6. **Graceful Shutdown**: Listens to system termination signals for graceful shutdown.
7. **Logging**: Extensive logging capabilities, with options for both console and file output.
8. **Error Handling**: Robust error handling for both I/O operations and Solana connector failures.

## Performance Optimizations

### Asynchronous Programming

The use of Tokio provides a non-blocking, asynchronous runtime that allows for handling multiple tasks concurrently without waiting for blocking I/O operations.

### Data Compression

Data is compressed using Zstd with a high compression level (21), ensuring that the archived files are as small as possible for storage. This significantly reduces the amount of disk space required for long-term storage.

### CPU Utilization

The Zstd compression encoder is configured to use half the number of available CPU cores, balancing compression speed with available system resources.

### Time Bucketing

Data is bundled into 1-minute time buckets, which allows for easier querying and extraction of specific time ranges from the archival storage.

### Disk Space Monitoring

A separate asynchronous task monitors available disk space and triggers an alert if it falls below a threshold (1 GB), gracefully shutting down the application to prevent data loss or corruption.

### Efficient Message Handling

The program uses Tokio's message-passing channels with backpressure handling to efficiently process incoming Solana messages. It also uses buffered writing to improve I/O performance.

## Installation

Please follow the standard Rust project build process to compile the code:

```bash
cargo build --release
```

## Usage

After compiling, run the executable:

```bash
./target/release/pastel_solana_data_ingester
```

## License

This project is licensed under the MIT License.

---

The Pastel Solana Data Ingester is an advanced data archival solution, optimized for performance and designed to serve as a robust, long-term storage mechanism for Solana network activity.
