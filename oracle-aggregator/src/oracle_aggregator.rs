use crate::types::{Asset, OracleConfig, PriceData, SettingsConfig};
use soroban_sdk::{contractclient, Address, Env, Vec};
#[contractclient(name = "OracleAggregator")]
pub trait OracleAggregatorTrait {
    fn initialize(e: Env, admin: Address, config: SettingsConfig);

    fn base(e: Env) -> Asset;

    fn decimals(e: Env) -> u32;

    fn assets(e: Env) -> Vec<Asset>;

    fn asset_config(e: Env, asset: Asset) -> OracleConfig;

    fn last_price(e: Env, asset: Asset) -> Option<PriceData>;

    fn remove_asset(e: Env, asset: Asset);
}
