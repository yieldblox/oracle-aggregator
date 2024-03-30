use crate::{
    storage,
    types::{Asset, PriceData},
};
use soroban_sdk::Env;
pub fn check_valid_velocity(
    e: &Env,
    asset: &Asset,
    price: &PriceData,
    prev_price: &PriceData,
) -> bool {
    let velocity_threshold = storage::get_velocity_threshold(&e);
    if prev_price.timestamp < price.timestamp {
        let velocity = ((price.price - prev_price.price).abs() * 10000
            / prev_price.price
            / (price.timestamp - prev_price.timestamp) as i128) as u32;
        if velocity > velocity_threshold {
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
}
