use soroban_sdk::{
    unwrap::UnwrapOptimized, Address, Env, IntoVal, Map, Symbol, TryFromVal, Val, Vec,
};

use crate::types::{Asset, OracleConfig, PriceData};

const ADMIN_KEY: &str = "Admin";
const IS_INIT_KEY: &str = "IsInit";
const ORACLES_KEY: &str = "Oracles";
const OUTLIER_THRESHOLD_KEY: &str = "OutlierThreshold";
const BASE_KEY: &str = "Base";
const DECIMALS_KEY: &str = "Decimals";
const ASSET_TO_ORACLE_KEY: &str = "AssetToOracle";

const CIRCUIT_BREAKER_KEY: &str = "CircuitBreaker";
const VELOCITY_THRESHOLD_KEY: &str = "VelocityThreshold";
const STATUS_KEY: &str = "Status";
const TIMEOUT_KEY: &str = "Timeout";
const CIRCUIT_BREAKER_TIMEOUT_KEY: &str = "CircuitBreakerTimeout";

const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5 seconds per ledger on average
const LEDGER_THRESHOLD_SHARED: u32 = 14 * ONE_DAY_LEDGERS;
const LEDGER_BUMP_SHARED: u32 = 15 * ONE_DAY_LEDGERS;

//********** Storage Utils **********//

/// Bump the instance lifetime by the defined amount
pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
}

/// Fetch an entry in persistent storage that has a default value if it doesn't exist
fn get_persistent_default<K: IntoVal<Env, Val>, V: TryFromVal<Env, Val>>(
    e: &Env,
    key: &K,
    default: V,
    bump_threshold: u32,
    bump_amount: u32,
) -> V {
    if let Some(result) = e.storage().persistent().get::<K, V>(key) {
        e.storage()
            .persistent()
            .extend_ttl(key, bump_threshold, bump_amount);
        result
    } else {
        default
    }
}

/********** Instance **********/

/// Check if the contract has been initialized
pub fn get_is_init(e: &Env) -> bool {
    e.storage().instance().has(&Symbol::new(e, IS_INIT_KEY))
}

/// Set the contract as initialized
pub fn set_is_init(e: &Env) {
    e.storage()
        .instance()
        .set::<Symbol, bool>(&Symbol::new(e, IS_INIT_KEY), &true);
}

/// Get the admin address
pub fn get_admin(e: &Env) -> Address {
    e.storage()
        .instance()
        .get::<Symbol, Address>(&Symbol::new(e, ADMIN_KEY))
        .unwrap()
}

/// Set the admin address
pub fn set_admin(e: &Env, admin: &Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&Symbol::new(e, ADMIN_KEY), &admin);
}

/********** Persistent **********/
pub fn set_oracles(e: &Env, oracles: &Vec<Address>) {
    e.storage()
        .persistent()
        .set::<Symbol, Vec<Address>>(&Symbol::new(e, ORACLES_KEY), oracles);
}

pub fn get_oracles(e: &Env) -> Vec<Address> {
    e.storage().persistent().extend_ttl(
        &Symbol::new(e, ORACLES_KEY),
        LEDGER_THRESHOLD_SHARED,
        LEDGER_BUMP_SHARED,
    );
    e.storage()
        .persistent()
        .get::<Symbol, Vec<Address>>(&Symbol::new(e, ORACLES_KEY))
        .unwrap_optimized()
}

pub fn has_oracle(e: &Env, oracle: &Address) -> bool {
    let oracles = get_oracles(e);
    oracles.contains(oracle)
}

pub fn remove_oracle(e: &Env, oracle: &Address) {
    let mut oracles = get_oracles(e);
    for (index, address) in oracles.iter().enumerate() {
        if address == *oracle {
            oracles.remove(index as u32);
            break;
        }
    }
    set_oracles(e, &oracles);
}

pub fn set_oracle_config(e: &Env, oracle: &Address, config: &OracleConfig) {
    e.storage()
        .persistent()
        .set::<Address, OracleConfig>(&oracle, config);
}

