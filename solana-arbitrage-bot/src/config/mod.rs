mod settings;

pub use settings::*;

use crate::types::common::{ArbitrageError, BotConfig, SecurityConfig};
use solana_sdk::signature::Keypair;
use std::str::FromStr;

pub fn load_config() -> Result<BotConfig, ArbitrageError> {
    dotenv::dotenv().ok();
    
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .map_err(|_| ArbitrageError::ConfigError("SOLANA_RPC_URL not set".to_string()))?;

    let keypair_path = std::env::var("KEYPAIR_PATH")
        .map_err(|_| ArbitrageError::ConfigError("KEYPAIR_PATH not set".to_string()))?;

    let keypair = load_keypair(&keypair_path)?;
    
    let min_profit_percentage = std::env::var("MIN_PROFIT_PERCENTAGE")
        .map(|v| v.parse::<f64>())
        .unwrap_or(Ok(1.0))
        .map_err(|_| ArbitrageError::ConfigError("Invalid MIN_PROFIT_PERCENTAGE".to_string()))?;

    let max_trade_size = std::env::var("MAX_TRADE_SIZE")
        .map(|v| v.parse::<u64>())
        .unwrap_or(Ok(1_000_000_000))
        .map_err(|_| ArbitrageError::ConfigError("Invalid MAX_TRADE_SIZE".to_string()))?;

    let use_flash_loans = std::env::var("USE_FLASH_LOANS")
        .map(|v| v.parse::<bool>())
        .unwrap_or(Ok(false))
        .map_err(|_| ArbitrageError::ConfigError("Invalid USE_FLASH_LOANS".to_string()))?;

    let mev_protection = std::env::var("MEV_PROTECTION")
        .map(|v| v.parse::<bool>())
        .unwrap_or(Ok(true))
        .map_err(|_| ArbitrageError::ConfigError("Invalid MEV_PROTECTION".to_string()))?;

    let quantum_security = std::env::var("QUANTUM_SECURITY")
        .map(|v| v.parse::<bool>())
        .unwrap_or(Ok(true))
        .map_err(|_| ArbitrageError::ConfigError("Invalid QUANTUM_SECURITY".to_string()))?;

    Ok(BotConfig {
        keypair: Some(keypair),
        rpc_url,
        min_profit_percentage,
        max_trade_size,
        markets_whitelist: None, // Can be loaded from a separate config file
        tokens_whitelist: None,  // Can be loaded from a separate config file
        use_flash_loans,
        mev_protection,
        quantum_security,
    })
}

fn load_keypair(path: &str) -> Result<Keypair, ArbitrageError> {
    let keypair_bytes = std::fs::read_to_string(path)
        .map_err(|e| ArbitrageError::ConfigError(format!("Failed to read keypair file: {}", e)))?;
    
    let keypair_bytes = keypair_bytes.trim()
        .split(',')
        .map(|s| u8::from_str(s))
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|e| ArbitrageError::ConfigError(format!("Invalid keypair format: {}", e)))?;

    Keypair::from_bytes(&keypair_bytes)
        .map_err(|e| ArbitrageError::ConfigError(format!("Invalid keypair: {}", e)))
}

pub fn load_security_config() -> SecurityConfig {
    SecurityConfig {
        level: crate::types::common::SecurityLevel::High,
        max_slippage: 1.0, // 1% max slippage
        timeout_ms: 5000,  // 5 second timeout
        require_signatures: true,
    }
}
