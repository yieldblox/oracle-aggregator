use soroban_sdk::Env;

use crate::{
    storage,
    types::{AssetConfig, OracleConfig, PriceData, PriceFeedClient},
};

/// Fetch a price based on the asset config
pub fn get_price(e: &Env, oracle: &OracleConfig, config: &AssetConfig) -> Option<PriceData> {
    let decimals = storage::get_decimals(e);
    let max_age = storage::get_max_age(e);
    let oldest_timestamp = e.ledger().timestamp() - max_age;
    let oracle_client = PriceFeedClient::new(e, &oracle.address);
    let mut price: Option<PriceData> = None;
    if config.max_dev > 0 && config.max_dev < 100 {
        let prices = oracle_client.prices(&config.asset, &4);
        if let Some(prices) = prices {
            if prices.len() >= 2 {
                let first_price = prices.get_unchecked(0);
                let second_price = prices.get_unchecked(1);
                let diff = (first_price.price - second_price.price).abs();
                let max_dev = (second_price.price * config.max_dev as i128) / 100;
                if diff < max_dev {
                    price = Some(first_price);
                }
            }
        }
    } else {
        let round_timestamp = oracle_client.last_timestamp();
        if round_timestamp >= oldest_timestamp {
            price = oracle_client.price(&config.asset, &round_timestamp);
        }
    }

    if let Some(ref mut price) = price {
        // normalize the decimals and verify the timestamp returned
        normalize_price(price, &decimals, &oracle.decimals);
        if price.timestamp >= oldest_timestamp {
            return Some(price.clone());
        }
    }
    return None;
}

/// Normalize the price data to the correct number of decimals
fn normalize_price(price_data: &mut PriceData, decimals: &u32, oracle_decimals: &u32) {
    if oracle_decimals > decimals {
        let diff = oracle_decimals - decimals;
        price_data.price = price_data.price / 10_i128.pow(diff);
    } else if oracle_decimals < decimals {
        let diff = decimals - oracle_decimals;
        price_data.price = price_data.price * 10_i128.pow(diff);
    }
}

// @dev: `get_price` tested in intergration tests in `test_lastprice.rs`
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_normalize_price() {
        let price = PriceData {
            price: 1_2345678,
            timestamp: 100,
        };

        let mut price_1 = price.clone();
        normalize_price(&mut price_1, &6, &7);
        assert_eq!(price_1.price, 1234567);

        let mut price_2 = price.clone();
        normalize_price(&mut price_2, &8, &7);
        assert_eq!(price_2.price, 123456780);

        let mut price_3 = price.clone();
        normalize_price(&mut price_3, &18, &7);
        assert_eq!(price_3.price, 1234567800000000000);

        let mut price_4 = price.clone();
        normalize_price(&mut price_4, &2, &7);
        assert_eq!(price_4.price, 123);
    }
}
