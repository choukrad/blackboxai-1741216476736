mod jit_liquidity;
mod flash_loan;
mod front_running;

pub use jit_liquidity::*;
pub use flash_loan::*;
pub use front_running::*;

use crate::{
    types::common::{ArbitrageError, ArbitrageOpportunity},
    core::ArbitrageStrategy,
};

// Strategy factory for creating different arbitrage strategies
pub struct StrategyFactory;

impl StrategyFactory {
    pub fn create_strategy(strategy_type: &str) -> Result<Box<dyn ArbitrageStrategy>, ArbitrageError> {
        match strategy_type {
            "jit" => Ok(Box::new(JitLiquidityStrategy::new())),
            "flash_loan" => Ok(Box::new(FlashLoanStrategy::new())),
            "front_running" => Ok(Box::new(FrontRunningStrategy::new())),
            _ => Err(ArbitrageError::ConfigError(format!(
                "Unknown strategy type: {}",
                strategy_type
            ))),
        }
    }
}

// Common traits and utilities for strategies
pub trait OpportunityValidator {
    fn validate_opportunity(&self, opportunity: &ArbitrageOpportunity) -> Result<bool, ArbitrageError>;
}

pub trait RiskManager {
    fn check_risk_parameters(&self, opportunity: &ArbitrageOpportunity) -> Result<bool, ArbitrageError>;
}

pub trait MarketAnalyzer {
    fn analyze_market_conditions(&self) -> Result<bool, ArbitrageError>;
}
