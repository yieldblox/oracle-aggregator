use sep_40_oracle::Asset;
use soroban_sdk::{contracttype, Address, Env, IntoVal, Symbol, TryFromVal, Val};

const ADMIN_KEY: &str = "Admin";
const IS_INIT_KEY: &str = "IsInit";
const USDC_KEY: &str = "USDC";
const USDC_ORACLE_KEY: &str = "USDCOrcl";
const DEFAULT_ORACLE_KEY: &str = "DEFOrcl";
const BASE_KEY: &str = "Base";
const DECIMALS_KEY: &str = "Decimals";

const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5 seconds per ledger on average
const LEDGER_THRESHOLD_SHARED: u32 = 30 * ONE_DAY_LEDGERS;
const LEDGER_BUMP_SHARED: u32 = 31 * ONE_DAY_LEDGERS;

#[derive(Clone)]
#[contracttype]
pub enum AggregatorDataKey {
    AssetConfig(Asset),
    CircuitBreakerStatus(Asset),
    CircuitBreakerTimeout(Asset),
    Blocked(Asset),
}

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

/// Get the base asset
pub fn get_base(e: &Env) -> Asset {
    e.storage()
        .instance()
        .get::<Symbol, Asset>(&Symbol::new(e, BASE_KEY))
        .unwrap()
}

/// Set the base asset
pub fn set_base(e: &Env, base: &Asset) {
    e.storage()
        .instance()
        .set::<Symbol, Asset>(&Symbol::new(e, BASE_KEY), base);
}

/// Get the decimals the oracle reports in
pub fn get_decimals(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get::<Symbol, u32>(&Symbol::new(e, DECIMALS_KEY))
        .unwrap()
}

/// Set the decimals the oracle reports in
pub fn set_decimals(e: &Env, decimals: &u32) {
    e.storage()
        .instance()
        .set::<Symbol, u32>(&Symbol::new(e, DECIMALS_KEY), decimals);
}

/// Get the admin address
pub fn get_usdc(e: &Env) -> Address {
    e.storage()
        .instance()
        .get::<Symbol, Address>(&Symbol::new(e, USDC_KEY))
        .unwrap()
}

/// Set the admin address
pub fn set_usdc(e: &Env, usdc: &Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&Symbol::new(e, USDC_KEY), &usdc);
}

/// Get the oracle address for USDC
pub fn get_usdc_oracle(e: &Env) -> Address {
    e.storage()
        .instance()
        .get::<Symbol, Address>(&Symbol::new(e, USDC_ORACLE_KEY))
        .unwrap()
}

/// Set the oracle address for USDC
pub fn set_usdc_oracle(e: &Env, oracle: &Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&Symbol::new(e, USDC_ORACLE_KEY), &oracle);
}

/// Get the oracle address for all non-USDC assets
pub fn get_default_oracle(e: &Env) -> Address {
    e.storage()
        .instance()
        .get::<Symbol, Address>(&Symbol::new(e, DEFAULT_ORACLE_KEY))
        .unwrap()
}

/// Set the oracle address for all non-USDC assets
pub fn set_default_oracle(e: &Env, oracle: &Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&Symbol::new(e, DEFAULT_ORACLE_KEY), &oracle);
}

/********** Persistent **********/

pub fn set_blocked_status(e: &Env, asset: &Asset, blocked: &bool) {
    let key = AggregatorDataKey::Blocked(asset.clone());
    e.storage()
        .persistent()
        .set::<AggregatorDataKey, bool>(&key, blocked);
}

pub fn get_blocked_status(e: &Env, asset: &Asset) -> bool {
    let key = AggregatorDataKey::Blocked(asset.clone());
    get_persistent_default(&e, &key, false, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED)
}
