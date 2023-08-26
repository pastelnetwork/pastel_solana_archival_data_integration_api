use reqwest::Client;
use reqwest::Response;
use serde_json::{json, Value};
use std::error::Error;
use std::env;
use dotenv::dotenv;

pub async fn get_block_production() -> Result<Value, Box<dyn Error>> {
    post_request("getBlockProduction", json!({})).await
}

pub async fn get_block(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getBlock", params).await
}

pub async fn get_block_time(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getBlockTime", params).await
}

pub async fn get_block_commitment(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getBlockCommitment", params).await
}

pub async fn get_blocks_with_limit(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getBlocksWithLimit", params).await
}

pub async fn get_block_height() -> Result<Value, Box<dyn Error>> {
    post_request("getBlockHeight", json!({})).await
}

pub async fn get_blocks(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getBlocks", params).await
}

pub async fn is_blockhash_valid(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("isBlockhashValid", params).await
}

// Account Information

pub async fn get_balance(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getBalance", params).await
}

pub async fn get_largest_accounts() -> Result<Value, Box<dyn Error>> {
    post_request("getLargestAccounts", json!({})).await
}

pub async fn get_account_info(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getAccountInfo", params).await
}

pub async fn get_vote_accounts() -> Result<Value, Box<dyn Error>> {
    post_request("getVoteAccounts", json!({})).await
}

pub async fn get_multiple_accounts(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getMultipleAccounts", params).await
}

pub async fn get_program_accounts(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getProgramAccounts", params).await
}

// Node Information

pub async fn get_cluster_nodes() -> Result<Value, Box<dyn Error>> {
    post_request("getClusterNodes", json!({})).await
}

pub async fn get_health() -> Result<Value, Box<dyn Error>> {
    post_request("getHealth", json!({})).await
}

pub async fn get_version() -> Result<Value, Box<dyn Error>> {
    post_request("getVersion", json!({})).await
}

pub async fn get_identity() -> Result<Value, Box<dyn Error>> {
    post_request("getIdentity", json!({})).await
}


// Network Inflation Information

pub async fn get_inflation_governor() -> Result<Value, Box<dyn Error>> {
    post_request("getInflationGovernor", json!({})).await
}

pub async fn get_inflation_rate() -> Result<Value, Box<dyn Error>> {
    post_request("getInflationRate", json!({})).await
}

pub async fn get_inflation_reward(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getInflationReward", params).await
}

pub async fn get_supply() -> Result<Value, Box<dyn Error>> {
    post_request("getSupply", json!({})).await
}

// Network Information

pub async fn get_epoch_schedule() -> Result<Value, Box<dyn Error>> {
    post_request("getEpochSchedule", json!({})).await
}

pub async fn get_epoch_info() -> Result<Value, Box<dyn Error>> {
    post_request("getEpochInfo", json!({})).await
}

pub async fn get_fee_for_message(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getFeeForMessage", params).await
}

pub async fn get_highest_snapshot_slot() -> Result<Value, Box<dyn Error>> {
    post_request("getHighestSnapshotSlot", json!({})).await
}

pub async fn get_genesis_hash() -> Result<Value, Box<dyn Error>> {
    post_request("getGenesisHash", json!({})).await
}

pub async fn get_recent_performance_samples() -> Result<Value, Box<dyn Error>> {
    post_request("getRecentPerformanceSamples", json!({})).await
}

pub async fn get_first_available_block() -> Result<Value, Box<dyn Error>> {
    post_request("getFirstAvailableBlock", json!({})).await
}

pub async fn get_minimum_balance_for_rent_exemption(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getMinimumBalanceForRentExemption", params).await
}

// Transaction Information

pub async fn get_transaction(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getTransaction", params).await
}

pub async fn send_transaction(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("sendTransaction", params).await
}

pub async fn get_signature_statuses(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getSignatureStatuses", params).await
}

pub async fn get_signatures_for_address(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getSignaturesForAddress", params).await
}

pub async fn simulate_transaction(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("simulateTransaction", params).await
}

// Slot Information

pub async fn minimum_ledger_slot() -> Result<Value, Box<dyn Error>> {
    post_request("minimumLedgerSlot", json!({})).await
}

pub async fn get_max_shred_insert_slot() -> Result<Value, Box<dyn Error>> {
    post_request("getMaxShredInsertSlot", json!({})).await
}

pub async fn get_slot() -> Result<Value, Box<dyn Error>> {
    post_request("getSlot", json!({})).await
}

pub async fn get_slot_leader() -> Result<Value, Box<dyn Error>> {
    post_request("getSlotLeader", json!({})).await
}

pub async fn get_slot_leaders() -> Result<Value, Box<dyn Error>> {
    post_request("getSlotLeaders", json!({})).await
}

pub async fn get_max_retransmit_slot() -> Result<Value, Box<dyn Error>> {
    post_request("getMaxRetransmitSlot", json!({})).await
}

// Token Information

pub async fn get_token_accounts_by_owner(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getTokenAccountsByOwner", params).await
}

pub async fn get_token_account_balance(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getTokenAccountBalance", params).await
}

pub async fn get_token_supply(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getTokenSupply", params).await
}

// Helper function to make a POST request
pub async fn post_request(method: &str, params: Value) -> Result<Value, Box<dyn Error>> {
    let client = Client::new();
    let payload = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": method,
        "params": params
    });

    dotenv().ok(); // Load the .env file
    let alchemy_api_key = env::var("ALCHEMY_API_KEY").expect("ALCHEMY_API_KEY must be set"); // Retrieve the Alchemy API key from the environment variable
    let alchemy_url = format!("https://solana-mainnet.g.alchemy.com/v2/{}", alchemy_api_key);

    let response: Response = client
        .post(alchemy_url)
        .header("Content-Type", "application/json")
        .body(payload.to_string())
        .send()
        .await?;

    let data: Value = response.json().await?;
    Ok(data)
}