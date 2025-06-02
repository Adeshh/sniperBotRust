use anyhow::Result;
use ethers::prelude::*;
use std::sync::Arc;
use tracing::info;

// Import our modules
use crate::uniswap::{UniswapTrader, GasConfig, quick_buy_token, parse_ether, get_deadline_from_now, quick_swap_tokens, quick_swap_tokens_with_approval};

// Example integration with token detection
pub async fn execute_trade_on_detection(
    token_address: &str,
    private_key: &str,
    rpc_url: &str,
    eth_amount: &str,
    slippage: f64
) -> Result<()> {
    info!("🚀 Executing trade for detected token: {}", token_address);
    
    // Setup wallet and provider
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let wallet: LocalWallet = private_key.parse()?;
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);
    
    // Create Uniswap trader
    let trader = UniswapTrader::new(client.clone())?;
    
    // Parse addresses
    let token_addr: Address = token_address.parse()?;
    let recipient = client.address();
    
    // Execute quick buy
    let receipt = quick_buy_token(
        &trader,
        token_addr,
        eth_amount,
        slippage,
        recipient
    ).await?;
    
    info!("✅ Trade executed successfully! Tx: {:?}", receipt.transaction_hash);
    Ok(())
}

// Example of manual swap with custom gas
pub async fn manual_swap_example(
    token_address: &str,
    private_key: &str, 
    rpc_url: &str
) -> Result<()> {
    // Setup
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let wallet: LocalWallet = private_key.parse()?;
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);
    let trader = UniswapTrader::new(client.clone())?;
    
    // Addresses
    let token_addr: Address = token_address.parse()?;
    let recipient = client.address();
    
    // Amount and timing
    let eth_amount = parse_ether("0.1")?; // 0.1 ETH
    let deadline = get_deadline_from_now(300); // 5 minutes
    
    // Get quote first
    let weth: Address = "0x4200000000000000000000000000000000000006".parse()?;
    let path = vec![weth, token_addr];
    let amounts_out = trader.get_amounts_out(eth_amount, path).await?;
    let expected_tokens = amounts_out[1];
    
    // Calculate minimum with 5% slippage
    let min_tokens = expected_tokens * 95 / 100;
    
    info!("💰 Expected: {}, Minimum: {}", expected_tokens, min_tokens);
    
    // Custom gas configuration for speed
    let gas_config = GasConfig::new()
        .with_gas_limit(800_000)
        .with_eip1559_gas(10_000_000_000, 5_000_000_000); // 10 gwei max, 5 gwei priority
    
    // Execute swap
    let receipt = trader.swap_eth_for_tokens(
        eth_amount,
        token_addr, 
        min_tokens,
        recipient,
        deadline,
        Some(gas_config)
    ).await?;
    
    info!("🎯 Swap completed! Block: {}", receipt.block_number.unwrap_or_default());
    Ok(())
}

// Example of selling tokens back to ETH
pub async fn sell_tokens_example(
    token_address: &str,
    private_key: &str,
    rpc_url: &str,
    token_amount: &str
) -> Result<()> {
    // Setup  
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let wallet: LocalWallet = private_key.parse()?;
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);
    let trader = UniswapTrader::new(client.clone())?;
    
    // Parse values
    let token_addr: Address = token_address.parse()?;
    let recipient = client.address();
    let amount_to_sell: U256 = token_amount.parse()?;
    
    // Check current allowance and approve if needed
    let current_allowance = trader.check_allowance(token_addr, recipient).await?;
    if current_allowance < amount_to_sell {
        info!("🔓 Approving token spending...");
        trader.approve_token(
            token_addr,
            U256::MAX, // Approve maximum to avoid future approvals
            Some(GasConfig::fast())
        ).await?;
    }
    
    // Get quote for selling
    let weth: Address = "0x4200000000000000000000000000000000000006".parse()?;
    let path = vec![token_addr, weth];
    let amounts_out = trader.get_amounts_out(amount_to_sell, path).await?;
    let expected_eth = amounts_out[1];
    
    // 3% slippage for selling
    let min_eth = expected_eth * 97 / 100;
    let deadline = get_deadline_from_now(300);
    
    info!("💸 Selling {} tokens for minimum {} ETH", amount_to_sell, min_eth);
    
    // Execute sell
    let receipt = trader.swap_tokens_for_eth(
        token_addr,
        amount_to_sell,
        min_eth,
        recipient,
        deadline,
        Some(GasConfig::turbo()) // Use turbo gas for selling
    ).await?;
    
    info!("💰 Tokens sold! Tx: {:?}", receipt.transaction_hash);
    Ok(())
}

