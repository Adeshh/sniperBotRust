# Uniswap V2 Trading Module

This module provides comprehensive Uniswap V2 trading functionality with custom gas settings for the Rust sniper bot.

## Features

- ✅ **Token Approval Functions** - Approve tokens for trading with custom gas
- 🔄 **Swap Functions** - ETH ↔ Token and Token ↔ Token swaps
- ⚡ **Custom Gas Settings** - Configurable gas for speed optimization
- 💰 **Quote Functions** - Get trading quotes before execution
- 📊 **Token Information** - Fetch token details and balances
- 🚀 **Quick Buy Helper** - One-function token purchasing with slippage protection

## Gas Configuration

### Preset Configurations

```rust
use crate::uniswap::GasConfig;

// Default: Conservative gas settings
let default = GasConfig::default();

// Fast: Higher gas for faster execution
let fast = GasConfig::fast();

// Turbo: Maximum speed for sniping
let turbo = GasConfig::turbo();
```

### Custom Gas Settings

```rust
// Custom EIP-1559 gas configuration
let custom_gas = GasConfig::new()
    .with_gas_limit(800_000)
    .with_eip1559_gas(10_000_000_000, 5_000_000_000); // 10 gwei max, 5 gwei priority

// Legacy gas pricing
let legacy_gas = GasConfig::new()
    .with_gas_limit(1_000_000)
    .with_legacy_gas_price(20_000_000_000); // 20 gwei
```

## Usage Examples

### 1. Quick Token Purchase

```rust
use crate::uniswap::{UniswapTrader, quick_buy_token};

// Simple one-function buy with automatic slippage calculation
let receipt = quick_buy_token(
    &trader,
    token_address,
    "0.1",      // 0.1 ETH
    5.0,        // 5% slippage
    recipient
).await?;
```

### 2. Manual Swap with Custom Gas

```rust
// ETH -> Token swap with turbo gas
let receipt = trader.swap_eth_for_tokens(
    parse_ether("0.1")?,
    token_address,
    min_tokens_out,
    recipient,
    get_deadline_from_now(300),
    Some(GasConfig::turbo())
).await?;
```

### 3. Token Approval

```rust
// Approve token for trading (required before selling)
let receipt = trader.approve_token(
    token_address,
    U256::MAX,  // Approve maximum to avoid future approvals
    Some(GasConfig::fast())
).await?;

// Check current allowance
let allowance = trader.check_allowance(token_address, owner_address).await?;
```

### 4. Selling Tokens

```rust
// Token -> ETH swap
let receipt = trader.swap_tokens_for_eth(
    token_address,
    token_amount,
    min_eth_out,
    recipient,
    get_deadline_from_now(300),
    Some(GasConfig::turbo())
).await?;
```

### 5. Get Trading Quotes

```rust
// Get expected output before trading
let amounts_out = trader.get_amounts_out(
    parse_ether("0.1")?,
    vec![weth_address, token_address]
).await?;

let expected_tokens = amounts_out[1];
println!("Expected tokens: {}", expected_tokens);
```

### 6. Token Information

```rust
// Get token details
let (name, symbol, decimals) = trader.get_token_info(token_address).await?;
println!("Token: {} ({}) - {} decimals", name, symbol, decimals);

// Get token balance
let balance = trader.get_token_balance(token_address, wallet_address).await?;
```

## Integration with Token Detector

### Automatic Trading on Detection

```rust
// In your detection callback
pub async fn on_token_detected(token_address: &str) -> Result<()> {
    info!("🎯 Token detected: {}", token_address);
    
    // Execute immediate buy
    let receipt = execute_trade_on_detection(
        token_address,
        &env::var("PRIVATE_KEY")?,
        &env::var("RPC_URL")?,
        "0.05",  // 0.05 ETH
        3.0      // 3% slippage
    ).await?;
    
    info!("✅ Purchase complete: {:?}", receipt.transaction_hash);
    Ok(())
}
```

## Gas Settings Comparison

| Configuration | Gas Limit | Max Fee (gwei) | Priority Fee (gwei) | Use Case |
|---------------|-----------|----------------|-------------------|----------|
| `default()` | 500k | 1 | 0.1 | Regular trading |
| `fast()` | 800k | 5 | 2 | Quick execution |
| `turbo()` | 1M | 20 | 10 | Sniping/MEV |

## Environment Variables

Add to your `.env` file:

```env
# Trading configuration
PRIVATE_KEY=your_private_key_here
RPC_URL=https://mainnet.base.org
DEFAULT_SLIPPAGE=5.0
DEFAULT_ETH_AMOUNT=0.1
```

## Safety Features

- **Slippage Protection**: Automatic minimum output calculation
- **Deadline Protection**: Transactions expire after set time
- **Allowance Checking**: Prevents unnecessary approval transactions
- **Balance Verification**: Check balances before trading
- **Gas Estimation**: Configurable gas for different network conditions

## Error Handling

All functions return `Result<T>` for proper error handling:

```rust
match trader.swap_eth_for_tokens(...).await {
    Ok(receipt) => info!("✅ Swap successful: {:?}", receipt.transaction_hash),
    Err(e) => error!("❌ Swap failed: {}", e),
}
```

## Network Configuration

Currently configured for **Base Network**:
- Uniswap V2 Router: `0x4752ba5dbc23f44d87826276bf6fd6b1c372ad24`
- WETH: `0x4200000000000000000000000000000000000006`

To use on other networks, update the constants in `src/uniswap.rs`.

## Performance Tips

1. **Use `turbo()` gas for sniping** - Highest chance of inclusion
2. **Pre-approve tokens** - Avoid approval step during selling
3. **Cache trader instance** - Reuse for multiple operations
4. **Monitor gas prices** - Adjust configurations based on network congestion
5. **Use appropriate slippage** - Higher for volatile tokens, lower for stable ones

## Building and Testing

```bash
# Build with trading features
cargo build --release

# Run tests
cargo test uniswap

# Test gas configurations
cargo test test_gas_configurations
``` 