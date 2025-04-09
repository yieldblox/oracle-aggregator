use soroban_sdk::{contractclient, contracttype, Address, Env, Symbol};

/// Price data for an asset at a specific timestamp
#[contracttype]
#[derive(Clone, Debug)]
pub struct PriceData {
    pub price: i128,
    pub timestamp: u64,
}

/// Asset type
#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum Asset {
    Stellar(Address),
    Other(Symbol),
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct AssetConfig {
    /// The asset used to fetch prices from the source oracle
    pub asset: Asset,
    /// The index of the oracle used for this asset
    pub oracle_index: u32,
    /// The maximum deviation allowed for a stable price, as a percentage with 0 decimals
    /// (e.g 5 => 5%). If this is 0, the oracle will just fetch the last price within the
    /// resolution time.
    pub max_dev: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct OracleConfig {
    /// The oracle id
    pub address: Address,
    /// The index of the oracle
    pub index: u32,
    /// The resolution of the oracle, in seconds
    pub resolution: u32,
    /// The decimals of the oracle
    pub decimals: u32,
}

#[allow(dead_code)]
#[contractclient(name = "PriceFeedClient")]
pub trait PriceFeed {
    /// Get the resolution of the oracle
    fn resolution(env: Env) -> u32;
    /// Get the decimals of the oracle
    fn decimals(env: Env) -> u32;
    /// Get the last timestamp the oracle was updated
    fn last_timestamp(env: Env) -> u64;
    /// Get the price for an asset at a specific timestamp
    fn price(env: Env, asset: &Asset, timestamp: &u64) -> Option<PriceData>;
}
