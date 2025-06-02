use anyhow::{Result, anyhow};
use futures_util::{SinkExt, StreamExt};
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{info, error};

// Configuration (matching JS exactly) - CORE LOGIC UNCHANGED
const TARGET_TOPIC: &str = "0xf9d151d23a5253296eb20ab40959cf48828ea2732d337416716e302ed83ca658";
const DEPLOYER: &str = "0x71B8EFC8BCaD65a5D9386D07f2Dff57ab4EAf533";
const WANTED: &str = "0x81F7cA6AF86D1CA6335E44A2C28bC88807491415";
const UNWANTED: &str = "0x03Fb99ea8d3A832729a69C3e8273533b52f30D1A";

// Pre-compiled patterns (matching JS) - CORE LOGIC UNCHANGED
const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

#[derive(Debug, Clone)]
enum Confidence {
    Wanted,
    Unwanted,
    Verify,
}

#[derive(Debug, Clone)]
struct TokenResult {
    token: String,
    confidence: Confidence,
}

// Global state (matching JS) - CORE LOGIC UNCHANGED
pub struct TokenDetector {
    wss_url: String,
    use_tx_verification: bool,
    should_stop: Arc<Mutex<bool>>,
    processed_txs: Arc<Mutex<HashSet<String>>>,
    caller_cache: Arc<Mutex<HashMap<String, String>>>,
    rejected_callers: Arc<Mutex<HashSet<String>>>,
    address_regex: Regex,
    wanted_hex: String,
    unwanted_hex: String,
}

impl TokenDetector {
    pub fn new() -> Result<Self> {
        // Load WSS_URL from environment (matching JS) - CORE LOGIC UNCHANGED
        let wss_url = std::env::var("WSS_URL")
            .map_err(|_| anyhow!("WSS_URL environment variable not set"))?;
        
        // Load USE_TX_VERIFICATION from environment (default: true)
        let use_tx_verification = std::env::var("USE_TX_VERIFICATION")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        info!("🔧 Transaction verification: {}", if use_tx_verification { "ENABLED" } else { "DISABLED" });
        
        // Pre-compiled regex (matching JS addressRegex) - CORE LOGIC UNCHANGED
        let address_regex = Regex::new(r"000000000000000000000000([a-fA-F0-9]{40})")
            .map_err(|e| anyhow!("Failed to compile regex: {}", e))?;
        
        // Pre-computed hex values (matching JS) - CORE LOGIC UNCHANGED
        let wanted_hex = WANTED[2..].to_lowercase(); // Remove 0x prefix
        let unwanted_hex = UNWANTED[2..].to_lowercase(); // Remove 0x prefix
        
        Ok(Self {
            wss_url,
            use_tx_verification,
            should_stop: Arc::new(Mutex::new(false)),
            processed_txs: Arc::new(Mutex::new(HashSet::new())),
            caller_cache: Arc::new(Mutex::new(HashMap::new())),
            rejected_callers: Arc::new(Mutex::new(HashSet::new())),
            address_regex,
            wanted_hex,
            unwanted_hex,
        })
    }

