use crate::types::PriceData;
use soroban_sdk::{Env, Vec};
pub fn remove_outliers(
    e: &Env,
    price_data: Vec<PriceData>,
    outlier_threshold: u32,
) -> Vec<PriceData> {
    let mut valid_prices = Vec::new(&e);
    let median = price_data.get_unchecked(price_data.len() / 2);
    for price in price_data.iter() {
        if (price.price - median.price).abs() * 10000 / median.price <= outlier_threshold as i128 {
            valid_prices.push_back(price);
        }
    }
    valid_prices
}

pub fn calculate_average(price_data: Vec<PriceData>) -> PriceData {
    let mut price_sum = 0;
    let mut timestamp_sum = 0;
    for price in price_data.iter() {
        price_sum += price.price;
        timestamp_sum += price.timestamp;
    }
    let price_average = price_sum / price_data.len() as i128;
    let timestamp_average = timestamp_sum / price_data.len() as u64;

    PriceData {
        price: price_average,
        timestamp: timestamp_average,
    }
}

pub fn normalize_price(
    mut price_data: PriceData,
    decimals: &u32,
    oracle_decimals: &u32,
) -> PriceData {
    if oracle_decimals > decimals {
        let diff = oracle_decimals - decimals;
        price_data.price = price_data.price / 10_i128.pow(diff);
        return price_data;
    } else if oracle_decimals < decimals {
        let diff = decimals - oracle_decimals;
        price_data.price = price_data.price * 10_i128.pow(diff);
        return price_data;
    } else {
        return price_data;
    }
}
