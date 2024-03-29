use oracle_aggregator::types::{Asset, PriceData};
use soroban_sdk::{contract, contractimpl, Address, Env, Vec};
#[contract]
pub struct MockOracle;

#[contractimpl]
impl MockOracle {
    pub fn price(e: Env, asset: Asset, timestamp: u64) -> Option<PriceData> {
        let prices: Vec<PriceData> = e.storage().persistent().get(&asset).unwrap();
        for price in prices.iter() {
            if price.timestamp == timestamp {
                return Some(price.clone());
            }
        }
        return None;
    }

    pub fn last_price(e: Env, asset: Asset) -> Option<PriceData> {
        let prices: Vec<PriceData> = e.storage().persistent().get(&asset).unwrap();
        if prices.len() > 0 {
            return Some(prices.get(0).unwrap().clone());
        } else {
            return None;
        }
    }

    pub fn prices(e: Env, asset: Asset, records: u32) -> Option<Vec<PriceData>> {
        if e.storage().persistent().has(&asset) {
            let prices = e
                .storage()
                .persistent()
                .get::<Asset, Vec<PriceData>>(&asset);
            if let Some(prices) = prices {
                let mut result = Vec::new(&e);
                for i in 0..records {
                    if i < prices.len() as u32 {
                        result.push_back(prices.get_unchecked(i).clone());
                    }
                }
                return Some(result);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    pub fn set_prices(e: Env, asset: Asset, prices: Vec<PriceData>) {
        e.storage().persistent().set(&asset, &prices);
    }
}

pub fn create_mock_oracle<'a>(e: &Env) -> (Address, MockOracleClient<'a>) {
    let mock_oracle_address = e.register_contract(None, MockOracle {});
    let mock_oracle_client: MockOracleClient<'a> = MockOracleClient::new(&e, &mock_oracle_address);
    return (mock_oracle_address, mock_oracle_client);
}
