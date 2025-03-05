mod arbitrage_engine;
mod profit_calculator;
mod transaction_builder;

pub use arbitrage_engine::*;
pub use profit_calculator::*;
pub use transaction_builder::*;

use crate::types::common::{ArbitrageError, ArbitrageOpportunity, ExecutionResult};
use solana_sdk::pubkey::Pubkey;

pub trait ArbitrageStrategy {
    fn name(&self) -> &'static str;
    
    fn analyze(&self, markets: &[Pubkey]) -> Result<Vec<ArbitrageOpportunity>, ArbitrageError>;
    
    fn execute(&self, opportunity: &ArbitrageOpportunity) -> Result<ExecutionResult, ArbitrageError>;
    
    fn validate(&self, opportunity: &ArbitrageOpportunity) -> Result<bool, ArbitrageError>;
}

pub trait ProfitCalculator {
    fn calculate_profit(
        &self,
        opportunity: &ArbitrageOpportunity,
        include_fees: bool,
    ) -> Result<f64, ArbitrageError>;
    
    fn estimate_gas_costs(&self, opportunity: &ArbitrageOpportunity) -> Result<u64, ArbitrageError>;
}

pub trait TransactionBuilder {
    fn build_transaction(
        &self,
        opportunity: &ArbitrageOpportunity,
    ) -> Result<Vec<u8>, ArbitrageError>;
    
    fn simulate_transaction(
        &self,
        transaction: &[u8],
    ) -> Result<bool, ArbitrageError>;
}
