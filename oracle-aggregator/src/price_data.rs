use crate::types::PriceData;

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
