use oracle_aggregator::types::{Asset, PriceData};
use soroban_sdk::{testutils::Address as _, Address, Env, Error, Symbol, Vec};
use test_suite::{
    env::EnvTestUtils,
    oracle_aggregator::{create_oracle_aggregator, default_aggregator_settings},
};
#[test]
fn test_lastprice() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let (settings_config, _, _) = default_aggregator_settings(&e);
    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e, &admin, &settings_config);

    let xlm_price = oracle_aggregator_client.last_price(&settings_config.assets.get(0).unwrap());
    match xlm_price {
        Some(price) => {
            assert_eq!(price.price, 0_1100000);
        }
        None => {
            assert!(false)
        }
    }

    let weth_price = oracle_aggregator_client.last_price(&settings_config.assets.get(2).unwrap());
    match weth_price {
        Some(price) => {
            assert_eq!(price.price, 1010_0000000);
        }
        None => {
            assert!(false)
        }
    }
}

#[test]
fn test_lastprice_circuit_breaker() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let (settings_config, xlm_usdc_oracle, _) = default_aggregator_settings(&e);

    let asset_config = settings_config.asset_configs.get(0).unwrap();
    let asset = settings_config.assets.get(0).unwrap();
    let mut prices = Vec::from_array(
        &e,
        [
            PriceData {
                price: 0_110000000,
                timestamp: e.ledger().timestamp(),
            },
            PriceData {
                price: 0_084515384,
                timestamp: e.ledger().timestamp() - asset_config.resolution,
            },
        ],
    );
    xlm_usdc_oracle.set_prices(&asset, &prices);

    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e, &admin, &settings_config);

    let xlm_price = oracle_aggregator_client.last_price(&asset);

    match xlm_price {
        Some(_) => {
            assert!(false)
        }
        None => {
            assert!(true)
        }
    }

    assert_eq!(
        oracle_aggregator_client.try_last_price(&asset).err(),
        Some(Ok(Error::from_contract_error(104)))
    );

    let timeout_ledgers = (settings_config.circuit_breaker_timeout / 5) as u32;
    e.jump(timeout_ledgers);

    prices.push_front(PriceData {
        price: 0_100000000,
        timestamp: e.ledger().timestamp() - asset_config.resolution,
    });
    prices.push_front(PriceData {
        price: 0_098000000,
        timestamp: e.ledger().timestamp(),
    });
    xlm_usdc_oracle.set_prices(&asset, &prices);

    let xlm_price = oracle_aggregator_client.last_price(&asset);
    match xlm_price {
        Some(price) => {
            assert_eq!(price.price, 0_0980000);
        }
        None => {
            assert!(false)
        }
    }
}

#[test]
#[should_panic(expected = "Error(Contract, #105)")]
fn test_lastprice_asset_not_found() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let (settings_config, _, _) = default_aggregator_settings(&e);
    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e, &admin, &settings_config);

    oracle_aggregator_client.last_price(&Asset::Other(Symbol::new(&e, "NOT_FOUND")));
}
