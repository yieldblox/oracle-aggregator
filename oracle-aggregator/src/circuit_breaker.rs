use crate::{
    storage,
    types::{Asset, PriceData},
};
use soroban_sdk::Env;
pub fn check_valid_velocity(e: &Env, asset: &Asset, price: &PriceData) -> bool {
    let velocity_threshold = storage::get_velocity_threshold(&e);
    let last_price = storage::get_last_price(&e, &asset);

    if let Some(last_price) = last_price {
        if last_price.timestamp < price.timestamp {
            let velocity = ((price.price - last_price.price).abs() * 10000
                / last_price.price
                / (price.timestamp - last_price.timestamp) as i128)
                as u32;
            if velocity > velocity_threshold {
                // Circuit breaker triggered
                // Set tripped circuit breaker true
                let circuit_breaker_timeout = storage::get_circuit_breaker_timeout(&e);
                storage::set_circuit_breaker_status(&e, &true);
                storage::set_timeout(&e, &(e.ledger().timestamp() + circuit_breaker_timeout));
                return false;
            } else {
                return true;
            }
        } else {
            return true;
        }
    } else {
        return true;
    }
}
