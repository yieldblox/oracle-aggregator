use crate::types::{Asset, OracleConfig, PriceData};
use soroban_sdk::{contractclient, Address, Env, Vec};
#[contractclient(name = "OracleAggregator")]
pub trait OracleAggregatorTrait {
    fn initialize(
        e: Env,
        admin: Address,
        oracles: Vec<Address>,
        oracle_configs: Vec<OracleConfig>,
        decimals: u32,
        base: Asset,
        outlier_threshold: u32,
    );

    fn base(e: Env) -> Asset;

    fn decimals(e: Env) -> u32;

    fn assets(e: Env) -> Vec<Asset>;

    fn price(e: Env, asset: Asset, timestamp: u64) -> Option<PriceData>;

    fn last_price(e: Env, asset: Asset) -> Option<PriceData>;

    fn prices(e: Env, asset: Asset, records: u32) -> Option<Vec<PriceData>>;

    fn remove_oracle(e: Env, oracle: Address);
}
