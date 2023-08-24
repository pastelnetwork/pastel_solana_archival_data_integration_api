use tokio_tungstenite::tungstenite::Message;
use futures_util::StreamExt;
use futures_util::SinkExt;
use std::error::Error;
use serde_json::json;
use tokio_tungstenite::connect_async;
use tokio::sync::mpsc::Sender;
use log::{info, debug, error};

const ALCHEMY_URL: &str = "wss://solana-mainnet.g.alchemy.com/v2/gVG8FAb-Y6MrxnkCIHtnP03OEbGMon6p";

pub struct SolanaConnector;

impl SolanaConnector {
    pub async fn new(tx: Sender<String>, error_tx: Sender<()>, verbose_logging: bool) -> Result<SolanaConnector, Box<dyn Error>> {
        info!("Initializing Solana connector...");

        let (ws_stream, _) = connect_async(ALCHEMY_URL).await?;
        let (write, mut read) = ws_stream.split();
        let mut write_handle = write.sink_map_err(|e| format!("WebSocket write error: {}", e));

        let subscriptions = vec![
            json!({"jsonrpc": "2.0", "id": 1, "method": "signatureSubscribe", "params": ["all"]}),
            json!({"jsonrpc": "2.0", "id": 2, "method": "slotSubscribe"}),
            json!({"jsonrpc": "2.0", "id": 3, "method": "slotsUpdatesSubscribe"}),
            json!({"jsonrpc": "2.0", "id": 4, "method": "blockSubscribe"}),
            json!({"jsonrpc": "2.0", "id": 5, "method": "logsSubscribe", "params": ["all"]}),
        ];
        tokio::spawn(async move {
            for subscription in subscriptions {
                write_handle.send(Message::Text(subscription.to_string())).await.ok();
            }
        });
        let error_tx_clone = error_tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(msg) => {
                        let message_content = msg.to_string();
                        let json_msg: Result<serde_json::Value, _> = serde_json::from_str(&message_content);
                        if let Ok(json_msg) = json_msg {
                            if let Some(method) = json_msg.get("method").and_then(|v| v.as_str()) {
                                if verbose_logging {
                                    debug!("Received message of type {}, length: {}", method, message_content.len());
                                }
                            }
                        }
                        tx.send(message_content).await.ok();
                    }
                    Err(err) => {
                        error!("Error reading message: {}", err);
                        // Send error signal to trigger reconnection
                        error_tx_clone.send(()).await.ok();
                        break;
                    }
                }
            }
        });
        Ok(SolanaConnector)
    }
}
