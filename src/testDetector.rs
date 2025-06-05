use anyhow::{Result, anyhow};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{info, error};

// Configuration - OwnershipTransferred event detection
const OWNERSHIP_TRANSFERRED_TOPIC: &str = "0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0";
const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
const TARGET_NEW_OWNER: &str = "0xE220329659D41B2a9F26E83816B424bDAcF62567";

#[derive(Debug, Clone)]
pub struct TokenResult {
    pub token: String,
    pub block_number: u64,
    pub transaction_hash: String,
    pub previous_owner: String,
    pub new_owner: String,
}

// Optimized detector using OwnershipTransferred events - SPEED OPTIMIZED
pub struct TokenDetector {
    wss_url: String,
}

impl TokenDetector {
    pub fn new() -> Result<Self> {
        let wss_url = std::env::var("WSS_URL")
            .map_err(|_| anyhow!("WSS_URL environment variable not set"))?;
        
        info!("🚀 OPTIMIZED OwnershipTransferred detector initialized");
        info!("🎯 Target new owner: {}", TARGET_NEW_OWNER);
        info!("⚡ Speed mode: No caching, immediate returns");
        
        Ok(Self {
            wss_url,
        })
    }

    // Fast OwnershipTransferred event processing - OPTIMIZED FOR SPEED
    fn process_ownership_event(&self, log_data: &Value) -> Option<TokenResult> {
        // Extract data immediately - minimal allocations
        let tx_hash = log_data["transactionHash"].as_str()?;
        let block_hex = log_data["blockNumber"].as_str()?;
        let token_address = log_data["address"].as_str()?; // Contract that emitted = token address
        
        // Parse block number quickly
        let block_number = u64::from_str_radix(&block_hex[2..], 16).ok()?;
        
        // Extract topics - OwnershipTransferred event structure
        let topics = log_data["topics"].as_array()?;
        if topics.len() < 3 {
            return None;
        }
        
        // Fast topic extraction - no unnecessary allocations
        let previous_owner = topics[1].as_str()?.trim_start_matches("0x");
        let new_owner = topics[2].as_str()?.trim_start_matches("0x");
        
        // Quick validation - optimized for speed (no string allocations)
        let prev_addr = if previous_owner.len() == 64 { &previous_owner[24..] } else { previous_owner };
        let new_addr = if new_owner.len() == 64 { &new_owner[24..] } else { new_owner };
        
        // Fast comparison - should already be filtered by WebSocket
        if prev_addr.chars().all(|c| c == '0') && 
           new_addr.eq_ignore_ascii_case(&TARGET_NEW_OWNER[2..]) {
            
            // Format addresses only when we have a match (lazy evaluation)
            let previous_owner_addr = format!("0x{}", prev_addr);
            let new_owner_addr = format!("0x{}", new_addr);
            
            info!("🚀 TOKEN DETECTED: {} in block {} (ownership {} -> {})", 
                  token_address, block_number, previous_owner_addr, new_owner_addr);
            
            Some(TokenResult {
                token: token_address.to_string(),
                block_number,
                transaction_hash: tx_hash.to_string(),
                previous_owner: previous_owner_addr,
                new_owner: new_owner_addr,
            })
        } else {
            None
        }
    }

    // Test token detection for a specific block - SPEED OPTIMIZED
    pub async fn test_block(&self, block_number: u64) -> Result<Vec<TokenResult>> {
        info!("🧪 Testing block {} for OwnershipTransferred events (SPEED MODE)...", block_number);
        
        let mut detected_tokens = Vec::new();
        
        let (ws_stream, _) = connect_async(&self.wss_url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        let block_hex = format!("0x{:x}", block_number);
        
        // WebSocket-level filtering for OwnershipTransferred events
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getLogs",
            "params": [{
                "topics": [
                    OWNERSHIP_TRANSFERRED_TOPIC,
                    format!("0x{:0>64}", ZERO_ADDRESS.trim_start_matches("0x")), // previousOwner = zero address
                    format!("0x{:0>64}", TARGET_NEW_OWNER.trim_start_matches("0x")) // newOwner = target address
                ],
                "fromBlock": block_hex,
                "toBlock": block_hex
            }],
            "id": 1
        });
        
        ws_sender.send(Message::Text(request.to_string())).await?;
        