pub fn get_oracle_config(e: &Env, oracle: &Address) -> OracleConfig {
    e.storage()
        .persistent()
        .extend_ttl(oracle, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
    e.storage().persistent().get(oracle).unwrap_optimized()
}

pub fn has_oracle_config(e: &Env, oracle: &Address) -> bool {
    e.storage().persistent().has(&oracle)
}

pub fn remove_oracle_config(e: &Env, oracle: &Address) {
    e.storage().persistent().remove(&oracle);
}

pub fn set_outlier_threshold(e: &Env, threshold: &u32) {
    e.storage()
        .persistent()
        .set::<Symbol, u32>(&Symbol::new(&e, OUTLIER_THRESHOLD_KEY), threshold);
}

pub fn get_outlier_threshold(e: &Env) -> u32 {
    get_persistent_default(
        e,
        &Symbol::new(e, "Oracles"),
        0,
        LEDGER_THRESHOLD_SHARED,
        LEDGER_BUMP_SHARED,
    )
}

pub fn set_base(e: &Env, base: &Asset) {
    e.storage()
        .persistent()
        .set::<Symbol, Asset>(&Symbol::new(e, BASE_KEY), base);
}

pub fn get_base(e: &Env) -> Asset {
    e.storage().persistent().extend_ttl(
        &Symbol::new(e, BASE_KEY),
        LEDGER_THRESHOLD_SHARED,
        LEDGER_BUMP_SHARED,
    );
    e.storage()
        .persistent()
        .get::<Symbol, Asset>(&Symbol::new(e, BASE_KEY))
        .unwrap()
}

pub fn set_decimals(e: &Env, decimals: &u32) {
    e.storage()
        .persistent()
        .set::<Symbol, u32>(&Symbol::new(e, DECIMALS_KEY), decimals);
}

pub fn get_decimals(e: &Env) -> u32 {
    e.storage().persistent().extend_ttl(
        &Symbol::new(e, DECIMALS_KEY),
        LEDGER_THRESHOLD_SHARED,
        LEDGER_BUMP_SHARED,
    );
    e.storage()
        .persistent()
        .get::<Symbol, u32>(&Symbol::new(e, DECIMALS_KEY))
        .unwrap()
}

pub fn set_asset_oracle_map(e: &Env, asset_to_oracles: &Map<Asset, Vec<Address>>) {
    e.storage()
        .persistent()
        .set::<Symbol, Map<Asset, Vec<Address>>>(
            &Symbol::new(&e, ASSET_TO_ORACLE_KEY),
            asset_to_oracles,
        );
}
pub fn get_asset_oracle_map(e: &Env) -> Map<Asset, Vec<Address>> {
    e.storage().persistent().extend_ttl(
        &Symbol::new(e, ASSET_TO_ORACLE_KEY),
        LEDGER_THRESHOLD_SHARED,
        LEDGER_BUMP_SHARED,
    );
    e.storage()
        .persistent()
        .get::<Symbol, Map<Asset, Vec<Address>>>(&Symbol::new(&e, ASSET_TO_ORACLE_KEY))
        .unwrap_optimized()
}

pub fn get_asset_oracles(e: &Env, asset: &Asset) -> Vec<Address> {
    let asset_to_oracles = e
        .storage()
        .persistent()
        .get::<Symbol, Map<Asset, Vec<Address>>>(&Symbol::new(&e, ASSET_TO_ORACLE_KEY));
    e.storage().persistent().extend_ttl(
        &Symbol::new(e, ASSET_TO_ORACLE_KEY),
        LEDGER_THRESHOLD_SHARED,
        LEDGER_BUMP_SHARED,
    );
    match asset_to_oracles {
        Some(asset_to_oracles) => asset_to_oracles.get(asset.clone()).unwrap(),
        None => Vec::new(&e),
    }
}

pub fn set_circuit_breaker(e: &Env) {
    e.storage()
        .persistent()
        .set::<Symbol, bool>(&Symbol::new(&e, CIRCUIT_BREAKER_KEY), &true);
}

pub fn has_circuit_breaker(e: &Env) -> bool {
    e.storage()
        .persistent()
        .has::<Symbol>(&Symbol::new(&e, CIRCUIT_BREAKER_KEY))
}

pub fn set_velocity_threshold(e: &Env, threshold: &u32) {
    e.storage()
        .persistent()
        .set::<Symbol, u32>(&Symbol::new(&e, VELOCITY_THRESHOLD_KEY), threshold);
}

pub fn get_velocity_threshold(e: &Env) -> u32 {
    get_persistent_default(
        e,
        &Symbol::new(e, VELOCITY_THRESHOLD_KEY),
        0,
        LEDGER_THRESHOLD_SHARED,
        LEDGER_BUMP_SHARED,
    )
}

pub fn set_last_price(e: &Env, asset: &Asset, price: &PriceData) {
    e.storage()
        .persistent()
        .set::<Asset, PriceData>(asset, price);
}

pub fn get_last_price(e: &Env, asset: &Asset) -> Option<PriceData> {
    e.storage()
        .persistent()
        .extend_ttl(asset, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
    e.storage().persistent().get::<Asset, PriceData>(asset)
}

pub fn set_circuit_breaker_status(e: &Env, status: &bool) {
    e.storage()
        .persistent()
        .set::<Symbol, bool>(&Symbol::new(&e, STATUS_KEY), status);
}

pub fn get_circuit_breaker_status(e: &Env) -> bool {
    get_persistent_default(
        &e,
        &Symbol::new(&e, STATUS_KEY),
        false,
        LEDGER_THRESHOLD_SHARED,
        LEDGER_BUMP_SHARED,
    )
}

pub fn set_timeout(e: &Env, timeout: &u64) {
    e.storage()
        .persistent()
        .set::<Symbol, u64>(&Symbol::new(&e, TIMEOUT_KEY), timeout);
}

pub fn get_timeout(e: &Env) -> u64 {
    e.storage().persistent().extend_ttl(
        &Symbol::new(&e, TIMEOUT_KEY),
        LEDGER_THRESHOLD_SHARED,
        LEDGER_BUMP_SHARED,
    );
    e.storage()
        .persistent()
        .get::<Symbol, u64>(&Symbol::new(&e, CIRCUIT_BREAKER_TIMEOUT_KEY))
        .unwrap()
}

pub fn set_circuit_breaker_timeout(e: &Env, timeout: &u64) {
    e.storage()
        .persistent()
        .set::<Symbol, u64>(&Symbol::new(&e, CIRCUIT_BREAKER_TIMEOUT_KEY), timeout);
}

pub fn get_circuit_breaker_timeout(e: &Env) -> u64 {
    e.storage().persistent().extend_ttl(
        &Symbol::new(&e, CIRCUIT_BREAKER_TIMEOUT_KEY),
        LEDGER_THRESHOLD_SHARED,
        LEDGER_BUMP_SHARED,
    );
    e.storage()
        .persistent()
        .get::<Symbol, u64>(&Symbol::new(&e, CIRCUIT_BREAKER_TIMEOUT_KEY))
        .unwrap()
}
