mod solana_rest_api_functions;
use reqwest;
use std::process::Command;
use std::error::Error;
use std::fs;
use tokio;

pub struct OldFaithfulSolanaConnector;

impl OldFaithfulSolanaConnector {
    pub async fn new() -> Result<(), Box<dyn Error>> {
        // Get current epoch
        let epoch_info = solana_rest_api_functions::get_epoch_info().await?;
        let current_epoch: u64 = epoch_info["result"]["epoch"]
        .as_u64()
        .ok_or("Epoch is missing or not a u64")?;
    
        // Create directory if it doesn't exist
        let dir_path = "old_faithful_index_files";
        if !fs::metadata(&dir_path).is_ok() {
            fs::create_dir(&dir_path)?;
        }

        // Download indices
        Self::download_index(current_epoch, "cid-to-offset", dir_path).await?;
        Self::download_index(current_epoch, "slot-to-cid", dir_path).await?;
        Self::download_index(current_epoch, "sig-to-cid", dir_path).await?;

        // Run RPC server
        Self::run_rpc_server(current_epoch, dir_path);

        Ok(())
    }

    async fn download_index(epoch: u64, index_type: &str, dir_path: &str) -> Result<(), Box<dyn Error>> {
        let url = format!("https://files.old-faithful.net/{}/epoch-{}.car.{}.index", epoch, epoch, index_type);
        let resp = reqwest::get(&url).await?;
        let bytes = resp.bytes().await?;
        let file_path = format!("{}/epoch-{}.car.{}.index", dir_path, epoch, index_type);
        tokio::fs::write(&file_path, bytes).await?;
        Ok(())
    }

    fn run_rpc_server(epoch: u64, dir_path: &str) {
        let output = Command::new("faithful-cli")
            .arg("rpc-server-car")
            .arg("--listen")
            .arg(":7999")
            .arg(format!("{}/epoch-{}.car", dir_path, epoch))
            .arg(format!("{}/epoch-{}.car.*.cid-to-offset.index", dir_path, epoch))
            .arg(format!("{}/epoch-{}.car.*.slot-to-cid.index", dir_path, epoch))
            .arg(format!("{}/epoch-{}.car.*.sig-to-cid.index", dir_path, epoch))
            .output()
            .expect("Failed to execute command");

        if !output.status.success() {
            eprintln!("Failed to run RPC server: {:?}", output);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    OldFaithfulSolanaConnector::new().await?;
    Ok(())
}
