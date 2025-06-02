use anyhow::Result;
use ethers::prelude::*;
use std::sync::Arc;
use std::env;
use tracing::{info, error};

// Import modules
mod testDetector;
mod uniswap;

use testDetector::TokenDetector;
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
    
    info!("🚀 Starting token detection and auto-swap system");
    
    // Setup wallet and provider for swapping using WebSocket (faster for sniping)
    let provider = Provider::<Ws>::connect(&wss_url).await?;
    let wallet: LocalWallet = private_key.parse()?;
    
    // Set the correct chain ID for Base network (8453)
    let wallet = wallet.with_chain_id(8453u64);
    
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);
    let recipient = client.address();
    
    // Create Uniswap trader
    let trader = UniswapTrader::new(client.clone())?;
    info!("✅ Uniswap trader initialized (WebSocket)");
    
    // Create token detector
    let detector = TokenDetector::new()?;
    info!("✅ Token detector initialized");
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() == 1 {
        // Live detection mode - no arguments provided
        info!("🔴 LIVE DETECTION MODE - Waiting for real-time token deployments...");
        
        // Create a callback that immediately executes swap when token is found
        let trader_clone = Arc::new(trader);
        let callback = move |token_address: String| {
            let trader = trader_clone.clone();
            async move {
                info!("🎯 TOKEN DETECTED: {} - Executing immediate swap", token_address);
                
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
        
        // Start live detection with immediate swap callback
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
        
    } else if args.len() == 3 {
        // Historical testing mode - block range provided
        let from_block: u64 = args[1].parse()
            .expect("Invalid from_block number");
        let to_block: u64 = args[2].parse()
            .expect("Invalid to_block number");
        
        info!("🧪 HISTORICAL TEST MODE - Testing block range: {} to {}", from_block, to_block);
        
        // Test the block range and execute swaps for detected tokens
        match detector.test_block_range(from_block, to_block).await {
            Ok(detected_tokens) => {
                if detected_tokens.is_empty() {
                    info!("❌ No tokens detected in range");
                    return Ok(());
                }
                
                info!("🎯 Detected {} token(s), executing swaps...", detected_tokens.len());
                
                for token_address in detected_tokens {
                    match execute_swap(&trader, &token_address, recipient).await {
                        Ok(_) => info!("✅ Swap completed for token: {}", token_address),
                        Err(e) => error!("❌ Swap failed for token {}: {}", token_address, e),
                    }
                }
            }
            Err(e) => {
                error!("❌ Block range test failed: {}", e);
            }
        }
        
    } else {
        eprintln!("Usage:");
        eprintln!("  {} - Start live detection mode", args[0]);
        eprintln!("  {} <from_block> <to_block> - Test historical block range", args[0]);
        std::process::exit(1);
    }
    
    Ok(())
}

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
    let amount_in = U256::from(1_000_000_000_000_000u64); // 0.001 VIRTUALS (18 decimals)
    let path = vec![virtuals_address, token_out];
    let deadline = get_deadline_from_now(300); // 5 minutes
    
    // Minimum amount out (allowing for slippage)
    let amount_out_min = U256::from(1); // Accept any amount of output tokens
    
    // Execute swap immediately - NO LOGS BEFORE THIS POINT
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