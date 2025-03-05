use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use crate::types::common::{SecurityLevel, ArbitrageError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub network: NetworkSettings,
    pub trading: TradingSettings,
    pub security: SecuritySettings,
    pub monitoring: MonitoringSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    pub rpc_endpoints: Vec<String>,
    pub ws_endpoints: Vec<String>,
    pub backup_nodes: Vec<String>,
    pub max_retries: u32,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSettings {
    pub markets: MarketSettings,
    pub execution: ExecutionSettings,
    pub risk: RiskSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSettings {
    pub whitelisted_markets: Vec<String>,
    pub whitelisted_tokens: Vec<String>,
    pub blacklisted_markets: Vec<String>,
    pub min_liquidity: u64,
    pub max_spread: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSettings {
    pub max_concurrent_trades: u32,
    pub min_profit_threshold: f64,
    pub max_position_size: u64,
    pub flash_loan_enabled: bool,
    pub flash_loan_sources: Vec<String>,
    pub execution_strategies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSettings {
    pub max_loss_threshold: f64,
    pub daily_volume_limit: u64,
    pub position_timeout: u64,
    pub slippage_tolerance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub level: SecurityLevel,
    pub mev_protection: MevProtectionSettings,
    pub quantum_security: QuantumSecuritySettings,
    pub transaction_guards: TransactionGuardSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevProtectionSettings {
    pub enabled: bool,
    pub protection_level: u32,
    pub sandwich_detection: bool,
    pub frontrunning_detection: bool,
    pub backrunning_detection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumSecuritySettings {
    pub enabled: bool,
    pub encryption_level: String,
    pub key_rotation_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionGuardSettings {
    pub signature_verification: bool,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub require_confirmations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSettings {
    pub log_level: String,
    pub metrics_enabled: bool,
    pub alert_endpoints: Vec<String>,
    pub performance_tracking: bool,
}

impl Settings {
    pub fn load() -> Result<Self, ArbitrageError> {
        // Load from environment or config file
        let settings = Self::default();
        
        // Validate settings
        settings.validate()?;
        
        Ok(settings)
    }

    fn validate(&self) -> Result<(), ArbitrageError> {
        // Validate network settings
        if self.network.rpc_endpoints.is_empty() {
            return Err(ArbitrageError::ConfigError("No RPC endpoints configured".to_string()));
        }

        // Validate trading settings
        if self.trading.execution.min_profit_threshold <= 0.0 {
            return Err(ArbitrageError::ConfigError("Invalid profit threshold".to_string()));
        }

        // Validate security settings
        if self.security.mev_protection.enabled && self.security.mev_protection.protection_level == 0 {
            return Err(ArbitrageError::ConfigError("Invalid MEV protection level".to_string()));
        }

        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            network: NetworkSettings {
                rpc_endpoints: vec![std::env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string())],
                ws_endpoints: vec![],
                backup_nodes: vec![],
                max_retries: 3,
                timeout_ms: 30000,
            },
            trading: TradingSettings {
                markets: MarketSettings {
                    whitelisted_markets: vec![],
                    whitelisted_tokens: vec![],
                    blacklisted_markets: vec![],
                    min_liquidity: 1000000,
                    max_spread: 0.05,
                },
                execution: ExecutionSettings {
                    max_concurrent_trades: 3,
                    min_profit_threshold: 0.01,
                    max_position_size: 1000000000,
                    flash_loan_enabled: true,
                    flash_loan_sources: vec!["solend".to_string(), "port".to_string()],
                    execution_strategies: vec!["jit".to_string(), "flash_loan".to_string()],
                },
                risk: RiskSettings {
                    max_loss_threshold: -0.02,
                    daily_volume_limit: 1000000000000,
                    position_timeout: 30000,
                    slippage_tolerance: 0.01,
                },
            },
            security: SecuritySettings {
                level: SecurityLevel::High,
                mev_protection: MevProtectionSettings {
                    enabled: true,
                    protection_level: 2,
                    sandwich_detection: true,
                    frontrunning_detection: true,
                    backrunning_detection: true,
                },
                quantum_security: QuantumSecuritySettings {
                    enabled: true,
                    encryption_level: "AES-256".to_string(),
                    key_rotation_interval: 3600,
                },
                transaction_guards: TransactionGuardSettings {
                    signature_verification: true,
                    timeout_ms: 5000,
                    max_retries: 3,
                    require_confirmations: 1,
                },
            },
            monitoring: MonitoringSettings {
                log_level: "info".to_string(),
                metrics_enabled: true,
                alert_endpoints: vec![],
                performance_tracking: true,
            },
        }
    }
}

// Helper functions for parsing settings from environment variables
pub fn parse_pubkey_list(input: &str) -> Result<Vec<Pubkey>, ArbitrageError> {
    input
        .split(',')
        .map(|s| Pubkey::from_str(s.trim()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| ArbitrageError::ConfigError(format!("Invalid pubkey in list: {}", e)))
}

pub fn parse_market_pairs(input: &str) -> Result<HashMap<Pubkey, Pubkey>, ArbitrageError> {
    let pairs: Result<HashMap<_, _>, _> = input
        .split(';')
        .map(|pair| {
            let tokens: Vec<&str> = pair.split(',').collect();
            if tokens.len() != 2 {
                return Err(ArbitrageError::ConfigError("Invalid market pair format".to_string()));
            }
            
            let market = Pubkey::from_str(tokens[0].trim())
                .map_err(|e| ArbitrageError::ConfigError(format!("Invalid market pubkey: {}", e)))?;
            let token = Pubkey::from_str(tokens[1].trim())
                .map_err(|e| ArbitrageError::ConfigError(format!("Invalid token pubkey: {}", e)))?;
                
            Ok((market, token))
        })
        .collect();
    
    pairs
}
