use reqwest::Client;
use serde_json::{json, Value};
use std::error::Error;

const ALCHEMY_URL: &str = "wss://solana-mainnet.g.alchemy.com/v2/gVG8FAb-Y6MrxnkCIHtnP03OEbGMon6p";


async fn get_block_production() -> Result<Value, Box<dyn Error>> {
    post_request("getBlockProduction", json!({})).await
}

async fn get_block(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getBlock", params).await
}

async fn get_block_time(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getBlockTime", params).await
}

async fn get_block_commitment(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getBlockCommitment", params).await
}

async fn get_blocks_with_limit(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getBlocksWithLimit", params).await
}

async fn get_block_height() -> Result<Value, Box<dyn Error>> {
    post_request("getBlockHeight", json!({})).await
}

async fn get_blocks(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getBlocks", params).await
}

async fn is_blockhash_valid(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("isBlockhashValid", params).await
}

// Account Information

async fn get_balance(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getBalance", params).await
}

async fn get_largest_accounts() -> Result<Value, Box<dyn Error>> {
    post_request("getLargestAccounts", json!({})).await
}

async fn get_account_info(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getAccountInfo", params).await
}

async fn get_vote_accounts() -> Result<Value, Box<dyn Error>> {
    post_request("getVoteAccounts", json!({})).await
}

async fn get_multiple_accounts(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getMultipleAccounts", params).await
}

async fn get_program_accounts(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getProgramAccounts", params).await
}

// Node Information

async fn get_cluster_nodes() -> Result<Value, Box<dyn Error>> {
    post_request("getClusterNodes", json!({})).await
}

async fn get_health() -> Result<Value, Box<dyn Error>> {
    post_request("getHealth", json!({})).await
}

async fn get_version() -> Result<Value, Box<dyn Error>> {
    post_request("getVersion", json!({})).await
}

async fn get_identity() -> Result<Value, Box<dyn Error>> {
    post_request("getIdentity", json!({})).await
}


// Network Inflation Information

async fn get_inflation_governor() -> Result<Value, Box<dyn Error>> {
    post_request("getInflationGovernor", json!({})).await
}

async fn get_inflation_rate() -> Result<Value, Box<dyn Error>> {
    post_request("getInflationRate", json!({})).await
}

async fn get_inflation_reward(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getInflationReward", params).await
}

async fn get_supply() -> Result<Value, Box<dyn Error>> {
    post_request("getSupply", json!({})).await
}

// Network Information

async fn get_epoch_schedule() -> Result<Value, Box<dyn Error>> {
    post_request("getEpochSchedule", json!({})).await
}

async fn get_epoch_info() -> Result<Value, Box<dyn Error>> {
    post_request("getEpochInfo", json!({})).await
}

async fn get_fee_for_message(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getFeeForMessage", params).await
}

async fn get_highest_snapshot_slot() -> Result<Value, Box<dyn Error>> {
    post_request("getHighestSnapshotSlot", json!({})).await
}

async fn get_genesis_hash() -> Result<Value, Box<dyn Error>> {
    post_request("getGenesisHash", json!({})).await
}

async fn get_recent_performance_samples() -> Result<Value, Box<dyn Error>> {
    post_request("getRecentPerformanceSamples", json!({})).await
}

async fn get_first_available_block() -> Result<Value, Box<dyn Error>> {
    post_request("getFirstAvailableBlock", json!({})).await
}

async fn get_minimum_balance_for_rent_exemption(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getMinimumBalanceForRentExemption", params).await
}

// Transaction Information

async fn get_transaction(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getTransaction", params).await
}

async fn send_transaction(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("sendTransaction", params).await
}

async fn get_signature_statuses(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getSignatureStatuses", params).await
}

async fn get_signatures_for_address(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getSignaturesForAddress", params).await
}

async fn simulate_transaction(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("simulateTransaction", params).await
}

// Slot Information

async fn minimum_ledger_slot() -> Result<Value, Box<dyn Error>> {
    post_request("minimumLedgerSlot", json!({})).await
}

async fn get_max_shred_insert_slot() -> Result<Value, Box<dyn Error>> {
    post_request("getMaxShredInsertSlot", json!({})).await
}

async fn get_slot() -> Result<Value, Box<dyn Error>> {
    post_request("getSlot", json!({})).await
}

async fn get_slot_leader() -> Result<Value, Box<dyn Error>> {
    post_request("getSlotLeader", json!({})).await
}

async fn get_slot_leaders() -> Result<Value, Box<dyn Error>> {
    post_request("getSlotLeaders", json!({})).await
}

async fn get_max_retransmit_slot() -> Result<Value, Box<dyn Error>> {
    post_request("getMaxRetransmitSlot", json!({})).await
}

// Token Information

async fn get_token_accounts_by_owner(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getTokenAccountsByOwner", params).await
}

async fn get_token_account_balance(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getTokenAccountBalance", params).await
}

async fn get_token_supply(params: Value) -> Result<Value, Box<dyn Error>> {
    post_request("getTokenSupply", params).await
}

// Helper function to make a POST request
async fn post_request(method: &str, params: Value) -> Result<Value, Box<dyn Error>> {
    let client = Client::new();
    let payload = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": method,
        "params": params
    });

    let response = client
        .post(ALCHEMY_URL)
        .header("Content-Type", "application/json")
        .body(payload.to_string())
        .send()
        .await?;

    let data: Value = response.json().await?;
    Ok(data)
}