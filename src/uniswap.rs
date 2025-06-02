use anyhow::{Result, anyhow};
use ethers::prelude::*;
use ethers::types::{Address, U256};
use std::sync::Arc;
use tracing::info;

// Uniswap V2 Router address (Base network)
const UNISWAP_V2_ROUTER: &str = "0x4752ba5dbc23f44d87826276bf6fd6b1c372ad24";

// Gas configuration
#[derive(Debug, Clone)]
pub struct GasConfig {
    pub gas_limit: U256,
    pub gas_price: Option<U256>,  // For legacy transactions
    pub max_fee_per_gas: Option<U256>,  // For EIP-1559
    pub max_priority_fee_per_gas: Option<U256>,  // For EIP-1559
}

impl Default for GasConfig {
    fn default() -> Self {
        Self {
            gas_limit: U256::from(5_000_000),  // 500k gas limit
            gas_price: None,
            max_fee_per_gas: Some(U256::from(2_500_000u64)),  // 0.02 gwei
            max_priority_fee_per_gas: Some(U256::from(1_500_000u64)),  // 0.1 gwei
        }
    }
}

impl GasConfig {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_gas_limit(mut self, gas_limit: u64) -> Self {
        self.gas_limit = U256::from(gas_limit);
        self
    }
    
    pub fn with_legacy_gas_price(mut self, gas_price: u64) -> Self {
        self.gas_price = Some(U256::from(gas_price));
        self.max_fee_per_gas = None;
        self.max_priority_fee_per_gas = None;
        self
    }
    
    pub fn with_eip1559_gas(mut self, max_fee: u64, priority_fee: u64) -> Self {
        self.max_fee_per_gas = Some(U256::from(max_fee));
        self.max_priority_fee_per_gas = Some(U256::from(priority_fee));
        self.gas_price = None;
        self
    }
    
    pub fn fast() -> Self {
        Self {
            gas_limit: U256::from(800_000),
            gas_price: None,
            max_fee_per_gas: Some(U256::from(5_000_000_000u64)),  // 5 gwei
            max_priority_fee_per_gas: Some(U256::from(2_000_000_000u64)),  // 2 gwei
        }
    }
    
    pub fn turbo() -> Self {
        Self {
            gas_limit: U256::from(500_000),
            gas_price: None,
            max_fee_per_gas: Some(U256::from(20_000_000_000u64)),  // 20 gwei
            max_priority_fee_per_gas: Some(U256::from(10_000_000_000u64)),  // 10 gwei
        }
    }
}

// Uniswap V2 Router ABI (simplified)
abigen!(
    UniswapV2Router,
    r#"[
        function swapExactTokensForTokens(uint amountIn, uint amountOutMin, address[] calldata path, address to, uint deadline) external returns (uint[] memory amounts)
    ]"#
);

// ERC20 Token ABI (simplified)
abigen!(
    ERC20Token,
    r#"[
        function approve(address spender, uint256 amount) external returns (bool)
        function allowance(address owner, address spender) external view returns (uint256)
    ]"#
);

pub struct UniswapTrader<M> {
    client: Arc<M>,
    router: UniswapV2Router<M>,
}

impl<M: Middleware + 'static> UniswapTrader<M> {
    pub fn new(client: Arc<M>) -> Result<Self> {
        let router_address: Address = UNISWAP_V2_ROUTER.parse()?;
        let router = UniswapV2Router::new(router_address, client.clone());
        
        Ok(Self {
            client,
            router,
        })
    }
    
    // Approve token spending
    pub async fn approve_token(
        &self, 
        token_address: Address, 
        amount: U256,
        gas_config: Option<GasConfig>
    ) -> Result<TransactionReceipt> {
        info!("✅ Approving token {} for amount: {}", token_address, amount);
        
        let token = ERC20Token::new(token_address, self.client.clone());
        let router_address: Address = UNISWAP_V2_ROUTER.parse()?;
        
        let mut tx = token.approve(router_address, amount);
        
        // Apply gas configuration
        if let Some(gas_config) = gas_config {
            tx = tx.gas(gas_config.gas_limit);
            
            if let Some(gas_price) = gas_config.gas_price {
                tx = tx.gas_price(gas_price);
            } else if let Some(max_fee) = gas_config.max_fee_per_gas {
                tx = tx.gas_price(max_fee);  // For simplicity, using gas_price for EIP-1559
            }
        }
        
        let pending_tx = tx.send().await?;
        info!("📤 Approval transaction sent: {:?}", pending_tx.tx_hash());
        
        let receipt = pending_tx.await?.ok_or_else(|| anyhow!("Approval transaction failed"))?;
        info!("✅ Approval confirmed in block: {}", receipt.block_number.unwrap_or_default());
        
        Ok(receipt)
    }
    
    // Check token allowance
    pub async fn check_allowance(&self, token_address: Address, owner: Address) -> Result<U256> {
        let token = ERC20Token::new(token_address, self.client.clone());
        let router_address: Address = UNISWAP_V2_ROUTER.parse()?;
        
        let allowance = token.allowance(owner, router_address).call().await?;
        info!("🔍 Current allowance: {}", allowance);
        
        Ok(allowance)
    }
    
    // Swap exact tokens for tokens
    pub async fn swap_exact_tokens_for_tokens(
        &self,
        amount_in: U256,
        amount_out_min: U256,
        path: Vec<Address>,
        to: Address,
        deadline: U256,
        gas_config: Option<GasConfig>
    ) -> Result<TransactionReceipt> {
        info!("🔄 Swapping exact tokens: {} for minimum {} tokens", amount_in, amount_out_min);
        info!("📍 Path: {:?}", path);
        
        let mut tx = self.router
            .swap_exact_tokens_for_tokens(amount_in, amount_out_min, path, to, deadline);
        
        // Apply gas configuration
        if let Some(gas_config) = gas_config {
            tx = tx.gas(gas_config.gas_limit);
            
            if let Some(gas_price) = gas_config.gas_price {
                tx = tx.gas_price(gas_price);
            } else if let Some(max_fee) = gas_config.max_fee_per_gas {
                tx = tx.gas_price(max_fee);  // For simplicity, using gas_price for EIP-1559
            }
        }
        
        let pending_tx = tx.send().await?;
        info!("📤 Swap transaction sent: {:?}", pending_tx.tx_hash());
        
        let receipt = pending_tx.await?.ok_or_else(|| anyhow!("Swap transaction failed"))?;
        info!("✅ Swap confirmed in block: {}", receipt.block_number.unwrap_or_default());
        
        Ok(receipt)
    }
}

// Utility functions
pub fn get_deadline_from_now(seconds: u64) -> U256 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    U256::from(now + seconds)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gas_config() {
        let fast_config = GasConfig::fast();
        assert_eq!(fast_config.gas_limit, U256::from(800_000));
        
        let custom_config = GasConfig::new()
            .with_gas_limit(1_000_000)
            .with_eip1559_gas(10_000_000_000, 5_000_000_000);
        assert_eq!(custom_config.gas_limit, U256::from(1_000_000));
    }
} 