    // Extract token and determine caller in one pass (matching JS extractTokenAndCaller) - CORE LOGIC UNCHANGED
    fn extract_token_and_caller(&self, data: &str) -> Option<TokenResult> {
        if data.is_empty() || data.len() < 130 {
            return None;
        }
        
        let mut addresses = Vec::new();
        
        // Extract addresses using regex (matching JS logic exactly) - CORE LOGIC UNCHANGED
        for cap in self.address_regex.captures_iter(data) {
            if addresses.len() >= 10 {
                break;
            }
            let addr = format!("0x{}", &cap[1]);
            if addr != ZERO_ADDRESS {
                addresses.push(addr);
            }
        }
        
        if addresses.len() < 2 {
            return None;
        }
        
        // Token is at addresses[1] (matching JS exactly) - CORE LOGIC UNCHANGED
        let token = addresses[1].clone();
        
        // Check exact addresses first (matching JS logic) - CORE LOGIC UNCHANGED
        for addr in &addresses {
            if addr.to_lowercase() == WANTED.to_lowercase() {
                return Some(TokenResult {
                    token,
                    confidence: Confidence::Wanted,
                });
            }
            if addr.to_lowercase() == UNWANTED.to_lowercase() {
                return Some(TokenResult {
                    token,
                    confidence: Confidence::Unwanted,
                });
            }
        }
        
        // Pattern matching fallback (matching JS logic) - CORE LOGIC UNCHANGED
        let data_lower = data.to_lowercase();
        if data_lower.contains(&self.unwanted_hex) {
            return Some(TokenResult {
                token,
                confidence: Confidence::Unwanted,
            });
        }
        if data_lower.contains(&self.wanted_hex) {
            return Some(TokenResult {
                token,
                confidence: Confidence::Wanted,
            });
        }
        
        Some(TokenResult {
            token,
            confidence: Confidence::Verify,
        })
    }
    
    // Verify caller with caching (matching JS verifyCaller) - CORE LOGIC UNCHANGED
    async fn verify_caller(&self, tx_hash: &str) -> Result<bool> {
        // Check cache first (matching JS logic) - CORE LOGIC UNCHANGED
        {
            let cache = self.caller_cache.lock().await;
            if let Some(caller) = cache.get(tx_hash) {
                return Ok(caller.to_lowercase() == WANTED.to_lowercase());
            }
        }
        
        // Check rejected callers (matching JS logic) - CORE LOGIC UNCHANGED
        {
            let rejected = self.rejected_callers.lock().await;
            if rejected.contains(tx_hash) {
                return Ok(false);
            }
        }
        
        // Get transaction via WebSocket (matching JS getTransaction) - CORE LOGIC UNCHANGED
        let (ws_stream, _) = connect_async(&self.wss_url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionByHash",
            "params": [tx_hash],
            "id": 1
        });
        
        ws_sender.send(Message::Text(request.to_string())).await?;
        
