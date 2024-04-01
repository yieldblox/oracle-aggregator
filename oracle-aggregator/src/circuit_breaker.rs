use crate::{
    errors::OracleAggregatorErrors,
    storage,
    types::{Asset, PriceData},
};
use soroban_sdk::{panic_with_error, Env};

pub fn check_valid_velocity(
    e: &Env,
    asset: &Asset,
    price: &PriceData,
    prev_price: &PriceData,
) -> bool {
    let velocity_threshold = storage::get_velocity_threshold(&e);
    if prev_price.timestamp < price.timestamp {
        let percent_price_delta = (price.price - prev_price.price).abs() * 10000 / prev_price.price;
        let velocity =
            percent_price_delta * 10000 / (price.timestamp - prev_price.timestamp) as i128;
        if velocity as u32 > velocity_threshold {
            let circuit_breaker_timeout = storage::get_timeout(&e);
            storage::set_circuit_breaker_status(&e, &asset, &true);
            storage::set_circuit_breaker_timeout(
                &e,
                &asset,
                &(e.ledger().timestamp() + circuit_breaker_timeout),
            );
            return false;
        } else {
            return true;
        }
    } else {
        return true;
    }
}

pub fn check_circuit_breaker(e: &Env, asset: &Asset) {
    if storage::has_circuit_breaker(&e) && storage::get_circuit_breaker_status(&e, &asset) {
        if storage::get_circuit_breaker_timeout(&e, &asset) > e.ledger().timestamp() {
            panic_with_error!(&e, OracleAggregatorErrors::CircuitBreakerTripped);
        } else {
            storage::set_circuit_breaker_status(&e, &asset, &false);
        }
    }
}
