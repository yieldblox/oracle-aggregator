use sep_40_oracle::PriceData;
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
