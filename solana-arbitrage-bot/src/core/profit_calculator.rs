use {
    crate::{
        types::common::{ArbitrageError, ArbitrageOpportunity, MarketState, TradeStep},
        config::Settings,
    },
    solana_sdk::pubkey::Pubkey,
    std::sync::Arc,
};

pub struct ProfitCalculator {
    settings: Arc<Settings>,
}

impl ProfitCalculator {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings: Arc::new(settings),
        }
    }

    pub fn calculate_total_profit(
        &self,
        opportunity: &ArbitrageOpportunity,
        market_states: &[MarketState],
    ) -> Result<f64, ArbitrageError> {
        let mut total_profit = 0.0;
        let mut current_amount = opportunity.required_amount as f64;

        // Calculate profit for each step in the arbitrage route
        for step in &opportunity.route {
            let (profit, new_amount) = self.calculate_step_profit(step, current_amount, market_states)?;
            total_profit += profit;
            current_amount = new_amount;
        }

        // Subtract fees and costs
        let fees = self.calculate_total_fees(opportunity)?;
        let gas_costs = self.estimate_gas_costs(opportunity)?;
        
        total_profit -= (fees + gas_costs as f64);

        Ok(total_profit)
    }

    pub fn calculate_step_profit(
        &self,
        step: &TradeStep,
        input_amount: f64,
        market_states: &[MarketState],
    ) -> Result<(f64, f64), ArbitrageError> {
        let market_state = self.get_market_state(&step.market, market_states)?;
        
        let (profit, output_amount) = match step.side {
            crate::types::common::TradeSide::Buy => {
                self.calculate_buy_profit(input_amount, step.price, market_state)?
            }
            crate::types::common::TradeSide::Sell => {
                self.calculate_sell_profit(input_amount, step.price, market_state)?
            }
        };

        Ok((profit, output_amount))
    }

    pub fn estimate_gas_costs(&self, opportunity: &ArbitrageOpportunity) -> Result<u64, ArbitrageError> {
        // Base cost for transaction
        let mut total_cost = 5000;

        // Add cost for each instruction in the route
        total_cost += opportunity.route.len() as u64 * 1000;

        // Add extra cost if using flash loans
        if opportunity.route.len() > 2 {
            total_cost += 2000; // Additional cost for flash loan
        }

        // Add cost for complex computations
        if self.settings.security.mev_protection.enabled {
            total_cost += 1000; // MEV protection overhead
        }

        Ok(total_cost)
    }

    pub fn calculate_total_fees(&self, opportunity: &ArbitrageOpportunity) -> Result<f64, ArbigrageError> {
        let mut total_fees = 0.0;

        // Trading fees
        for step in &opportunity.route {
            total_fees += self.calculate_trading_fee(step)?;
        }

        // Flash loan fees if applicable
        if opportunity.route.len() > 2 {
            total_fees += self.calculate_flash_loan_fee(opportunity.required_amount)?;
        }

        // Protocol fees
        total_fees += self.calculate_protocol_fees(opportunity)?;

        Ok(total_fees)
    }

    fn calculate_buy_profit(
        &self,
        input_amount: f64,
        price: f64,
        market_state: &MarketState,
    ) -> Result<(f64, f64), ArbitrageError> {
        // Calculate slippage based on order size
        let slippage = self.calculate_slippage(input_amount, market_state)?;
        let effective_price = price * (1.0 + slippage);

        // Calculate output amount after fees
        let base_output = input_amount / effective_price;
        let fee_rate = self.get_market_fee_rate(market_state);
        let output_after_fees = base_output * (1.0 - fee_rate);

        // Calculate profit/loss
        let profit = output_after_fees * market_state.best_bid - input_amount;

        Ok((profit, output_after_fees))
    }

    fn calculate_sell_profit(
        &self,
        input_amount: f64,
        price: f64,
        market_state: &MarketState,
    ) -> Result<(f64, f64), ArbitrageError> {
        // Calculate slippage based on order size
        let slippage = self.calculate_slippage(input_amount, market_state)?;
        let effective_price = price * (1.0 - slippage);

        // Calculate output amount after fees
        let base_output = input_amount * effective_price;
        let fee_rate = self.get_market_fee_rate(market_state);
        let output_after_fees = base_output * (1.0 - fee_rate);

        // Calculate profit/loss
        let profit = output_after_fees - input_amount * market_state.best_ask;

        Ok((profit, output_after_fees))
    }

    fn calculate_slippage(
        &self,
        amount: f64,
        market_state: &MarketState,
    ) -> Result<f64, ArbitrageError> {
        // Basic linear slippage model
        // For more accuracy, implement a more sophisticated model based on order book depth
        let base_liquidity = 100000.0; // Base liquidity threshold
        let slippage_factor = 0.1; // Slippage sensitivity

        let normalized_amount = amount / base_liquidity;
        let slippage = normalized_amount * slippage_factor;

        // Cap maximum slippage
        let max_slippage = self.settings.trading.risk.slippage_tolerance;
        Ok(slippage.min(max_slippage))
    }

    fn calculate_trading_fee(&self, step: &TradeStep) -> Result<f64, ArbitrageError> {
        // Standard percentage fee
        let fee_rate = 0.003; // 0.3% fee
        Ok(step.amount as f64 * fee_rate)
    }

    fn calculate_flash_loan_fee(&self, amount: u64) -> Result<f64, ArbitrageError> {
        // Standard flash loan fee (0.09%)
        let fee_rate = 0.0009;
        Ok(amount as f64 * fee_rate)
    }

    fn calculate_protocol_fees(&self, opportunity: &ArbitrageOpportunity) -> Result<f64, ArbitrageError> {
        // Network fees and protocol-specific fees
        let base_fee = 0.001; // 0.1% base fee
        Ok(opportunity.required_amount as f64 * base_fee)
    }

    fn get_market_state<'a>(
        &self,
        market: &Pubkey,
        market_states: &'a [MarketState],
    ) -> Result<&'a MarketState, ArbitrageError> {
        market_states
            .iter()
            .find(|state| state.market_address == *market)
            .ok_or_else(|| ArbitrageError::MarketError("Market state not found".to_string()))
    }

    fn get_market_fee_rate(&self, market_state: &MarketState) -> f64 {
        // Could be customized based on market or token pair
        0.003 // Default 0.3% fee
    }

    pub fn is_profitable(
        &self,
        opportunity: &ArbitrageOpportunity,
        market_states: &[MarketState],
    ) -> Result<bool, ArbitrageError> {
        let total_profit = self.calculate_total_profit(opportunity, market_states)?;
        let min_profit_threshold = self.settings.trading.execution.min_profit_threshold;

        // Check if profit meets minimum threshold
        if total_profit < min_profit_threshold {
            return Ok(false);
        }

        // Validate against risk settings
        if !self.validate_risk_parameters(opportunity, total_profit)? {
            return Ok(false);
        }

        Ok(true)
    }

    fn validate_risk_parameters(
        &self,
        opportunity: &ArbitrageOpportunity,
        profit: f64,
    ) -> Result<bool, ArbitrageError> {
        // Check maximum position size
        if opportunity.required_amount > self.settings.trading.execution.max_position_size {
            return Ok(false);
        }

        // Check profit vs risk ratio
        let risk_ratio = profit / opportunity.required_amount as f64;
        if risk_ratio < self.settings.trading.risk.max_loss_threshold {
            return Ok(false);
        }

        Ok(true)
    }
}
