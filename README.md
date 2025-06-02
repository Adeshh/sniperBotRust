# 🚀 Rust Token Sniping Bot

A high-performance, real-time token detection and automatic swapping bot built in Rust for Base network. The bot monitors token deployments and executes immediate swaps with minimal latency.

## 📋 Features

- **Real-time Token Detection**: WebSocket-based monitoring for instant detection
- **Immediate Swap Execution**: Sub-second swap execution upon token detection  
- **Historical Testing**: Test detection logic on past block ranges
- **Minimal Latency**: Optimized for maximum speed with streamlined logging
- **Base Network**: Configured for Base (Chain ID 8453) with Uniswap V2
- **Verification System**: Optional transaction caller verification
- **Multiple Modes**: Production sniping and testing capabilities

## 🏗️ Architecture

### File Structure

```
src/
├── main.rs           # 🎯 Production sniping bot (live detection only)
├── detector.rs       # 🔍 Core detection logic (production)
├── testMain.rs       # 🧪 Testing bot (live + historical testing)  
├── testDetector.rs   # 🔍 Core detection + testing functions
└── uniswap.rs        # 💱 Uniswap V2 swap functionality
```

### Component Overview

| Component | Purpose | Used By |
|-----------|---------|---------|
| **main.rs** | Production sniping with minimal latency | Live trading |
| **detector.rs** | Core detection logic (live only) | main.rs |
| **testMain.rs** | Development/testing with historical data | Testing/Dev |
| **testDetector.rs** | Core detection + testing functions | testMain.rs |
| **uniswap.rs** | Swap execution and gas management | All binaries |

## 🛠️ Setup

### Prerequisites

- Rust 1.70+ 
- Base network RPC access (WebSocket required)
- Private key with VIRTUALS tokens
- Environment variables configured

### Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd rustBot
   ```

2. **Install dependencies**
   ```bash
   cargo build
   ```

3. **Configure environment variables**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

### Environment Variables

Create a `.env` file with the following variables:

```env
# Required
PRIVATE_KEY=your_private_key_here
WSS_URL=wss://base-mainnet.g.alchemy.com/v2/your-api-key

# Optional
USE_TX_VERIFICATION=true  # Enable transaction caller verification (default: true)
```

⚠️ **Security Note**: Never commit your `.env` file or private keys to version control.

## 🚀 Usage

### Production Sniping (Live Detection)

The main production bot for real-time token sniping:

```bash
# Start live sniping (default run)
cargo run

# Or explicitly run main binary
cargo run --bin main
```

**Flow:**
```
🔴 Live monitoring → 🎯 Token detected → ⚡ Immediate swap → 📋 Results logged
```

**Output Example:**
```
🚀 Starting live token detection and auto-swap system
✅ Uniswap trader initialized
✅ Token detector initialized  
🔴 LIVE DETECTION MODE - Monitoring for real-time token deployments...
🎯 TOKEN DETECTED: 0x1234... - Executing immediate swap
🎯 SWAP SENT! Hash: 0xabcd...
⚡ Execution Time: 89ms
✅ Swap execution completed for token: 0x1234...
```

### Testing & Development

For testing detection logic and development:

```bash
# Live detection with testing capabilities
cargo run --bin testMain

# Historical testing on specific blocks
cargo run --bin testMain 30948300 30948310

# Single block testing
cargo run --bin testMain 30948304
```

### Individual Components

```bash
# Core detection only (no swapping)
cargo run --bin detector

# Testing detection only (no swapping)  
cargo run --bin testDetector 30948300 30948310
```

## ⚙️ Configuration

### Token Configuration

The bot is configured to swap **VIRTUALS → Detected Tokens**:

- **Input Token**: VIRTUALS (`0x0b3e328455c4059eeb9e3f84b5543f74e24e7e1b`)
- **Amount**: 0.001 VIRTUALS per swap
- **Network**: Base (Chain ID 8453)
- **DEX**: Uniswap V2

### Detection Parameters

The bot monitors for tokens deployed by specific addresses:

- **Target Deployer**: `0x71B8EFC8BCaD65a5D9386D07f2Dff57ab4EAf533`
- **Wanted Caller**: `0x81F7cA6AF86D1CA6335E44A2C28bC88807491415`  
- **Unwanted Caller**: `0x03Fb99ea8d3A832729a69C3e8273533b52f30D1A`

### Gas Configuration

- **Default Gas**: Conservative settings for reliable execution
- **Gas Limit**: Auto-calculated with buffer
- **Chain**: Base network (low gas fees)

## 🧪 Testing Workflow

### 1. Historical Testing

Test detection logic on past blocks to verify accuracy:

```bash
# Test specific block range
cargo run --bin testMain 30948300 30948310

# Expected output:
🧪 HISTORICAL TEST MODE - Testing block range: 30948300 to 30948310
🎯 Detected 2 token(s), executing swaps...
✅ Swap completed for token: 0x1234...
```

### 2. Live Testing

Test live detection without historical data:

```bash
# Live testing mode
cargo run --bin testMain

