use anyhow::{Result, anyhow};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::info;

// Configuration - OwnershipTransferred live detection
const OWNERSHIP_TRANSFERRED_TOPIC: &str = "0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0";
const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
const TARGET_NEW_OWNER: &str = "0xE220329659D41B2a9F26E83816B424bDAcF62567";

// Production detector - optimized for speed
pub struct TokenDetector {
    wss_url: String,
}

impl TokenDetector {
    pub fn new() -> Result<Self> {
        let wss_url = std::env::var("WSS_URL")
            .map_err(|_| anyhow!("WSS_URL environment variable not set"))?;
        
        info!("🚀 Fast OwnershipTransferred detector initialized");
        
        Ok(Self {
            wss_url,
        })
    }

    // Fast OwnershipTransferred event processing - MINIMAL LOGGING
    fn process_ownership_event(&self, log_data: &Value) -> Option<String> {
        let token_address = log_data["address"].as_str()?;
        let topics = log_data["topics"].as_array()?;
        
        if topics.len() < 3 {
            return None;
        }
        
        // Fast validation - no logging for maximum speed
        let previous_owner = topics[1].as_str()?.trim_start_matches("0x");
        let new_owner = topics[2].as_str()?.trim_start_matches("0x");
        
        let prev_addr = if previous_owner.len() == 64 { &previous_owner[24..] } else { previous_owner };
        let new_addr = if new_owner.len() == 64 { &new_owner[24..] } else { new_owner };
        
        if prev_addr.chars().all(|c| c == '0') && 
           new_addr.eq_ignore_ascii_case(&TARGET_NEW_OWNER[2..]) {
            // Only log on successful detection - critical info only
            Some(token_address.to_string())
        } else {
            None
        }
    }

    // Live token detection - MINIMAL LOGGING FOR SPEED
    pub async fn get_token_address<F, Fut>(&self, on_token_found: Option<F>) -> Result<String>
    where
        F: FnOnce(String) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        info!("🔍 Starting live detection...");
        
        let (ws_stream, _) = connect_async(&self.wss_url).await
            .map_err(|e| anyhow!("WebSocket connection failed: {}", e))?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        // Subscribe with WebSocket-level filtering
        let subscription = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_subscribe",
            "params": [
                "logs",
                {
                    "topics": [
                        OWNERSHIP_TRANSFERRED_TOPIC,
                        format!("0x{:0>64}", ZERO_ADDRESS.trim_start_matches("0x")),
                        format!("0x{:0>64}", TARGET_NEW_OWNER.trim_start_matches("0x"))
                    ]
                }
            ]
        });
        
        ws_sender.send(Message::Text(subscription.to_string())).await
            .map_err(|e| anyhow!("Subscription failed: {}", e))?;
        
        info!("✅ Monitoring active - awaiting token...");
        
        let mut callback_option = on_token_found;
        let mut subscription_confirmed = false;
        
        // Ultra-fast event loop - NO UNNECESSARY LOGGING
        while let Some(msg) = ws_receiver.next().await {
            match msg.map_err(|e| anyhow!("WebSocket error: {}", e))? {
                Message::Text(text) => {
                    if let Ok(json) = serde_json::from_str::<Value>(&text) {
                        // Handle subscription confirmation - no logging for speed
                        if json.get("id").is_some() && json.get("result").is_some() {
                            subscription_confirmed = true;
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
                        
                        // Process events - NO LOGGING FOR MAXIMUM SPEED
                        if json.get("method").and_then(|m| m.as_str()) == Some("eth_subscription") {
                            if let Some(params) = json.get("params") {
                                if let Some(result) = params.get("result") {
                                    // IMMEDIATE TOKEN DETECTION - NO LOGGING
                                    if let Some(token) = self.process_ownership_event(result) {
                                        // Execute callback in background - don't wait
                                        if let Some(callback) = callback_option.take() {
                                            let token_clone = token.clone();
                                            tokio::spawn(async move {
                                                let _ = callback(token_clone).await;
                                            });
                                        }
                                        
                                        // RETURN IMMEDIATELY - NO LOGGING FOR SPEED
                                        return Ok(token);
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
        
        Ok("No token detected".to_string())
    }
} 