mod config;
mod core;
mod strategies;
mod types;

use {
    crate::{
        config::Settings,
        core::{ArbitrageEngine, ArbitrageStrategy},
        strategies::StrategyFactory,
        types::common::ArbitrageError,
    },
    solana_sdk::{
        signature::Keypair,
        signer::Signer,
    },
    std::{str::FromStr, env},
    tokio,
};

#[tokio::main]
async fn main() -> Result<(), ArbitrageError> {
    // Initialize logging
    env_logger::init();
    log::info!("Starting Solana Arbitrage Bot...");

    // Load configuration
    let settings = Settings::load()?;
    log::info!("Configuration loaded successfully");

    // Load keypair
    let keypair = load_keypair()?;
    log::info!("Loaded keypair: {}", keypair.pubkey());

    // Initialize arbitrage engine
    let engine = ArbitrageEngine::new(settings.clone(), keypair)?;
    log::info!("Arbitrage engine initialized");

    // Initialize strategies
    let strategies = initialize_strategies(&settings)?;
    log::info!("Initialized {} strategies", strategies.len());

    // Start the arbitrage bot
    log::info!("Starting arbitrage operations...");
    engine.start().await?;

    Ok(())
}

fn load_keypair() -> Result<Keypair, ArbitrageError> {
    let keypair_path = env::var("KEYPAIR_PATH")
        .map_err(|_| ArbitrageError::ConfigError("KEYPAIR_PATH not set".to_string()))?;

    let keypair_bytes = std::fs::read_to_string(&keypair_path)
        .map_err(|e| ArbitrageError::ConfigError(format!("Failed to read keypair file: {}", e)))?;

    let keypair_bytes: Vec<u8> = keypair_bytes
        .trim()
        .split(',')
        .map(|s| u8::from_str(s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| ArbitrageError::ConfigError(format!("Invalid keypair format: {}", e)))?;

    Keypair::from_bytes(&keypair_bytes)
        .map_err(|e| ArbitrageError::ConfigError(format!("Invalid keypair: {}", e)))
}

fn initialize_strategies(settings: &Settings) -> Result<Vec<Box<dyn ArbitrageStrategy>>, ArbitrageError> {
    let mut strategies = Vec::new();

    // Initialize JIT Liquidity Strategy
    if settings.trading.execution.execution_strategies.contains(&"jit".to_string()) {
        strategies.push(StrategyFactory::create_strategy("jit")?);
    }

    // Initialize Flash Loan Strategy
    if settings.trading.execution.execution_strategies.contains(&"flash_loan".to_string()) {
        strategies.push(StrategyFactory::create_strategy("flash_loan")?);
    }

    // Initialize Front Running Strategy
    if settings.trading.execution.execution_strategies.contains(&"front_running".to_string()) {
        strategies.push(StrategyFactory::create_strategy("front_running")?);
    }

    Ok(strategies)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_strategy_initialization() {
        let mut settings = Settings::default();
        settings.trading.execution.execution_strategies = vec![
            "jit".to_string(),
            "flash_loan".to_string(),
            "front_running".to_string(),
        ];

        let strategies = initialize_strategies(&settings).unwrap();
        assert_eq!(strategies.len(), 3);
    }

    #[test]
    fn test_keypair_loading() {
        // This test requires a valid keypair file to be present
        std::env::set_var("KEYPAIR_PATH", "test_keypair.json");
        assert!(load_keypair().is_err()); // Should fail if test file doesn't exist
    }
}
