use oracle_aggregator::types::{Asset, PriceData};
use soroban_sdk::{testutils::Address as _, Address, Env, Error, Symbol, Vec};
use test_suite::{
    env::EnvTestUtils,
    oracle_aggregator::{create_oracle_aggregator, default_aggregator_settings},
};
#[test]
fn test_prices() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let (settings_config, _, _) = default_aggregator_settings(&e);
    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e, &admin, &settings_config);

    let xlm = settings_config.assets.get(0).unwrap();
    let xlm_price = oracle_aggregator_client.prices(&xlm, &3);
    match xlm_price {
        Some(prices) => {
            assert_eq!(prices.get(0).unwrap().price, 0_1100000);
            assert_eq!(prices.get(1).unwrap().price, 0_1000000);
            assert_eq!(prices.get(2).unwrap().price, 0_1200000);
        }
        None => {
            assert!(false)
        }
    }
}

#[test]
fn test_prices_circuit_breaker() {
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
            PriceData {
                price: 0_080000000,
                timestamp: e.ledger().timestamp() - asset_config.resolution * 2,
            },
        ],
    );
    xlm_usdc_oracle.set_prices(&asset, &prices);

    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e, &admin, &settings_config);

    let xlm_price = oracle_aggregator_client.prices(&asset, &3);

    match xlm_price {
        Some(_) => {
            assert!(false)
        }
        None => {
            assert!(true)
        }
    }

    assert_eq!(
        oracle_aggregator_client.try_prices(&asset, &3).err(),
        Some(Ok(Error::from_contract_error(104)))
    );

    let timeout_ledgers = (settings_config.circuit_breaker_timeout / 5) as u32;
    e.jump(timeout_ledgers);

    prices.push_front(PriceData {
        price: 0_105000000,
        timestamp: e.ledger().timestamp() - asset_config.resolution * 2,
    });
    prices.push_front(PriceData {
        price: 0_100000000,
        timestamp: e.ledger().timestamp() - asset_config.resolution,
    });
    prices.push_front(PriceData {
        price: 0_098000000,
        timestamp: e.ledger().timestamp(),
    });
    xlm_usdc_oracle.set_prices(&asset, &prices);

    let xlm_prices = oracle_aggregator_client.prices(&asset, &3);
    match xlm_prices {
        Some(prices) => {
            assert_eq!(prices.get(0).unwrap().price, 0_0980000);
            assert_eq!(prices.get(1).unwrap().price, 0_1000000);
            assert_eq!(prices.get(2).unwrap().price, 0_1050000);
        }
        None => {
            assert!(false)
        }
    }
}

#[test]
#[should_panic(expected = "Error(Contract, #105)")]
fn test_prices_asset_not_found() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let (settings_config, _, _) = default_aggregator_settings(&e);
    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e, &admin, &settings_config);

    oracle_aggregator_client.prices(&Asset::Other(Symbol::new(&e, "NOT_FOUND")), &3);
}
