# Testing Functionality

The `testDetector.rs` provides historical testing capabilities while keeping the **exact same core logic** as `detector.rs`.

## Core Logic Guarantee

✅ **ALL CORE DETECTION LOGIC IS UNCHANGED**:
- `extract_token_and_caller()` - Identical regex and confidence logic
- `verify_caller()` - Same caching and verification process  
- `process_event()` - Same confidence handling and callback execution
- Same constants, patterns, and cache management

## Testing Features

### Single Block Testing
```bash
# Test a specific block number
cargo run --bin testDetector 12345678
```

### Block Range Testing  
```bash
# Test a range of blocks
cargo run --bin testDetector 12345678 12345680
```

## Usage Examples

1. **Test Recent Block**:
   ```bash
   # Replace with actual block number
   cargo run --bin testDetector 21850000
   ```

2. **Test Block Range** (max ~1000 blocks recommended):
   ```bash
   # Test last 10 blocks of activity
   cargo run --bin testDetector 21850000 21850010
   ```

3. **Live Mode** (original detector):
   ```bash
   # Live monitoring (default)
   cargo run
   # Or explicitly
   cargo run --bin detector
   ```

## Expected Output

### When Tokens Found:
```
🧪 Testing block 12345678 for token deployments...
📊 Found 3 logs in block 12345678
🔍 Found token: 0xabc123... (confidence: Verify)
✅ VERIFIED token detected: 0xabc123...
🎯 Detected 1 matching tokens in block 12345678
🎯 DETECTED TOKENS:
   0xabc123...
```

### When No Tokens Found:
```
🧪 Testing block 12345678 for token deployments...
📊 Found 0 logs in block 12345678
🔍 No matching tokens found in block 12345678
```

## How It Works

1. **Historical Log Retrieval**: Uses `eth_getLogs` with block filters
2. **Same Detection Logic**: Processes logs through identical core functions
3. **Confidence Processing**: Applies same WANTED/UNWANTED/VERIFY logic
4. **Transaction Verification**: Uses same caching and verification when needed
5. **Output Summary**: Lists all detected tokens that would trigger in live mode

## Configuration

The testing mode uses the **exact same configuration** as live mode:
- Same `TARGET_TOPIC`, `DEPLOYER`, `WANTED`, `UNWANTED` constants
- Same `USE_TX_VERIFICATION` setting
- Same WSS_URL environment variable requirement

## Performance Notes

- **Block Range Limit**: Keep ranges under 1000 blocks to avoid timeouts
- **Network Calls**: Each verification still requires individual transaction lookups  
- **Cache Benefits**: Repeated calls benefit from caller verification caching
- **Logging**: Use `RUST_LOG=info` for detailed output, `RUST_LOG=error` for quiet mode

## Comparison Guarantee

Since the core logic is **100% identical**, tokens detected in testing mode will behave exactly the same in live monitoring mode. This allows you to:

- ✅ Validate detection accuracy against historical data
- ✅ Test configuration changes before going live  
- ✅ Debug specific block ranges where issues occurred
- ✅ Verify that historical deployments would have been caught 