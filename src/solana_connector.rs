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

        let subscriptions = vec![
            json!({"jsonrpc": "2.0", "id": 1, "method": "signatureSubscribe", "params": ["all"]}),
            json!({"jsonrpc": "2.0", "id": 2, "method": "slotSubscribe"}),
            json!({"jsonrpc": "2.0", "id": 3, "method": "slotsUpdatesSubscribe"}),
            json!({"jsonrpc": "2.0", "id": 4, "method": "blockSubscribe"}),
            json!({"jsonrpc": "2.0", "id": 5, "method": "logsSubscribe", "params": ["all"]}),
        ];

        let mut write_handle = write.sink_map_err(|e| format!("WebSocket write error: {}", e));
        
        // Send the subscriptions to the WebSocket
        for subscription in subscriptions {
            write_handle.send(Message::Text(subscription.to_string())).await?;
        }

        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(msg) => {
                        if msg.is_text() {
                            match msg.to_text() {
                                Ok(message_content) => {
                                    let json_msg: Result<serde_json::Value, _> = serde_json::from_str(&message_content);
                                    if let Ok(json_msg) = json_msg {
                                        if verbose_logging {
                                            let method = json_msg.get("method").and_then(|v| v.as_str()).unwrap_or("");
                                            debug!("Received message of type {}, length: {}", method, message_content.len());
                                        }
                                        tx.send(message_content.to_string()).await.ok();
                                    }
                                }
                                Err(err) => {
                                    error!("Error converting message to text: {}", err);
                                }
                            }
                        }
                    }
                    Err(err) => {
                        error!("Error reading message: {}", err);
                        error_tx.send(()).await.ok();
                        break;
                    }
                }
            }
        });
        Ok(SolanaConnector)
    }
}
