use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub source_market: Pubkey,
    pub target_market: Pubkey,
    pub token_pair: TokenPair,
    pub profit_percentage: f64,
    pub required_amount: u64,
    pub estimated_profit: u64,
    pub route: Vec<TradeStep>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub base_token: Token,
    pub quote_token: Token,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub address: Pubkey,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeStep {
    pub market: Pubkey,
    pub side: TradeSide,
    pub amount: u64,
    pub price: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketState {
    pub market_address: Pubkey,
    pub base_token: Token,
    pub quote_token: Token,
    pub best_bid: f64,
    pub best_ask: f64,
    pub last_update: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashLoanParams {
    pub token: Token,
    pub amount: u64,
    pub protocol: FlashLoanProtocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlashLoanProtocol {
    Solend,
    Port,
    Marinade,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub keypair: Option<Keypair>,
    pub rpc_url: String,
    pub min_profit_percentage: f64,
    pub max_trade_size: u64,
    pub markets_whitelist: Option<Vec<Pubkey>>,
    pub tokens_whitelist: Option<Vec<Pubkey>>,
    pub use_flash_loans: bool,
    pub mev_protection: bool,
    pub quantum_security: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub profit_realized: Option<u64>,
    pub error: Option<String>,
    pub transaction_signature: Option<String>,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPrices {
    pub market: Pubkey,
    pub prices: HashMap<String, f64>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Maximum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub level: SecurityLevel,
    pub max_slippage: f64,
    pub timeout_ms: u64,
    pub require_signatures: bool,
}

// Error types for the arbitrage bot
#[derive(Debug, thiserror::Error)]
pub enum ArbitrageError {
    #[error("Market error: {0}")]
    MarketError(String),
    
    #[error("Insufficient funds: {0}")]
    InsufficientFunds(String),
    
    #[error("Flash loan error: {0}")]
    FlashLoanError(String),
    
    #[error("Transaction error: {0}")]
    TransactionError(String),
    
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("MEV attack detected: {0}")]
    MevAttackDetected(String),
}

// Result type alias for arbitrage operations
pub type ArbitrageResult<T> = Result<T, ArbitrageError>;
