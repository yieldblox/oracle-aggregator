use crate::{
    circuit_breaker::check_valid_velocity,
    errors::OracleAggregatorErrors,
    oracle_aggregator::OracleAggregatorTrait,
    price_data::{calculate_average, normalize_price, remove_outliers},
    storage::{self, get_oracle_config},
    types::{Asset, OracleConfig, PriceData},
};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, unwrap::UnwrapOptimized, vec, Address, Env, IntoVal,
    Map, Symbol, Vec,
};
#[contract]
pub struct OracleAggregator;

#[contractimpl]
impl OracleAggregatorTrait for OracleAggregator {
    fn initialize(
        e: Env,
        admin: Address,
        oracles: Vec<Address>,
        oracle_configs: Vec<OracleConfig>,
        decimals: u32,
        base: Asset,
        outlier_threshold: u32,
    ) {
        if storage::get_is_init(&e) {
            panic_with_error!(&e, OracleAggregatorErrors::AlreadyInitialized);
        }

        if oracle_configs.len() > 0 {
            panic_with_error!(&e, OracleAggregatorErrors::NoOracles);
        }

        if oracles.len() != oracle_configs.len() {
            panic_with_error!(&e, OracleAggregatorErrors::InvalidOracleConfig);
        }

        let mut asset_to_oracles: Map<Asset, Vec<Address>> = Map::new(&e);
        for index in 0..oracles.len() {
            let oracle = oracles.get(index).unwrap_optimized();
            let config = oracle_configs.get(index).unwrap_optimized();
            for asset in config.supported_assets.iter() {
                let oracles = asset_to_oracles.get(asset.clone());
                match oracles {
                    Some(mut oracles) => {
                        oracles.push_back(oracle.clone());
                    }
                    None => {
                        asset_to_oracles.set(asset.clone(), vec![&e, oracle.clone()]);
                    }
                }
            }
            storage::set_oracle_config(&e, &oracle, &config);
        }

        storage::extend_instance(&e);
        storage::set_is_init(&e);
        storage::set_admin(&e, &admin);
        storage::set_base(&e, &base);
        storage::set_decimals(&e, &decimals);
        storage::set_outlier_threshold(&e, &outlier_threshold);
        storage::set_oracles(&e, &oracles);
        storage::set_asset_oracle_map(&e, &asset_to_oracles);
    }

    fn base(e: Env) -> Asset {
        storage::get_base(&e)
    }

    fn decimals(e: Env) -> u32 {
        storage::get_decimals(&e)
    }

    fn assets(e: Env) -> Vec<Asset> {
        storage::get_asset_oracle_map(&e).keys()
    }

    fn price(e: Env, asset: Asset, timestamp: u64) -> Option<PriceData> {
        if storage::has_circuit_breaker(&e) && storage::get_circuit_breaker_status(&e) {
            if storage::get_timeout(&e) < e.ledger().timestamp() {
                panic_with_error!(&e, OracleAggregatorErrors::CircuitBreakerTripped);
            } else {
                storage::set_circuit_breaker_status(&e, &false);
            }
        }
        let oracles = storage::get_asset_oracles(&e, &asset);
        let mut price_data = Vec::new(&e);
        let decimals = storage::get_decimals(&e);
        for oracle in oracles.iter() {
            let config = get_oracle_config(&e, &oracle);
            let price: Option<PriceData> = e.invoke_contract(
                &oracle,
                &Symbol::new(&e, "price"),
                vec![&e, asset.into_val(&e), timestamp.into_val(&e)],
            );
            if let Some(price) = price {
                let normalized_price = normalize_price(price, &decimals, &config.decimals);
                price_data.push_back(normalized_price);
            }
        }

        if price_data.len() == 0 {
            return None;
        } else {
            let outlier_threshold = storage::get_outlier_threshold(&e);
            let valid_prices = remove_outliers(&e, price_data, outlier_threshold);
            if valid_prices.len() == 0 {
                return None;
            } else {
                let average_price = calculate_average(valid_prices);
                if storage::has_circuit_breaker(&e) {
                    if check_valid_velocity(&e, &asset, &average_price) {
                        return Some(average_price);
                    } else {
                        return None;
                    }
                }
                return Some(average_price);
            }
        }
    }

