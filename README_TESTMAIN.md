# testMain.rs - Token Detection + Auto Swap

Combines historical token detection with immediate Uniswap V2 swapping using **WebSocket for maximum speed**.

## Setup

1. Ensure you have unlimited USDC approval for Uniswap V2 router
2. Set required environment variables in `.env`:

```env
# WebSocket URL (used for both detection and swapping)
WSS_URL=wss://your-base-websocket-url
PRIVATE_KEY=your_private_key_here

# Optional: Transaction verification (default: true)
USE_TX_VERIFICATION=true
```

## Usage

```bash
# Test specific block range and auto-swap detected tokens
cargo run --bin testMain <from_block> <to_block>

# Example: Test blocks 30948300 to 30948600
cargo run --bin testMain 30948300 30948600
```

## Speed Optimization

### **WebSocket vs HTTP RPC Speed Comparison:**

| Connection Type | Latency | Use Case | Speed |
|----------------|---------|----------|-------|
| **WebSocket (WSS)** | ~10-50ms | Sniping/Real-time | ⚡ **FASTEST** |
| HTTP RPC | ~50-200ms | Regular trading | 🐌 Slower |

### **Why WebSocket is Faster:**
- **Persistent Connection**: No connection overhead per request
- **Lower Latency**: Direct pipeline to blockchain node
- **Real-time**: Ideal for time-sensitive operations like sniping
- **Reduced Network Overhead**: No HTTP headers per transaction

## How It Works

1. **Token Detection**: Uses testDetector logic to scan historical blocks for token deployments
2. **Immediate Swapping**: When a token is detected, automatically swaps 1 USDC for the new token **via WebSocket**
3. **Clean Output**: Minimal logging focused on detection and swap results

## Configuration

- **Connection**: WebSocket only (fastest for sniping)
- **Swap Amount**: 1 USDC (1,000,000 units with 6 decimals)
- **Slippage**: Accepts any amount of output tokens (amount_out_min = 1)
- **Gas**: Uses turbo gas settings for fastest execution
- **Deadline**: 5 minutes from swap initiation

## Expected Output

```
🚀 Starting token detection and auto-swap system
✅ Uniswap trader initialized (WebSocket)
✅ Token detector initialized
🔍 Testing block range: 30948300 to 30948600
📊 Found 1 logs in block range 30948300 to 30948600
🔍 Block 30948304: Found token: 0x44aa51452b267c81ed99f0ef2bdc7c8ba47ee1a2 (confidence: Verify)
✅ Block 30948304: VERIFIED token detected: 0x44aa51452b267c81ed99f0ef2bdc7c8ba47ee1a2
🎯 Detected 1 token(s), executing swaps...
🔄 Executing swap for detected token: 0x44aa51452b267c81ed99f0ef2bdc7c8ba47ee1a2
💱 Swapping 1 USDC for 0x44aa51452b267c81ed99f0ef2bdc7c8ba47ee1a2 via WebSocket
🎯 Swap successful! Tx: 0x...
✅ Swap completed for token: 0x44aa51452b267c81ed99f0ef2bdc7c8ba47ee1a2
```

## Requirements

- USDC balance in wallet
- Unlimited USDC approval to Uniswap V2 router
- **WebSocket endpoint** for Base network (faster than HTTP RPC)
- Private key with sufficient ETH for gas

## Performance Tips

1. **Use High-Quality WSS Endpoints**:
   - Alchemy WebSocket: `wss://base-mainnet.g.alchemy.com/v2/YOUR_KEY`
   - QuickNode WebSocket: `wss://your-endpoint.base.quiknode.pro/YOUR_KEY/`
   
2. **Avoid Public WSS** (slower and rate-limited)

3. **For Maximum Speed**:
   - Use paid WSS provider (Alchemy, QuickNode, Infura)
   - Choose provider geographically close to you
   - Use turbo gas settings

## Safety Notes

- Only swaps detected tokens that pass confidence checks
- Uses minimal swap amounts for testing
- Includes 5-minute deadline protection
- No approval needed (assumes pre-approved USDC)
- **WebSocket connection** for lowest possible latency 