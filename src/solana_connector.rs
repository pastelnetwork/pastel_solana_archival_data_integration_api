use std::env;
use dotenv::dotenv;
use tokio_tungstenite::tungstenite::Message;
use futures_util::StreamExt;
use futures_util::SinkExt;
use std::error::Error;
use serde_json::json;
use tokio_tungstenite::connect_async;
use tokio::sync::mpsc::Sender;
use log::{info, debug, error};

pub struct SolanaConnector;

impl SolanaConnector {
    pub async fn new(tx: Sender<String>, verbose_logging: bool) -> Result<SolanaConnector, Box<dyn Error>> {
        dotenv().ok(); // Load the .env file
        let alchemy_api_key = env::var("ALCHEMY_API_KEY").expect("ALCHEMY_API_KEY must be set"); // Retrieve the Alchemy API key from the environment variable
        let alchemy_url = format!("wss://solana-mainnet.g.alchemy.com/v2/{}", alchemy_api_key);
        info!("Initializing Solana connector...");
        let (ws_stream, _) = connect_async(alchemy_url).await?;
        let (write, mut read) = ws_stream.split();
        let subscriptions = vec![
            json!({"jsonrpc": "2.0", "id": 1, "method": "slotSubscribe"}),
            json!({"jsonrpc": "2.0", "id": 2, "method": "slotsUpdatesSubscribe"}),
            json!({"jsonrpc": "2.0", "id": 3, "method": "blockSubscribe", "params": [{}, {"commitment": "confirmed", "encoding": "base64", "showRewards": true, "transactionDetails": "full"}]}),
            json!({"jsonrpc": "2.0", "id": 4, "method": "logsSubscribe", "params": ["allWithVotes", {"commitment": "finalized"}]}),
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
                                        if tx.send(message_content.to_string()).await.is_err() { // Corrected by calling .to_string()
                                            error!("Failed to send message to receiver.");
                                        }
                                    } else {
                                        error!("Failed to parse JSON message: {}", message_content);
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
                        break;
                    }
                }
            }
        });
        Ok(SolanaConnector)
    }
}        