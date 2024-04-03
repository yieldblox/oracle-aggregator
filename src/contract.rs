use crate::{
    circuit_breaker::{check_circuit_breaker, check_valid_velocity},
    errors::OracleAggregatorErrors,
    price_data::normalize_price,
    storage,
    types::{Asset, OracleConfig, PriceData, SettingsConfig},
};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, unwrap::UnwrapOptimized, vec, Address, Env, IntoVal,
    Symbol, Vec,
};
#[contract]
pub struct OracleAggregator;

#[contractimpl]
impl OracleAggregator {
    pub fn initialize(e: Env, admin: Address, config: SettingsConfig) {
        if storage::get_is_init(&e) {
            panic_with_error!(&e, OracleAggregatorErrors::AlreadyInitialized);
        }

        if config.assets.len() <= 0 {
            panic_with_error!(&e, OracleAggregatorErrors::NoOracles);
        }

        if config.assets.len() != config.asset_configs.len() {
            panic_with_error!(&e, OracleAggregatorErrors::InvalidOracleConfig);
        }

        for index in 0..config.assets.len() {
            let asset = config.assets.get(index).unwrap_optimized();
            let config = config.asset_configs.get(index).unwrap_optimized();
            storage::set_asset_config(&e, &asset, &config);
        }

        storage::extend_instance(&e);
        storage::set_is_init(&e);
        storage::set_admin(&e, &admin);
        storage::set_base(&e, &config.base);
        storage::set_decimals(&e, &config.decimals);
        storage::set_assets(&e, &config.assets);
        storage::set_circuit_breaker(&e, &config.enable_circuit_breaker);
        if config.enable_circuit_breaker {
            storage::set_velocity_threshold(&e, &config.circuit_breaker_threshold);
            storage::set_timeout(&e, &config.circuit_breaker_timeout);
        }
    }

    pub fn base(e: Env) -> Asset {
        storage::get_base(&e)
    }

    pub fn decimals(e: Env) -> u32 {
        storage::get_decimals(&e)
    }

    pub fn assets(e: Env) -> Vec<Asset> {
        storage::get_assets(&e)
    }

    pub fn asset_config(e: Env, asset: Asset) -> OracleConfig {
        if storage::has_asset_config(&e, &asset) {
            return storage::get_asset_config(&e, &asset);
        } else {
            panic_with_error!(&e, OracleAggregatorErrors::AssetNotFound);
        }
    }

    pub fn last_price(e: Env, asset: Asset) -> Option<PriceData> {
        if !storage::has_asset_config(&e, &asset) {
            panic_with_error!(&e, OracleAggregatorErrors::AssetNotFound);
        }
        check_circuit_breaker(&e, &asset);

        let config = storage::get_asset_config(&e, &asset);
        let price: Option<PriceData> = e.invoke_contract(
            &config.oracle_id,
            &Symbol::new(&e, "last_price"),
            vec![&e, asset.into_val(&e)],
        );

        if let Some(price) = price {
            let decimals = storage::get_decimals(&e);
            let normalized_price = normalize_price(price.clone(), &decimals, &config.decimals);

            if storage::has_circuit_breaker(&e) {
                let prev_timestamp = price.timestamp - config.resolution;
                let prev_price: Option<PriceData> = e.invoke_contract(
                    &config.oracle_id,
                    &Symbol::new(&e, "price"),
                    vec![&e, asset.into_val(&e), prev_timestamp.into_val(&e)],
                );

                if prev_price.is_some()
                    && !check_valid_velocity(&e, &asset, &price, &prev_price.unwrap_optimized())
                {
                    return None;
                }
            }

            return Some(normalized_price);
        } else {
            return None;
        }
    }

    pub fn remove_asset(e: Env, asset: Asset) {
        let admin = storage::get_admin(&e);
        admin.require_auth();

        if !storage::has_asset_config(&e, &asset) {
            panic_with_error!(&e, OracleAggregatorErrors::AssetNotFound);
        }

        storage::remove_asset_config(&e, &asset);
        let mut assets = storage::get_assets(&e);
        for index in 0..assets.len() {
            if assets.get(index).unwrap_optimized() == asset {
                assets.remove(index);
                break;
            }
        }
        storage::set_assets(&e, &assets);
    }
}
