use sep_40_oracle::Asset;
use soroban_sdk::{contracttype, Address, Vec};
#[contracttype]
#[derive(Clone)]
pub struct OracleConfig {
    pub oracle_id: Address,
    pub decimals: u32,
    pub resolution: u64,
    // The asset to be used when fetching the price
    pub asset: Asset,
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
