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
    let round_timestamp = oracle_client.last_timestamp();
    let oracle_resolution = oracle.resolution as u64;
    let mut price: Option<PriceData> = None;
    let mut next_timestamp = round_timestamp.clone();
    while price.is_none() && next_timestamp >= oldest_timestamp {
        price = oracle_client.price(&config.asset, &next_timestamp);
        next_timestamp = next_timestamp - oracle_resolution;
    }

    // a price was found
    if let Some(price) = price {
        // if we need to verify the max dev, look for an older price
        if config.max_dev > 0 && config.max_dev < 100 {
            // have a valid price for the asset from the oracle. Attempt to fetch an older price
            // to validate max_dev. Looks at most `max_age / resolution` prices back from the most recent
            // price.
            let max_steps = max_age / oracle_resolution;
            let mut old_price: Option<PriceData> = None;
            for _ in 0..max_steps {
                old_price = oracle_client.price(&config.asset, &next_timestamp);
                if old_price.is_some() {
                    break;
                }
                next_timestamp = next_timestamp - oracle_resolution;
            }
            if let Some(old_price) = old_price {
                // check the price is within the max_dev, return None if it is not
                let diff = (price.price - old_price.price).abs();
                let max_dev = (old_price.price * config.max_dev as i128) / 100;
                if diff > max_dev {
                    return None;
                }
            } else {
                // no old price found, so we cannot verify the max_dev, return None
                return None;
            }
        }

        // normalize the decimals and verify the timestamp returned
        let normalized_price = normalize_price(price, &decimals, &oracle.decimals);
        if normalized_price.timestamp >= oldest_timestamp {
            return Some(normalized_price);
        }
    }
    return None;
}

/// Normalize the price data to the correct number of decimals
fn normalize_price(mut price_data: PriceData, decimals: &u32, oracle_decimals: &u32) -> PriceData {
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
        let normalized_price = normalize_price(price.clone(), &6, &7);
        assert_eq!(normalized_price.price, 1234567);

        let normalized_price = normalize_price(price.clone(), &8, &7);
        assert_eq!(normalized_price.price, 123456780);

        let normalized_price = normalize_price(price.clone(), &18, &7);
        assert_eq!(normalized_price.price, 1234567800000000000);

        let normalized_price = normalize_price(price, &2, &7);
        assert_eq!(normalized_price.price, 123);
    }
}