// Example of checking token info and balance
pub async fn check_token_info_example(
    token_address: &str,
    wallet_address: &str,
    rpc_url: &str
) -> Result<()> {
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider);
    let trader = UniswapTrader::new(client)?;
    
    let token_addr: Address = token_address.parse()?;
    let wallet_addr: Address = wallet_address.parse()?;
    
    // Get token information
    let (name, symbol, decimals) = trader.get_token_info(token_addr).await?;
    info!("📋 Token: {} ({}) - {} decimals", name, symbol, decimals);
    
    // Get balance
    let balance = trader.get_token_balance(token_addr, wallet_addr).await?;
    let formatted_balance = balance.as_u128() as f64 / 10f64.powi(decimals as i32);
    info!("💰 Balance: {:.6} {}", formatted_balance, symbol);
    
    Ok(())
}

// Example of token-to-token swap (NO ETH involved)
pub async fn token_to_token_swap_example(
    token_in_address: &str,
    token_out_address: &str,
    amount_in: &str,
    private_key: &str,
    rpc_url: &str
) -> Result<()> {
    info!("🔄 Executing token-to-token swap");
    
    // Setup
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let wallet: LocalWallet = private_key.parse()?;
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);
    let trader = UniswapTrader::new(client.clone())?;
    
    // Parse addresses and amount
    let token_in: Address = token_in_address.parse()?;
    let token_out: Address = token_out_address.parse()?;
    let amount: U256 = amount_in.parse()?;
    let recipient = client.address();
    
    // Method 1: Quick swap with automatic approval
    let receipt = quick_swap_tokens_with_approval(
        &trader,
        token_in,
        token_out,
        amount,
        3.0,    // 3% slippage
        recipient,
        false   // Direct swap (not via WETH)
    ).await?;
    
    info!("✅ Token-to-token swap completed! Tx: {:?}", receipt.transaction_hash);
    Ok(())
}

// Example of multi-hop token swap via WETH
pub async fn token_to_token_via_weth_example(
    token_in_address: &str,
    token_out_address: &str,
    amount_in: &str,
    private_key: &str,
    rpc_url: &str
) -> Result<()> {
    info!("🔄 Executing token-to-token swap via WETH");
    
    // Setup
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let wallet: LocalWallet = private_key.parse()?;
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);
    let trader = UniswapTrader::new(client.clone())?;
    
    // Parse addresses and amount
    let token_in: Address = token_in_address.parse()?;
    let token_out: Address = token_out_address.parse()?;
    let amount: U256 = amount_in.parse()?;
    let recipient = client.address();
    
    // Method 2: Multi-hop swap via WETH
    let receipt = quick_swap_tokens_with_approval(
        &trader,
        token_in,
        token_out,
        amount,
        5.0,    // 5% slippage (higher for multi-hop)
        recipient,
        true    // Via WETH routing
    ).await?;
    
    info!("✅ Multi-hop token swap completed! Tx: {:?}", receipt.transaction_hash);
    Ok(())
}