        while let Some(msg) = ws_receiver.next().await {
            match msg? {
                Message::Text(text) => {
                    if let Ok(json) = serde_json::from_str::<Value>(&text) {
                        if let Some(result) = json.get("result") {
                            if let Some(logs) = result.as_array() {
                                info!("📊 Found {} OwnershipTransferred events in block {}", logs.len(), block_number);
                                
                                // Process all events immediately - no duplicate checking for speed
                                for log in logs {
                                    if let Some(token_result) = self.process_ownership_event(log) {
                                        detected_tokens.push(token_result);
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        
        if detected_tokens.is_empty() {
            info!("🔍 No matching OwnershipTransferred events in block {}", block_number);
        } else {
            info!("🎯 Detected {} tokens via OwnershipTransferred in block {}", detected_tokens.len(), block_number);
        }
        
        Ok(detected_tokens)
    }

    // Test token detection for a range of blocks - SPEED OPTIMIZED
    pub async fn test_block_range(&self, from_block: u64, to_block: u64) -> Result<Vec<TokenResult>> {
        info!("🧪 Testing block range {} to {} for OwnershipTransferred events (SPEED MODE)...", from_block, to_block);
        
        let mut all_detected_tokens = Vec::new();
        
        let (ws_stream, _) = connect_async(&self.wss_url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        let from_block_hex = format!("0x{:x}", from_block);
        let to_block_hex = format!("0x{:x}", to_block);
        
        // WebSocket-level filtering for OwnershipTransferred events
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getLogs",
            "params": [{
                "topics": [
                    OWNERSHIP_TRANSFERRED_TOPIC,
                    format!("0x{:0>64}", ZERO_ADDRESS.trim_start_matches("0x")), // previousOwner = zero address
                    format!("0x{:0>64}", TARGET_NEW_OWNER.trim_start_matches("0x")) // newOwner = target address
                ],
                "fromBlock": from_block_hex,
                "toBlock": to_block_hex
            }],
            "id": 1
        });
        
        ws_sender.send(Message::Text(request.to_string())).await?;
        
        while let Some(msg) = ws_receiver.next().await {
            match msg? {
                Message::Text(text) => {
                    if let Ok(json) = serde_json::from_str::<Value>(&text) {
                        if let Some(result) = json.get("result") {
                            if let Some(logs) = result.as_array() {
                                info!("📊 Found {} OwnershipTransferred events in range {} to {}", 
                                      logs.len(), from_block, to_block);
                                
                                // Process all events immediately - no duplicate checking for speed
                                for log in logs {
                                    if let Some(token_result) = self.process_ownership_event(log) {
                                        all_detected_tokens.push(token_result);
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        
        if all_detected_tokens.is_empty() {
            info!("🔍 No matching OwnershipTransferred events in range {} to {}", from_block, to_block);
        } else {
            info!("🎯 Detected {} tokens via OwnershipTransferred in range {} to {}", 
                  all_detected_tokens.len(), from_block, to_block);
        }
        
        Ok(all_detected_tokens)
    }

    // Live token monitoring using OwnershipTransferred events - MAXIMUM SPEED!
    pub async fn monitor_live(&self) -> Result<String> {
        info!("🔍 Starting live OwnershipTransferred monitoring (IMMEDIATE RETURN MODE)...");
        info!("⚡ WebSocket filtering: {} -> {}", ZERO_ADDRESS, TARGET_NEW_OWNER);
        
        let (ws_stream, _) = connect_async(&self.wss_url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        // Subscribe to OwnershipTransferred events with WebSocket filtering
        let subscription = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_subscribe",
            "params": [
                "logs",
                {
                    "topics": [
                        OWNERSHIP_TRANSFERRED_TOPIC,
                        format!("0x{:0>64}", ZERO_ADDRESS.trim_start_matches("0x")), // previousOwner = zero address
                        format!("0x{:0>64}", TARGET_NEW_OWNER.trim_start_matches("0x")) // newOwner = target address
                    ]
                }
            ]
        });
        
        ws_sender.send(Message::Text(subscription.to_string())).await?;
        info!("📤 WebSocket subscription active - awaiting first token...");
        
        let mut subscription_confirmed = false;
        
        // Simplified event loop - OPTIMIZED FOR IMMEDIATE RETURN
        while let Some(msg) = ws_receiver.next().await {
            match msg? {
                Message::Text(text) => {
                    if let Ok(json) = serde_json::from_str::<Value>(&text) {
                        // Handle subscription confirmation
                        if json.get("id").is_some() && json.get("result").is_some() {
                            subscription_confirmed = true;
                            info!("✅ Live monitoring active - ready for immediate detection");
                            continue;
                        }
                        
                        // Handle subscription errors
                        if let Some(error) = json.get("error") {
                            return Err(anyhow!("Subscription error: {}", error));
                        }
                        
                        // Wait for subscription confirmation
                        if !subscription_confirmed {
                            continue;
                        }
                        
                        // Process OwnershipTransferred events - IMMEDIATE DETECTION
                        if json.get("method").and_then(|m| m.as_str()) == Some("eth_subscription") {
                            if let Some(params) = json.get("params") {
                                if let Some(result) = params.get("result") {
                                    // IMMEDIATE TOKEN DETECTION - NO DUPLICATE CHECKING FOR SPEED
                                    if let Some(token_result) = self.process_ownership_event(result) {
                                        info!("🎯 RETURNING TOKEN: {}", token_result.token);
                                        // RETURN IMMEDIATELY - BREAK ALL LOOPS
                                        return Ok(token_result.token);
                                    }
                                }
                            }
                        }
                    }
                }
                Message::Close(_) => {
                    return Err(anyhow!("WebSocket connection closed"));
                }
                _ => {}
            }
        }
        
        // Should never reach here with proper WebSocket filtering
        Ok("No token detected".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load environment variables
    dotenv::dotenv().ok();
    
    info!("🚀 OPTIMIZED OwnershipTransferred Token Detector - TESTING MODE");
    info!("⚡ Maximum speed optimizations enabled!");
    
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        error!("❌ Usage:");
        error!("   {} <block_number>           - Test single block", args[0]);
        error!("   {} <from_block> <to_block>  - Test block range", args[0]);
        error!("   {} live                     - Live monitoring", args[0]);
        std::process::exit(1);
    }
    
    let detector = TokenDetector::new()?;
    
    match args[1].as_str() {
        "live" => {
            match detector.monitor_live().await {
                Ok(token) => {
                    println!("🎯 DETECTED TOKEN: {}", token);
                }
                Err(e) => {
                    error!("❌ Live monitoring failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            if args.len() == 2 {
                // Single block test
                let block_number: u64 = args[1].parse()
                    .map_err(|_| anyhow!("Invalid block number: {}", args[1]))?;
                
                match detector.test_block(block_number).await {
                    Ok(tokens) => {
                        if !tokens.is_empty() {
                            println!("🎯 DETECTED TOKENS VIA OWNERSHIPTRANSFERRED:");
                            for token in tokens {
                                println!("   {} (Block: {}, TX: {}, {} -> {})", 
                                         token.token, token.block_number, token.transaction_hash,
                                         token.previous_owner, token.new_owner);
                            }
                        } else {
                            println!("🔍 No OwnershipTransferred events found in block {}", block_number);
                        }
                    }
                    Err(e) => {
                        error!("❌ Test failed: {}", e);
                        std::process::exit(1);
                    }
                }
            } else if args.len() == 3 {
                // Block range test
                let from_block: u64 = args[1].parse()
                    .map_err(|_| anyhow!("Invalid from_block: {}", args[1]))?;
                let to_block: u64 = args[2].parse()
                    .map_err(|_| anyhow!("Invalid to_block: {}", args[2]))?;
                
                if from_block > to_block {
                    error!("❌ from_block cannot be greater than to_block");
                    std::process::exit(1);
                }
                
                match detector.test_block_range(from_block, to_block).await {
                    Ok(tokens) => {
                        if !tokens.is_empty() {
                            println!("🎯 DETECTED TOKENS VIA OWNERSHIPTRANSFERRED IN RANGE {} to {}:", from_block, to_block);
                            for token in tokens {
                                println!("   {} (Block: {}, TX: {}, {} -> {})", 
                                         token.token, token.block_number, token.transaction_hash,
                                         token.previous_owner, token.new_owner);
                            }
                        } else {
                            println!("🔍 No OwnershipTransferred events found in range {} to {}", from_block, to_block);
                        }
                    }
                    Err(e) => {
                        error!("❌ Test failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
    
    Ok(())
} 