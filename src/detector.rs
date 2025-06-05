use anyhow::{Result, anyhow};
use futures_util::{SinkExt, StreamExt};
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{info, error};

// Configuration (matching JS exactly)
const TARGET_TOPIC: &str = "0xf9d151d23a5253296eb20ab40959cf48828ea2732d337416716e302ed83ca658";
const DEPLOYER: &str = "0x71B8EFC8BCaD65a5D9386D07f2Dff57ab4EAf533";
const WANTED: &str = "0x81F7cA6AF86D1CA6335E44A2C28bC88807491415";
const UNWANTED: &str = "0x03Fb99ea8d3A832729a69C3e8273533b52f30D1A";

// Pre-compiled patterns (optimized for speed)
const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
const ZERO_HEX_PATTERN: &str = "000000000000000000000000";

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

// Global state (matching JS)
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
    wanted_lower: String,
    unwanted_lower: String,
}

impl TokenDetector {
    pub fn new() -> Result<Self> {
        // Load WSS_URL from environment (matching JS)
        let wss_url = std::env::var("WSS_URL")
            .map_err(|_| anyhow!("WSS_URL environment variable not set"))?;
        
        // Load USE_TX_VERIFICATION from environment (default: true)
        let use_tx_verification = std::env::var("USE_TX_VERIFICATION")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        info!("🔧 Transaction verification: {}", if use_tx_verification { "ENABLED" } else { "DISABLED" });
        
        // Pre-compiled regex (optimized for performance - matching JS addressRegex with /g behavior)
        let address_regex = Regex::new(r"000000000000000000000000([a-fA-F0-9]{40})")
            .map_err(|e| anyhow!("Failed to compile regex: {}", e))?;
        
        // Pre-computed hex values (matching JS, optimized)
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
            wanted_lower: WANTED.to_lowercase(),
            unwanted_lower: UNWANTED.to_lowercase(),
        })
    }

    // Extract token and determine caller in one pass (optimized for speed)
    fn extract_token_and_caller(&self, data: &str) -> Option<TokenResult> {
        if data.is_empty() || data.len() < 130 {
            return None;
        }
        
        // Pre-allocate with known capacity for speed
        let mut addresses = Vec::with_capacity(10);
        
        // Extract addresses using optimized regex (matching JS logic exactly)
        for cap in self.address_regex.captures_iter(data) {
            if addresses.len() >= 10 {
                break;
            }
            
            // Get the captured hex string directly (avoid format! allocation)
            if let Some(hex_match) = cap.get(1) {
                let hex_str = hex_match.as_str();
                
                // Quick zero check without string allocation
                if hex_str != "0000000000000000000000000000000000000000" {
                    // Only allocate string when we need it
                    let addr = format!("0x{}", hex_str);
                    addresses.push(addr);
                }
            }
        }
        
        if addresses.len() < 2 {
            return None;
        }
        
        // Token is at addresses[1] (matching JS exactly)
        let token = addresses[1].clone();
        
        // Optimized exact address checking (pre-computed lowercase)
        for addr in &addresses {
            let addr_lower = addr.to_lowercase();
            if addr_lower == self.wanted_lower {
                return Some(TokenResult {
                    token,
                    confidence: Confidence::Wanted,
                });
            }
            if addr_lower == self.unwanted_lower {
                return Some(TokenResult {
                    token,
                    confidence: Confidence::Unwanted,
                });
            }
        }
        
        // Pattern matching fallback (optimized - convert data to lowercase once)
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
    
    // Verify caller with caching (matching JS verifyCaller)
    async fn verify_caller(&self, tx_hash: &str) -> Result<bool> {
        // Check cache first (matching JS logic)
        {
            let cache = self.caller_cache.lock().await;
            if let Some(caller) = cache.get(tx_hash) {
                return Ok(caller.to_lowercase() == WANTED.to_lowercase());
            }
        }
        
        // Check rejected callers (matching JS logic)
        {
            let rejected = self.rejected_callers.lock().await;
            if rejected.contains(tx_hash) {
                return Ok(false);
            }
        }
        
        // Get transaction via WebSocket (matching JS getTransaction)
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
                                // Transaction not found, reject
                                let mut rejected = self.rejected_callers.lock().await;
                                rejected.insert(tx_hash.to_string());
                                return Ok(false);
                            }
                            
                            if let Some(from_addr) = result["from"].as_str() {
                                // Cache the result (matching JS logic)
                                {
                                    let mut cache = self.caller_cache.lock().await;
                                    cache.insert(tx_hash.to_string(), from_addr.to_string());
                                }
                                
                                let is_wanted = from_addr.to_lowercase() == WANTED.to_lowercase();
                                
                                // Cache rejection if not wanted (matching JS logic)
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
        
        // Network error, reject (matching JS catch block)
        let mut rejected = self.rejected_callers.lock().await;
        rejected.insert(tx_hash.to_string());
        Ok(false)
    }
    
    // Process events (matching JS processEvent) - Returns token if found
    async fn process_event<F, Fut>(&self, log_data: &Value, callback: Option<F>) -> Result<Option<String>>
    where
        F: FnOnce(String) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        let tx_hash = log_data["transactionHash"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing transaction hash"))?;
        
        // Check if already processed (matching JS logic)
        {
            let mut processed = self.processed_txs.lock().await;
            if processed.contains(tx_hash) {
                return Ok(None);
            }
            
            // Simple cache management (matching JS logic exactly)
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
        
        // Handle based on confidence (matching JS logic exactly)
        match result.confidence {
            Confidence::Wanted => {
                info!("🚀 DETECTED: {}", result.token);
                // Set stop flag immediately (no need to wait for callback)
                {
                    let mut should_stop = self.should_stop.lock().await;
                    *should_stop = true;
                }
                
                // Execute callback in background if provided, but return token immediately
                if let Some(cb) = callback {
                    let token_clone = result.token.clone();
                    tokio::spawn(async move {
                        info!("🔄 Callback triggered for: {}", token_clone);
                        info!("⚡ Executing onTokenFound callback...");
                        match cb(token_clone.clone()).await {
                            Ok(_) => {
                                info!("✅ Callback completed successfully");
                            }
                            Err(e) => {
                                error!("❌ Callback execution failed: {}", e);
                            }
                        }
                    });
                }
                // Return immediately without waiting for callback
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
                            // Set stop flag immediately
                            {
                                let mut should_stop = self.should_stop.lock().await;
                                *should_stop = true;
                            }
                            
                            // Execute callback in background if provided, but return token immediately
                            if let Some(cb) = callback {
                                let token_clone = result.token.clone();
                                tokio::spawn(async move {
                                    info!("🔄 Verification callback triggered for: {}", token_clone);
                                    info!("⚡ Executing onTokenFound callback...");
                                    match cb(token_clone.clone()).await {
                                        Ok(_) => {
                                            info!("✅ Verification callback completed successfully");
                                        }
                                        Err(e) => {
                                            error!("❌ Verification callback failed: {}", e);
                                        }
                                    }
                                });
                            }
                            // Return immediately without waiting for callback
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
                    // Set stop flag immediately
                    {
                        let mut should_stop = self.should_stop.lock().await;
                        *should_stop = true;
                    }
                    
                    // Execute callback in background if provided, but return token immediately
                    if let Some(cb) = callback {
                        let token_clone = result.token.clone();
                        tokio::spawn(async move {
                            info!("🔄 Trust mode callback triggered for: {}", token_clone);
                            info!("⚡ Executing onTokenFound callback...");
                            match cb(token_clone.clone()).await {
                                Ok(_) => {
                                    info!("✅ Trust mode callback completed successfully");
                                }
                                Err(e) => {
                                    error!("❌ Trust mode callback failed: {}", e);
                                }
                            }
                        });
                    }
                    // Return immediately without waiting for callback
                    return Ok(Some(result.token));
                }
            }
        }
        
        Ok(None)
    }
    
    // Main function - Live token detection (matching JS getTokenAddress)
    pub async fn get_token_address<F, Fut>(&self, on_token_found: Option<F>) -> Result<String>
    where
        F: FnOnce(String) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        // Reset state (matching JS logic)
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
        
        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&self.wss_url).await
            .map_err(|e| anyhow!("Failed to connect to WebSocket: {}", e))?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        // Subscribe to logs
        let subscription = serde_json::json!({
            "jsonrpc": "2.0",
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
        
        ws_sender.send(Message::Text(subscription.to_string())).await
            .map_err(|e| anyhow!("Failed to send subscription: {}", e))?;
        
        info!("📤 Sent WebSocket subscription request");
        
        let mut callback_option = on_token_found;
        let mut subscription_confirmed = false;
        
        // Process incoming messages
        while let Some(msg) = ws_receiver.next().await {
            // Check if we should stop
            {
                let should_stop = self.should_stop.lock().await;
                if *should_stop {
                    break;
                }
            }
            
            match msg.map_err(|e| anyhow!("WebSocket error: {}", e))? {
                Message::Text(text) => {
                    info!("📥 Received WebSocket message: {}", text);
                    
                    if let Ok(json) = serde_json::from_str::<Value>(&text) {
                        // Handle subscription confirmation
                        if json.get("id").is_some() && json.get("result").is_some() {
                            if let Some(sub_id) = json["result"].as_str() {
                                subscription_confirmed = true;
                                info!("✅ WebSocket subscription established: {}", sub_id);
                                continue;
                            }
                        }
                        
                        // Handle subscription errors
                        if let Some(error) = json.get("error") {
                            error!("❌ Subscription failed: {}", error);
                            return Err(anyhow!("Subscription error: {}", error));
                        }
                        
                        // Only process events after subscription is confirmed
                        if !subscription_confirmed {
                            info!("⏳ Waiting for subscription confirmation...");
                            continue;
                        }
                        
                        // Handle subscription events (matching JS format)
                        if json.get("method").and_then(|m| m.as_str()) == Some("eth_subscription") {
                            if let Some(params) = json.get("params") {
                                if let Some(result) = params.get("result") {
                                    info!("🔍 Processing subscription event");
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
                    } else {
                        error!("❌ Failed to parse WebSocket message as JSON: {}", text);
                    }
                }
                Message::Close(_) => {
                    info!("🔌 WebSocket connection closed");
                    return Err(anyhow!("WebSocket connection closed"));
                }
                _ => {}
            }
        }
        
        Ok("No token detected".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load environment variables (matching JS require('dotenv').config())
    dotenv::dotenv().ok();
    
    info!("🤖 Rust Sniper Bot - Starting...");
    
    // Create detector (matching JS global state initialization)
    let detector = TokenDetector::new()?;
    
    // CLI - Live detection only (matching JS if (require.main === module))
    match detector.get_token_address(None::<fn(String) -> futures_util::future::Ready<Result<()>>>).await {
        Ok(result) => {
            println!("{}", result);
            Ok(())
        }
        Err(e) => {
            error!("Detection failed: {}", e);
            std::process::exit(1);
        }
    }
}