// Example of manual token-to-token swap with custom gas
pub async fn manual_token_to_token_swap(
    token_in_address: &str,
    token_out_address: &str,
    amount_in: &str,
    private_key: &str,
    rpc_url: &str
) -> Result<()> {
    info!("🔄 Manual token-to-token swap with custom settings");
    
    // Setup
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let wallet: LocalWallet = private_key.parse()?;
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);
    let trader = UniswapTrader::new(client.clone())?;
    
    // Parse addresses and amount
    let token_in: Address = token_in_address.parse()?;
    let token_out: Address = token_out_address.parse()?;
    let amount: U256 = amount_in.parse()?;
    let recipient = client.address();
    
    // Check allowance and approve if needed
    let current_allowance = trader.check_allowance(token_in, recipient).await?;
    if current_allowance < amount {
        info!("🔓 Approving token for swap...");
        trader.approve_token(
            token_in,
            U256::MAX,
            Some(GasConfig::fast())
        ).await?;
    }
    
    // Get quote for direct swap
    let amounts_out = trader.get_token_to_token_quote(token_in, token_out, amount).await?;
    let expected_output = amounts_out[1];
    
    // Calculate minimum with 2% slippage
    let min_output = expected_output * 98 / 100;
    
    info!("💰 Expected: {}, Minimum: {}", expected_output, min_output);
    
    // Custom gas for fastest execution
    let turbo_gas = GasConfig::new()
        .with_gas_limit(1_200_000)
        .with_eip1559_gas(25_000_000_000, 15_000_000_000); // Very high gas for speed
    
    // Execute direct swap
    let receipt = trader.swap_exact_tokens_for_tokens_direct(
        token_in,
        token_out,
        amount,
        min_output,
        recipient,
        Some(turbo_gas)
    ).await?;
    
    info!("🎯 Manual token swap completed! Block: {}", receipt.block_number.unwrap_or_default());
    Ok(())
}

// Example of custom path token swap
pub async fn custom_path_token_swap(
    path: Vec<&str>,
    amount_in: &str,
    private_key: &str,
    rpc_url: &str
) -> Result<()> {
    if path.len() < 2 {
        return Err(anyhow::anyhow!("Path must contain at least 2 tokens"));
    }
    
    info!("🔄 Custom path token swap with {} hops", path.len() - 1);
    
    // Setup
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let wallet: LocalWallet = private_key.parse()?;
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);
    let trader = UniswapTrader::new(client.clone())?;
    
    // Parse path and amount
    let parsed_path: Result<Vec<Address>> = path.iter()
        .map(|addr| addr.parse())
        .collect();
    let token_path = parsed_path?;
    let amount: U256 = amount_in.parse()?;
    let recipient = client.address();
    
    // Approve first token in path
    let token_in = token_path[0];
    let current_allowance = trader.check_allowance(token_in, recipient).await?;
    if current_allowance < amount {
        info!("🔓 Approving first token in path...");
        trader.approve_token(
            token_in,
            U256::MAX,
            Some(GasConfig::fast())
        ).await?;
    }
    
    // Get quote for custom path
    let amounts_out = trader.get_amounts_out(amount, token_path.clone()).await?;
    let expected_output = amounts_out[amounts_out.len() - 1];
    
    // 4% slippage for complex paths
    let min_output = expected_output * 96 / 100;
    
    info!("💰 Custom path - Expected: {}, Minimum: {}", expected_output, min_output);
    
    // Execute custom path swap
    let receipt = trader.swap_tokens_custom_path(
        token_path,
        amount,
        min_output,
        recipient,
        Some(GasConfig::turbo())
    ).await?;
    
    info!("🎯 Custom path swap completed! Tx: {:?}", receipt.transaction_hash);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_gas_configurations() {
        // Test different gas configurations
        let default_gas = GasConfig::default();
        let fast_gas = GasConfig::fast();
        let turbo_gas = GasConfig::turbo();
        
        println!("Default: {:?}", default_gas);
        println!("Fast: {:?}", fast_gas);
        println!("Turbo: {:?}", turbo_gas);
        
        // Test custom configuration
        let custom = GasConfig::new()
            .with_gas_limit(1_200_000)
            .with_eip1559_gas(25_000_000_000, 15_000_000_000);
        
        println!("Custom: {:?}", custom);
    }
} 