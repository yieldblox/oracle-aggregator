use crate::{
    circuit_breaker::check_valid_velocity,
    errors::OracleAggregatorErrors,
    oracle_aggregator::OracleAggregatorTrait,
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
impl OracleAggregatorTrait for OracleAggregator {
    fn initialize(e: Env, admin: Address, config: SettingsConfig) {
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

    fn base(e: Env) -> Asset {
        storage::get_base(&e)
    }

    fn decimals(e: Env) -> u32 {
        storage::get_decimals(&e)
    }

    fn assets(e: Env) -> Vec<Asset> {
        storage::get_assets(&e)
    }

    fn asset_config(e: Env, asset: Asset) -> OracleConfig {
        if storage::has_asset_config(&e, &asset) {
            return storage::get_asset_config(&e, &asset);
        } else {
            panic_with_error!(&e, OracleAggregatorErrors::AssetNotFound);
        }
    }

    fn price(e: Env, asset: Asset, timestamp: u64) -> Option<PriceData> {
        if storage::has_circuit_breaker(&e) && storage::get_circuit_breaker_status(&e, &asset) {
            if storage::get_timeout(&e) < e.ledger().timestamp() {
                panic_with_error!(&e, OracleAggregatorErrors::CircuitBreakerTripped);
            } else {
                storage::set_circuit_breaker_status(&e, &asset, &false);
            }
        }

        let config = storage::get_asset_config(&e, &asset);
        let normalized_timestamp = timestamp / config.resolution * config.resolution;
        let price: Option<PriceData> = e.invoke_contract(
            &config.oracle_id,
            &Symbol::new(&e, "price"),
            vec![&e, asset.into_val(&e), normalized_timestamp.into_val(&e)],
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
                if prev_price.is_some() {
                    if check_valid_velocity(&e, &asset, &price, &prev_price.unwrap_optimized()) {
                        return Some(normalized_price);
                    } else {
                        return None;
                    }
                } else {
                    // Oracles first price no need to check velocity
                    return Some(normalized_price);
                }
            }
            return Some(normalized_price);
        } else {
            return None;
        }
    }

    fn last_price(e: Env, asset: Asset) -> Option<PriceData> {
        if storage::has_circuit_breaker(&e) && storage::get_circuit_breaker_status(&e, &asset) {
            if storage::get_timeout(&e) < e.ledger().timestamp() {
                panic_with_error!(&e, OracleAggregatorErrors::CircuitBreakerTripped);
            } else {
                storage::set_circuit_breaker_status(&e, &asset, &false);
            }
        }

        let config = storage::get_asset_config(&e, &asset);
        let price: Option<PriceData> = e.invoke_contract(
            &config.oracle_id,
            &Symbol::new(&e, "last_price"),
            vec![&e, asset.into_val(&e)],
        );

        if let Some(price) = price {
            let decimals = storage::get_decimals(&e);
            let noramlized_price = normalize_price(price.clone(), &decimals, &config.decimals);

            if storage::has_circuit_breaker(&e) {
                let prev_timestamp = price.timestamp - config.resolution;
                let prev_price: Option<PriceData> = e.invoke_contract(
                    &config.oracle_id,
                    &Symbol::new(&e, "price"),
                    vec![&e, asset.into_val(&e), prev_timestamp.into_val(&e)],
                );

                if prev_price.is_some() {
                    if check_valid_velocity(&e, &asset, &price, &prev_price.unwrap_optimized()) {
                        return Some(noramlized_price);
                    } else {
                        return None;
                    }
                } else {
                    // Oracles first price no need to check velocity
                    return Some(noramlized_price);
                }
            }
            return Some(noramlized_price);
        } else {
            return None;
        }
    }

    fn prices(e: Env, asset: Asset, records: u32) -> Option<Vec<PriceData>> {
        if storage::has_circuit_breaker(&e) && storage::get_circuit_breaker_status(&e, &asset) {
            if storage::get_timeout(&e) < e.ledger().timestamp() {
                panic_with_error!(&e, OracleAggregatorErrors::CircuitBreakerTripped);
            } else {
                storage::set_circuit_breaker_status(&e, &asset, &false);
            }
        }

        let config = storage::get_asset_config(&e, &asset);
        let prices: Option<Vec<PriceData>> = e.invoke_contract(
            &config.oracle_id,
            &Symbol::new(&e, "prices"),
            vec![&e, asset.into_val(&e), records.into_val(&e)],
        );

        if let Some(prices) = prices {
            let mut normalized_prices = Vec::new(&e);
            for price in prices.iter() {
                let decimals = storage::get_decimals(&e);
                let noramlized_price = normalize_price(price.clone(), &decimals, &config.decimals);
                normalized_prices.push_back(noramlized_price);
            }

            if storage::has_circuit_breaker(&e) {
                for index in 0..prices.len() {
                    if index == 0 {
                        continue;
                    }
                    let price = prices.get(index).unwrap_optimized();
                    let prev_price = prices.get(index - 1).unwrap_optimized();
                    if !check_valid_velocity(&e, &asset, &price, &prev_price) {
                        return None;
                    }
                }
                Some(prices.clone());
            }
            return Some(prices);
        } else {
            return None;
        }
    }

    fn remove_asset(e: Env, asset: Asset) {
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
