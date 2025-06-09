use anyhow::Result;
use ethers::prelude::*;
use std::sync::Arc;
use std::env;
use tracing::{info, error};

// Import modules
mod detector;
mod uniswap;

use detector::TokenDetector;
use uniswap::{UniswapTrader, GasConfig, get_deadline_from_now};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load environment variables
    dotenv::dotenv().ok();
    
    let private_key = env::var("PRIVATE_KEY")
        .expect("PRIVATE_KEY environment variable not set");
    let wss_url = env::var("WSS_URL")
        .expect("WSS_URL environment variable not set");
    
    info!("🚀 Starting live token detection and auto-swap system");
    
    // 🚀 PRE-INITIALIZE SWAP CONNECTION (for maximum speed)
    info!("⚡ Pre-initializing swap WebSocket connection...");
    let swap_provider = Provider::<Ws>::connect(&wss_url).await?;
    let wallet: LocalWallet = private_key.parse()?;
    
    // Set the correct chain ID for Base network (8453)
    let wallet = wallet.with_chain_id(8453u64);
    
    let swap_client = SignerMiddleware::new(swap_provider, wallet);
    let swap_client = Arc::new(swap_client);
    let recipient = swap_client.address();
    
    // 🚀 PRE-INITIALIZE UNISWAP TRADER (ready for instant swap)
    let trader = Arc::new(UniswapTrader::new(swap_client.clone())?);
    info!("✅ Swap connection pre-initialized and ready");
    
    // 🔍 INITIALIZE DETECTOR WITH SEPARATE CONNECTION (no interference)
    let detector = TokenDetector::new()?;
    info!("✅ Token detector initialized with separate connection");
    
    info!("🔴 LIVE DETECTION MODE - Pre-initialized for instant swapping...");
    
    // ⚡ ULTRA-FAST CALLBACK with pre-initialized trader
    let trader_clone = trader.clone();
    let callback = move |token_address: String| {
        let trader = trader_clone.clone();
        async move {
            info!("🎯 TOKEN DETECTED: {} - Executing INSTANT swap", token_address);
            
            // INSTANT swap execution with pre-initialized connection
            match execute_swap(&trader, &token_address, recipient).await {
                Ok(_) => {
                    info!("✅ Swap execution completed for token: {}", token_address);
                    Ok(())
                }
                Err(e) => {
                    error!("❌ Swap failed for token {}: {}", token_address, e);
                    Err(e)
                }
            }
        }
    };
    
    // 🚀 START DETECTION with pre-initialized swap infrastructure
    match detector.get_token_address(Some(callback)).await {
        Ok(token) => {
            if token != "No token detected" {
                info!("✅ Live detection completed - Token: {}", token);
            } else {
                info!("❌ Live detection ended without finding tokens");
            }
        }
        Err(e) => {
            error!("❌ Live detection failed: {}", e);
        }
    }
    
    Ok(())
}

// ⚡ ULTRA-FAST SWAP with pre-initialized connection
async fn execute_swap<M: Middleware + 'static>(
    trader: &UniswapTrader<M>,
    token_address: &str,
    recipient: Address
) -> Result<()> {
    let start_time = std::time::Instant::now();
    
    // Parse token address
    let token_out: Address = token_address.parse()?;
    
    // Configuration - Using VIRTUALS token as input
    let virtuals_address: Address = "0x0b3e328455c4059eeb9e3f84b5543f74e24e7e1b".parse()?; // VIRTUALS token on Base
    let amount_in = U256::from(10_000_000_000_000_000_000u64); // 10 VIRTUALS (18 decimals)
    let path = vec![virtuals_address, token_out];
    let deadline = get_deadline_from_now(300); // 5 minutes
    
    // Minimum amount out (allowing for slippage)
    let amount_out_min = U256::from(1); // Accept any amount of output tokens
    
    // ⚡ INSTANT SWAP EXECUTION (connection already established)
    let receipt = trader.swap_exact_tokens_for_tokens(
        amount_in,
        amount_out_min,
        path,
        recipient,
        deadline,
        Some(GasConfig::default())
    ).await?;
    
    let execution_time = start_time.elapsed();
    
    // Log detailed transaction information after swap is sent
    info!("🎯 SWAP SENT! Hash: {}", receipt.transaction_hash);
    info!("⚡ Execution Time: {:?}", execution_time);
    info!("⛽ Gas Used: {}", receipt.gas_used.unwrap_or_default());
    info!("🎯 Block: {}", receipt.block_number.unwrap_or_default());
    info!("💰 Token: {}", token_address);
    info!("🔗 Explorer: https://basescan.org/tx/{}", receipt.transaction_hash);
    
    Ok(())
} 