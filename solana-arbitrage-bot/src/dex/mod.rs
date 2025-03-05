mod serum;
mod orca;
mod raydium;
mod jupiter;
mod openbook;

pub use serum::*;
pub use orca::*;
pub use raydium::*;
pub use jupiter::*;
pub use openbook::*;

use {
    crate::types::common::{ArbitrageError, MarketState, TokenPair},
    solana_sdk::pubkey::Pubkey,
    async_trait::async_trait,
};

#[async_trait]
pub trait DexInterface {
    fn name(&self) -> &'static str;
    async fn get_market_state(&self, market: &Pubkey) -> Result<MarketState, ArbitrageError>;
    async fn get_best_price(&self, market: &Pubkey) -> Result<(f64, f64), ArbitrageError>; // (bid, ask)
    async fn get_liquidity(&self, market: &Pubkey) -> Result<u64, ArbitrageError>;
    async fn create_swap_instruction(&self, market: &Pubkey, amount: u64, is_buy: bool) -> Result<Vec<u8>, ArbitrageError>;
    async fn estimate_price_impact(&self, market: &Pubkey, amount: u64, is_buy: bool) -> Result<f64, ArbitrageError>;
}

pub struct DexRegistry {
    serum: SerumDex,
    orca: OrcaDex,
    raydium: RaydiumDex,
    jupiter: JupiterDex,
    openbook: OpenbookDex,
}

impl DexRegistry {
    pub fn new() -> Self {
        Self {
            serum: SerumDex::new(),
            orca: OrcaDex::new(),
            raydium: RaydiumDex::new(),
            jupiter: JupiterDex::new(),
            openbook: OpenbookDex::new(),
        }
    }

    pub async fn get_best_execution_venue(
        &self,
        token_pair: &TokenPair,
        amount: u64,
        is_buy: bool,
    ) -> Result<(&dyn DexInterface, f64), ArbitrageError> {
        let mut best_price = f64::MAX;
        let mut best_dex: Option<&dyn DexInterface> = None;

        // Check all DEXes for best price
        let dexes: Vec<&dyn DexInterface> = vec![
            &self.serum,
            &self.orca,
            &self.raydium,
            &self.jupiter,
            &self.openbook,
        ];

        for dex in dexes {
            if let Ok(market) = self.find_market(dex, token_pair) {
                if let Ok((bid, ask)) = dex.get_best_price(&market).await {
                    let price = if is_buy { ask } else { bid };
                    if (is_buy && price < best_price) || (!is_buy && price > best_price) {
                        // Check if there's enough liquidity
                        if let Ok(liquidity) = dex.get_liquidity(&market).await {
                            if liquidity >= amount {
                                best_price = price;
                                best_dex = Some(dex);
                            }
                        }
                    }
                }
            }
        }

        best_dex
            .map(|dex| (dex, best_price))
            .ok_or_else(|| ArbitrageError::MarketError("No suitable execution venue found".to_string()))
    }

    pub async fn get_cross_dex_opportunities(
        &self,
        token_pair: &TokenPair,
        min_profit_percentage: f64,
    ) -> Result<Vec<CrossDexOpportunity>, ArbitrageError> {
        let mut opportunities = Vec::new();

        // Get all markets for the token pair
        let markets = self.get_all_markets(token_pair)?;

        // Compare prices across all DEXes
        for i in 0..markets.len() {
            for j in (i + 1)..markets.len() {
                let market1 = &markets[i];
                let market2 = &markets[j];

                if let Ok((profit, direction)) = self.calculate_cross_dex_profit(market1, market2).await {
                    if profit >= min_profit_percentage {
                        opportunities.push(CrossDexOpportunity {
                            source_market: market1.clone(),
                            target_market: market2.clone(),
                            profit_percentage: profit,
                            direction,
                        });
                    }
                }
            }
        }

        Ok(opportunities)
    }

    async fn calculate_cross_dex_profit(
        &self,
        market1: &MarketInfo,
        market2: &MarketInfo,
    ) -> Result<(f64, TradeDirection), ArbitrageError> {
        let (bid1, ask1) = market1.dex.get_best_price(&market1.address).await?;
        let (bid2, ask2) = market2.dex.get_best_price(&market2.address).await?;

        // Calculate profit in both directions
        let profit1 = (bid2 / ask1 - 1.0) * 100.0; // Buy on market1, sell on market2
        let profit2 = (bid1 / ask2 - 1.0) * 100.0; // Buy on market2, sell on market1

        if profit1 > profit2 {
            Ok((profit1, TradeDirection::Market1ToMarket2))
        } else {
            Ok((profit2, TradeDirection::Market2ToMarket1))
        }
    }

    fn find_market(&self, dex: &dyn DexInterface, token_pair: &TokenPair) -> Result<Pubkey, ArbitrageError> {
        // Implementation would look up the market address for the given token pair on the specific DEX
        unimplemented!("Market lookup not implemented")
    }

    fn get_all_markets(&self, token_pair: &TokenPair) -> Result<Vec<MarketInfo>, ArbitrageError> {
        // Implementation would return all markets across DEXes for the given token pair
        unimplemented!("Market collection not implemented")
    }
}

#[derive(Clone)]
pub struct MarketInfo {
    pub address: Pubkey,
    pub dex: Box<dyn DexInterface>,
}

#[derive(Debug)]
pub struct CrossDexOpportunity {
    pub source_market: MarketInfo,
    pub target_market: MarketInfo,
    pub profit_percentage: f64,
    pub direction: TradeDirection,
}

#[derive(Debug)]
pub enum TradeDirection {
    Market1ToMarket2,
    Market2ToMarket1,
}
