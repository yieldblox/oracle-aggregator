use soroban_sdk::{contracttype, Address, Symbol, Vec};

/// Price data for an asset at a specific timestamp
#[contracttype]
#[derive(Clone)]
pub struct PriceData {
    pub price: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, PartialEq, Eq)]
pub enum Asset {
    Stellar(Address),
    Other(Symbol),
}

#[contracttype]
#[derive(Clone)]
pub struct OracleConfig {
    pub decimals: u32,
    pub supported_assets: Vec<Asset>,
}