    fn last_price(e: Env, asset: Asset) -> Option<PriceData> {
        if storage::has_circuit_breaker(&e) && storage::get_circuit_breaker_status(&e) {
            panic_with_error!(&e, OracleAggregatorErrors::CircuitBreakerTripped);
        }
        let oracles = storage::get_asset_oracles(&e, &asset);
        let mut price_data = Vec::new(&e);
        let decimals = storage::get_decimals(&e);

        for oracle in oracles.iter() {
            let config = get_oracle_config(&e, &oracle);
            let price: Option<PriceData> = e.invoke_contract(
                &oracle,
                &Symbol::new(&e, "last_price"),
                vec![&e, asset.into_val(&e)],
            );
            if let Some(price) = price {
                let normalized_price = normalize_price(price, &decimals, &config.decimals);
                price_data.push_back(normalized_price);
            }
        }

        if price_data.len() == 0 {
            return None;
        } else {
            let outlier_threshold = storage::get_outlier_threshold(&e);
            let valid_prices = remove_outliers(&e, price_data, outlier_threshold);
            if valid_prices.len() == 0 {
                return None;
            } else {
                let average_lastprice = calculate_average(valid_prices);
                if storage::has_circuit_breaker(&e) {
                    if check_valid_velocity(&e, &asset, &average_lastprice) {
                        return Some(average_lastprice);
                    } else {
                        return None;
                    }
                }

                return Some(average_lastprice);
            }
        }
    }

    fn prices(e: Env, asset: Asset, records: u32) -> Option<Vec<PriceData>> {
        if storage::has_circuit_breaker(&e) && storage::get_circuit_breaker_status(&e) {
            panic_with_error!(&e, OracleAggregatorErrors::CircuitBreakerTripped);
        }
        let oracles = storage::get_asset_oracles(&e, &asset);
        let decimals = storage::get_decimals(&e);
        let mut avg_record_prices = Vec::new(&e);
        let mut record_prices: Vec<Vec<PriceData>> = Vec::new(&e);

        for oracle in oracles.iter() {
            let prices: Option<Vec<PriceData>> = e.invoke_contract(
                &oracle,
                &Symbol::new(&e, "prices"),
                vec![&e, asset.into_val(&e), records.into_val(&e)],
            );

            if let Some(prices) = prices {
                let config = get_oracle_config(&e, &oracle);
                for (index, price_record) in prices.iter().enumerate() {
                    let normalized_price =
                        normalize_price(price_record, &decimals, &config.decimals);
                    match record_prices.get(index as u32) {
                        Some(mut prices) => {
                            prices.push_back(normalized_price);
                        }
                        None => {
                            record_prices.push_back(vec![&e, normalized_price]);
                        }
                    }
                }
            }
        }

        let outlier_threshold = storage::get_outlier_threshold(&e);
        for record in record_prices.iter() {
            if record.len() == 0 {
                continue;
            }
            let valid_prices = remove_outliers(&e, record, outlier_threshold);
            if valid_prices.len() == 0 {
                continue;
            }
            let average_price = calculate_average(valid_prices);
            avg_record_prices.push_back(average_price);
        }

        if avg_record_prices.len() == 0 {
            return None;
        } else {
            if storage::has_circuit_breaker(&e) {
                if check_valid_velocity(&e, &asset, &avg_record_prices.get(0).unwrap_optimized()) {
                    return Some(avg_record_prices);
                } else {
                    return None;
                }
            }
            Some(avg_record_prices)
        }
    }

    fn remove_oracle(e: Env, oracle: Address) {
        let admin = storage::get_admin(&e);
        admin.require_auth();

        if !storage::has_oracle_config(&e, &oracle) || !storage::has_oracle(&e, &oracle) {
            panic_with_error!(&e, OracleAggregatorErrors::OracleNotFound);
        }
        let mut asset_oracle_map = storage::get_asset_oracle_map(&e);
        let oracle_config = storage::get_oracle_config(&e, &oracle);

        for supported_asset in oracle_config.supported_assets.iter() {
            let mut oracles = asset_oracle_map
                .get(supported_asset.clone())
                .unwrap_optimized();
            for (index, oracle_address) in oracles.iter().enumerate() {
                if oracle_address == oracle {
                    oracles.remove(index as u32);
                    break;
                }
            }
        }
        storage::set_asset_oracle_map(&e, &asset_oracle_map);
        storage::remove_oracle(&e, &oracle);
        storage::remove_oracle_config(&e, &oracle);
    }
}
