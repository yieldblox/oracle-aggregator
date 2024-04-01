use soroban_sdk::{contracttype, Address, Symbol, Vec};

/// Price data for an asset at a specific timestamp
#[contracttype]
#[derive(Clone)]
pub struct PriceData {
    pub price: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Asset {
    Stellar(Address),
    Other(Symbol),
}

#[contracttype]
#[derive(Clone)]
pub struct OracleConfig {
    pub oracle_id: Address,
    pub decimals: u32,
    pub resolution: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct SettingsConfig {
    pub assets: Vec<Asset>,
    pub asset_configs: Vec<OracleConfig>,
    pub decimals: u32,
    pub base: Asset,
    pub enable_circuit_breaker: bool,
    // The velocity threshold (5 decimals)
    pub circuit_breaker_threshold: u32,
    // The timeout for the circuit breaker (in seconds)
    pub circuit_breaker_timeout: u64,
}