        while let Some(msg) = ws_receiver.next().await {
            match msg? {
                Message::Text(text) => {
                    if let Ok(json) = serde_json::from_str::<Value>(&text) {
                        if let Some(result) = json.get("result") {
                            if result.is_null() {
                                // Transaction not found, reject - CORE LOGIC UNCHANGED
                                let mut rejected = self.rejected_callers.lock().await;
                                rejected.insert(tx_hash.to_string());
                                return Ok(false);
                            }
                            
                            if let Some(from_addr) = result["from"].as_str() {
                                // Cache the result (matching JS logic) - CORE LOGIC UNCHANGED
                                {
                                    let mut cache = self.caller_cache.lock().await;
                                    cache.insert(tx_hash.to_string(), from_addr.to_string());
                                }
                                
                                let is_wanted = from_addr.to_lowercase() == WANTED.to_lowercase();
                                
                                // Cache rejection if not wanted (matching JS logic) - CORE LOGIC UNCHANGED
                                if !is_wanted {
                                    let mut rejected = self.rejected_callers.lock().await;
                                    rejected.insert(tx_hash.to_string());
                                }
                                
                                return Ok(is_wanted);
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        
        // Network error, reject (matching JS catch block) - CORE LOGIC UNCHANGED
        let mut rejected = self.rejected_callers.lock().await;
        rejected.insert(tx_hash.to_string());
        Ok(false)
    }
    
    // Process events (matching JS processEvent) - Returns token if found - CORE LOGIC UNCHANGED
    async fn process_event<F, Fut>(&self, log_data: &Value, callback: Option<F>) -> Result<Option<String>>
    where
        F: FnOnce(String) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        let tx_hash = log_data["transactionHash"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing transaction hash"))?;
        
        // Check if already processed (matching JS logic) - CORE LOGIC UNCHANGED
        {
            let mut processed = self.processed_txs.lock().await;
            if processed.contains(tx_hash) {
                return Ok(None);
            }
            
            // Simple cache management (matching JS logic exactly) - CORE LOGIC UNCHANGED
            if processed.len() >= 1000 {
                processed.clear();
                let mut cache = self.caller_cache.lock().await;
                if cache.len() >= 500 {
                    cache.clear();
                }
                let mut rejected = self.rejected_callers.lock().await;
                if rejected.len() >= 100 {
                    rejected.clear();
                }
            }
            
            processed.insert(tx_hash.to_string());
        }
        
        let data = log_data["data"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing log data"))?;
        
        let result = match self.extract_token_and_caller(data) {
            Some(result) => result,
            None => return Ok(None),
        };
        
        // Handle based on confidence (matching JS logic exactly) - CORE LOGIC UNCHANGED
        match result.confidence {
            Confidence::Wanted => {
                info!("🚀 DETECTED: {}", result.token);
                // Execute callback if provided
                if let Some(cb) = callback {
                    if let Err(e) = cb(result.token.clone()).await {
                        error!("❌ Callback execution failed: {}", e);
                    }
                }
                return Ok(Some(result.token));
            }
            Confidence::Unwanted => {
                info!("❌ UNWANTED: {} from {} - continuing monitoring...", result.token, UNWANTED);
            }
            Confidence::Verify => {
                if self.use_tx_verification {
                    match self.verify_caller(tx_hash).await {
                        Ok(true) => {
                            info!("🚀 DETECTED: {}", result.token);
                            // Execute callback if provided
                            if let Some(cb) = callback {
                                if let Err(e) = cb(result.token.clone()).await {
                                    error!("❌ Verification callback failed: {}", e);
                                }
                            }
                            return Ok(Some(result.token));
                        }
                        Ok(false) => {
                            info!("❌ REJECTED: {} (wrong caller) - continuing monitoring...", result.token);
                        }
                        Err(_) => {
                            info!("❌ VERIFY ERROR: {} (network issue) - continuing monitoring...", result.token);
                        }
                    }
                } else {
                    info!("🚀 DETECTED: {}", result.token);
                    // Execute callback if provided
                    if let Some(cb) = callback {
                        if let Err(e) = cb(result.token.clone()).await {
                            error!("❌ Unverified mode callback failed: {}", e);
                        }
                    }
                    return Ok(Some(result.token));
                }
            }
        }
        
        Ok(None)
    }
    
    // Main function - Live token detection (matching JS getTokenAddress) - CORE LOGIC UNCHANGED
    pub async fn get_token_address<F, Fut>(&self, on_token_found: Option<F>) -> Result<String>
    where
        F: FnOnce(String) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        // Reset state (matching JS logic) - CORE LOGIC UNCHANGED
        {
            let mut should_stop = self.should_stop.lock().await;
            *should_stop = false;
        }
        {
            let mut processed = self.processed_txs.lock().await;
            processed.clear();
        }
        
        info!("🔍 Monitoring for tokens from: {}", WANTED);
        info!("❌ Will reject tokens from: {}", UNWANTED);
        
        // Connect to WebSocket (matching JS getProvider().on) - CORE LOGIC UNCHANGED
        let (ws_stream, _) = connect_async(&self.wss_url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        // Subscribe to logs (matching JS event listener) - CORE LOGIC UNCHANGED
        let subscription = serde_json::json!({
            "id": 1,
            "method": "eth_subscribe",
            "params": [
                "logs",
                {
                    "address": DEPLOYER,
                    "topics": [TARGET_TOPIC]
                }
            ]
        });
        
        ws_sender.send(Message::Text(subscription.to_string())).await?;
        
        let mut callback_option = on_token_found;
        
        // Process incoming messages (matching JS event handling) - CORE LOGIC UNCHANGED
        while let Some(msg) = ws_receiver.next().await {
            match msg? {
                Message::Text(text) => {
                    if let Ok(json) = serde_json::from_str::<Value>(&text) {
                        if let Some(params) = json.get("params") {
                            if let Some(result) = params.get("result") {
                                // Process event and get token immediately if found
                                if let Some(callback) = callback_option.take() {
                                    if let Ok(Some(token)) = self.process_event(result, Some(callback)).await {
                                        info!("🎯 Returning detected token immediately: {}", token);
                                        return Ok(token);
                                    }
                                } else {
                                    // No callback, just return first detected token
                                    if let Ok(Some(token)) = self.process_event(result, None::<fn(String) -> futures_util::future::Ready<Result<()>>>).await {
                                        info!("🎯 Returning detected token immediately: {}", token);
                                        return Ok(token);
                                    }
                                }
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        
        Ok("No token detected".to_string())
    }

    // ============================================================================
    // TESTING FUNCTIONALITY - NEW ADDITION (CORE LOGIC UNCHANGED)
    // ============================================================================

    // Test token detection for a specific block
    pub async fn test_block(&self, block_number: u64) -> Result<Vec<String>> {
        info!("🧪 Testing block {} for token deployments...", block_number);
        
        let mut detected_tokens = Vec::new();
        
        // Get logs from specific block using eth_getLogs
        let (ws_stream, _) = connect_async(&self.wss_url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        let block_hex = format!("0x{:x}", block_number);
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getLogs",
            "params": [{
                "address": DEPLOYER,
                "topics": [TARGET_TOPIC],
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
                                info!("📊 Found {} logs in block {}", logs.len(), block_number);
                                
                                for log in logs {
                                    if let Some(data) = log["data"].as_str() {
                                        if let Some(token_result) = self.extract_token_and_caller(data) {
                                            info!("🔍 Found token: {} (confidence: {:?})", token_result.token, token_result.confidence);
                                            
                                            // Use same logic as live detection for consistency
                                            match token_result.confidence {
                                                Confidence::Wanted => {
                                                    info!("✅ WANTED token detected: {}", token_result.token);
                                                    detected_tokens.push(token_result.token);
                                                }
                                                Confidence::Unwanted => {
                                                    info!("❌ UNWANTED token detected: {}", token_result.token);
                                                }
                                                Confidence::Verify => {
                                                    if self.use_tx_verification {
                                                        if let Some(tx_hash) = log["transactionHash"].as_str() {
                                                            match self.verify_caller(tx_hash).await {
                                                                Ok(true) => {
                                                                    info!("✅ VERIFIED token detected: {}", token_result.token);
                                                                    detected_tokens.push(token_result.token);
                                                                }
                                                                Ok(false) => {
                                                                    info!("❌ REJECTED token (wrong caller): {}", token_result.token);
                                                                }
                                                                Err(_) => {
                                                                    info!("⚠️ VERIFY ERROR for token: {}", token_result.token);
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        info!("✅ UNVERIFIED token detected: {}", token_result.token);
                                                        detected_tokens.push(token_result.token);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                break; // Exit after processing the response
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        
        if detected_tokens.is_empty() {
            info!("🔍 No matching tokens found in block {}", block_number);
        } else {
            info!("🎯 Detected {} matching tokens in block {}", detected_tokens.len(), block_number);
        }
        
        Ok(detected_tokens)
    }

    // Test token detection for a range of blocks
    pub async fn test_block_range(&self, from_block: u64, to_block: u64) -> Result<Vec<String>> {
        info!("🧪 Testing block range {} to {} for token deployments...", from_block, to_block);
        
        let mut all_detected_tokens = Vec::new();
        
        // Get logs from block range using eth_getLogs
        let (ws_stream, _) = connect_async(&self.wss_url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        let from_block_hex = format!("0x{:x}", from_block);
        let to_block_hex = format!("0x{:x}", to_block);
        
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getLogs",
            "params": [{
                "address": DEPLOYER,
                "topics": [TARGET_TOPIC],
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
                                info!("📊 Found {} logs in block range {} to {}", logs.len(), from_block, to_block);
                                
                                for log in logs {
                                    let block_num = log["blockNumber"].as_str()
                                        .and_then(|s| u64::from_str_radix(&s[2..], 16).ok())
                                        .unwrap_or(0);
                                    
                                    if let Some(data) = log["data"].as_str() {
                                        if let Some(token_result) = self.extract_token_and_caller(data) {
                                            info!("🔍 Block {}: Found token: {} (confidence: {:?})", 
                                                  block_num, token_result.token, token_result.confidence);
                                            
                                            // Use same logic as live detection for consistency
                                            match token_result.confidence {
                                                Confidence::Wanted => {
                                                    info!("✅ Block {}: WANTED token detected: {}", block_num, token_result.token);
                                                    all_detected_tokens.push(token_result.token);
                                                }
                                                Confidence::Unwanted => {
                                                    info!("❌ Block {}: UNWANTED token detected: {}", block_num, token_result.token);
                                                }
                                                Confidence::Verify => {
                                                    if self.use_tx_verification {
                                                        if let Some(tx_hash) = log["transactionHash"].as_str() {
                                                            match self.verify_caller(tx_hash).await {
                                                                Ok(true) => {
                                                                    info!("✅ Block {}: VERIFIED token detected: {}", block_num, token_result.token);
                                                                    all_detected_tokens.push(token_result.token);
                                                                }
                                                                Ok(false) => {
                                                                    info!("❌ Block {}: REJECTED token (wrong caller): {}", block_num, token_result.token);
                                                                }
                                                                Err(_) => {
                                                                    info!("⚠️ Block {}: VERIFY ERROR for token: {}", block_num, token_result.token);
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        info!("✅ Block {}: UNVERIFIED token detected: {}", block_num, token_result.token);
                                                        all_detected_tokens.push(token_result.token);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                break; // Exit after processing the response
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        
        if all_detected_tokens.is_empty() {
            info!("🔍 No matching tokens found in block range {} to {}", from_block, to_block);
        } else {
            info!("🎯 Detected {} matching tokens in block range {} to {}", 
                  all_detected_tokens.len(), from_block, to_block);
        }
        
        Ok(all_detected_tokens)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load environment variables (matching JS require('dotenv').config())
    dotenv::dotenv().ok();
    
    info!("🧪 Rust Sniper Bot - TEST MODE");
    
    // Parse command line arguments for testing
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        error!("❌ Usage: {} <block_number> OR <from_block> <to_block>", args[0]);
        error!("   Example: {} 12345678", args[0]);
        error!("   Example: {} 12345678 12345680", args[0]);
        std::process::exit(1);
    }
    
    // Create detector (using same initialization as detector.rs)
    let detector = TokenDetector::new()?;
    
    if args.len() == 2 {
        // Single block test
        let block_number: u64 = args[1].parse()
            .map_err(|_| anyhow!("Invalid block number: {}", args[1]))?;
        
        match detector.test_block(block_number).await {
            Ok(tokens) => {
                if !tokens.is_empty() {
                    println!("🎯 DETECTED TOKENS:");
                    for token in tokens {
                        println!("   {}", token);
                    }
                } else {
                    println!("🔍 No matching tokens found in block {}", block_number);
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
            error!("❌ from_block ({}) cannot be greater than to_block ({})", from_block, to_block);
            std::process::exit(1);
        }
        
        match detector.test_block_range(from_block, to_block).await {
            Ok(tokens) => {
                if !tokens.is_empty() {
                    println!("🎯 DETECTED TOKENS IN RANGE {} to {}:", from_block, to_block);
                    for token in tokens {
                        println!("   {}", token);
                    }
                } else {
                    println!("🔍 No matching tokens found in block range {} to {}", from_block, to_block);
                }
            }
            Err(e) => {
                error!("❌ Test failed: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        error!("❌ Too many arguments. Usage: {} <block_number> OR <from_block> <to_block>", args[0]);
        std::process::exit(1);
    }
    
    Ok(())
} 