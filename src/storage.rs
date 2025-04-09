use crate::types::{Asset, AssetConfig, OracleConfig};
use soroban_sdk::{map, unwrap::UnwrapOptimized, vec, Address, Env, Map, Symbol, Vec};

const ADMIN_KEY: &str = "Admin";
const BASE_KEY: &str = "Base";
const BASE_ASSETS_KEY: &str = "BaseAssets";
const ASSETS_KEY: &str = "Assets";
const ORACLES_KEY: &str = "Oracles";
const DECIMALS_KEY: &str = "Decimals";
const MAX_AGE_KEY: &str = "MaxAge";

const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5 seconds per ledger on average
const LEDGER_THRESHOLD: u32 = 30 * ONE_DAY_LEDGERS;
const LEDGER_BUMP: u32 = 31 * ONE_DAY_LEDGERS;

//********** Storage Utils **********//

/// Bump the instance lifetime by the defined amount
pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD, LEDGER_BUMP);
}

/********** Instance **********/

/// Set the admin address
pub fn set_admin(e: &Env, admin: &Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&Symbol::new(e, ADMIN_KEY), &admin);
}

/// Get the admin address
pub fn get_admin(e: &Env) -> Address {
    e.storage()
        .instance()
        .get::<Symbol, Address>(&Symbol::new(e, ADMIN_KEY))
        .unwrap_optimized()
}

/// Set the max age of a price, in seconds
pub fn set_max_age(e: &Env, max_age: &u64) {
    e.storage()
        .instance()
        .set::<Symbol, u64>(&Symbol::new(e, MAX_AGE_KEY), max_age);
}

/// Set the max age of a price, in seconds
pub fn get_max_age(e: &Env) -> u64 {
    e.storage()
        .instance()
        .get::<Symbol, u64>(&Symbol::new(e, MAX_AGE_KEY))
        .unwrap_optimized()
}

/// Set the base asset for the oracle aggregator
pub fn set_base(e: &Env, base: &Asset) {
    e.storage()
        .instance()
        .set::<Symbol, Asset>(&Symbol::new(e, BASE_KEY), base);
}

/// Get the base asset for the oracle aggregator
pub fn get_base(e: &Env) -> Asset {
    e.storage()
        .instance()
        .get::<Symbol, Asset>(&Symbol::new(e, BASE_KEY))
        .unwrap()
}

/// Set the number of decimals the oracle will report prices in
pub fn set_decimals(e: &Env, decimals: &u32) {
    e.storage()
        .instance()
        .set::<Symbol, u32>(&Symbol::new(e, DECIMALS_KEY), decimals);
}

/// Get the number of decimals the oracle will report prices in
pub fn get_decimals(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get::<Symbol, u32>(&Symbol::new(e, DECIMALS_KEY))
        .unwrap()
}

/// Set a list of base assets
pub fn set_base_assets(e: &Env, assets: &Vec<Asset>) {
    e.storage()
        .instance()
        .set::<Symbol, Vec<Asset>>(&Symbol::new(e, BASE_ASSETS_KEY), assets);
}

/// Get a list of base assets
pub fn get_base_assets(e: &Env) -> Vec<Asset> {
    e.storage()
        .instance()
        .get::<Symbol, Vec<Asset>>(&Symbol::new(e, BASE_ASSETS_KEY))
        .unwrap_or(vec![&e])
}

/// Set a list of oracles
pub fn set_oracles(e: &Env, assets: &Vec<OracleConfig>) {
    e.storage()
        .instance()
        .set::<Symbol, Vec<OracleConfig>>(&Symbol::new(e, ORACLES_KEY), assets);
}

/// Get a list of oracles
pub fn get_oracles(e: &Env) -> Vec<OracleConfig> {
    e.storage()
        .instance()
        .get::<Symbol, Vec<OracleConfig>>(&Symbol::new(e, ORACLES_KEY))
        .unwrap_or(vec![&e])
}

/// Set an asset configuration
pub fn set_asset_configs(e: &Env, config: &Map<Asset, AssetConfig>) {
    e.storage()
        .instance()
        .set::<Symbol, Map<Asset, AssetConfig>>(&Symbol::new(e, ASSETS_KEY), config);
}

/// Get an asset configuration
pub fn get_asset_configs(e: &Env) -> Map<Asset, AssetConfig> {
    e.storage()
        .instance()
        .get::<Symbol, Map<Asset, AssetConfig>>(&Symbol::new(e, ASSETS_KEY))
        .unwrap_or(map![&e])
}