# Will detect and swap real-time tokens
```

### 3. Detection Only

Test just the detection logic without swapping:

```bash
# Core detection testing
cargo run --bin testDetector 30948300 30948310

# Will show detected tokens without executing swaps
```

## 🎯 Detection Logic

### Token Confidence Levels

The detection system uses three confidence levels:

1. **Wanted** ✅: Direct match with wanted caller address
2. **Unwanted** ❌: Direct match with unwanted caller address  
3. **Verify** 🔍: Requires transaction verification

### Verification Process

When `USE_TX_VERIFICATION=true`:

1. Extract token address from log data
2. Check if deployer matches target address
3. If confidence requires verification:
   - Fetch transaction details via WebSocket
   - Verify transaction caller
   - Cache results for performance

### Monitoring Behavior

- **Wanted tokens**: Immediate swap execution + stop monitoring
- **Unwanted tokens**: Log rejection + continue monitoring
- **Verification failures**: Log rejection + continue monitoring
- **Network errors**: Log error + continue monitoring

## 🔧 Development

### Building

```bash
# Check compilation
cargo check

# Build all binaries
cargo build

# Build specific binary
cargo build --bin main
```

### Testing

```bash
# Run tests
cargo test

# Check with clippy
cargo clippy

# Format code
cargo fmt
```

### Performance Optimization

The bot is optimized for minimal latency:

- **WebSocket connections** for real-time data
- **Minimal logging** before swap execution
- **Direct token return** (no storage overhead)
- **Inline callback execution** for immediate swaps
- **Efficient caching** for transaction verification

## 📊 Monitoring & Logs

### Log Levels

- **INFO**: Normal operation, detection events, swap results
- **ERROR**: Swap failures, network errors, configuration issues

### Key Metrics Logged

- **Execution Time**: Time from detection to swap completion
- **Gas Used**: Actual gas consumption
- **Transaction Hash**: For blockchain verification
- **Explorer Links**: Direct links to BaseScan

### Example Log Output

```
2024-01-15T10:30:45.123Z INFO: 🚀 Starting live token detection and auto-swap system
2024-01-15T10:30:45.456Z INFO: ✅ Uniswap trader initialized
2024-01-15T10:30:45.789Z INFO: ✅ Token detector initialized
2024-01-15T10:30:46.012Z INFO: 🔴 LIVE DETECTION MODE - Monitoring...
2024-01-15T10:32:01.234Z INFO: 🎯 TOKEN DETECTED: 0x1234... - Executing immediate swap
2024-01-15T10:32:01.345Z INFO: 🎯 SWAP SENT! Hash: 0xabcd...
2024-01-15T10:32:01.346Z INFO: ⚡ Execution Time: 111ms
2024-01-15T10:32:01.347Z INFO: ⛽ Gas Used: 150000
2024-01-15T10:32:01.348Z INFO: 🔗 Explorer: https://basescan.org/tx/0xabcd...
```

## ⚠️ Important Notes

### Security

- **Never commit private keys** to version control
- **Use environment variables** for sensitive data
- **Test on small amounts** before production use
- **Monitor gas prices** and adjust accordingly

### Network Considerations

- **WebSocket reliability**: Use stable RPC providers
- **Base network status**: Monitor for congestion
- **Gas price volatility**: Adjust strategies as needed
- **Transaction confirmation**: Typical 2-4 seconds on Base

### Risk Management

- **Start with small amounts** (0.001 VIRTUALS default)
- **Test thoroughly** with historical data first
- **Monitor swap success rates** and adjust gas settings
- **Have emergency stop procedures** ready

## 🚀 Quick Start

1. **Setup environment**:
   ```bash
   git clone <repo>
   cd rustBot
   cp .env.example .env
   # Edit .env with your keys
   ```

2. **Test with historical data**:
   ```bash
   cargo run --bin testMain 30948300 30948310
   ```

3. **Start live sniping**:
   ```bash
   cargo run
   ```

## 📈 Performance Tips

- **Use Alchemy/QuickNode** for better RPC performance
- **Monitor Base gas tracker** for optimal timing
- **Consider private mempools** for competitive advantage
- **Adjust gas limits** based on network conditions
- **Use multiple RPC endpoints** for redundancy

## 🛟 Troubleshooting

### Common Issues

- **Connection errors**: Check WSS_URL and network connectivity
- **Gas failures**: Increase gas limit or check Base network status
- **No tokens detected**: Verify deployer addresses and network
- **Swap failures**: Check VIRTUALS balance and allowances

### Debug Steps

1. Test with historical blocks first
2. Check environment variables
3. Verify network connectivity  
4. Monitor Base network status
5. Check token balances

---

**Built with ⚡ for maximum speed token sniping on Base network.** 