#![cfg(test)]
use crate::testutils::{create_oracle_aggregator, default_aggregator_settings, EnvTestUtils};
use sep_40_oracle::Asset;
use soroban_sdk::{testutils::Address as _, Address, Env, Error, Symbol, Vec};
#[test]
fn test_lastprice() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let (settings_config, _, _) = default_aggregator_settings(&e);
    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e, &admin, &settings_config);

    let xlm_price = oracle_aggregator_client.lastprice(&settings_config.assets.get(0).unwrap());
    match xlm_price {
        Some(price) => {
            assert_eq!(price.price, 0_1100000);
        }
        None => {
            assert!(false)
        }
    }

    let weth_price = oracle_aggregator_client.lastprice(&settings_config.assets.get(2).unwrap());
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

    xlm_usdc_oracle.set_price(
        &Vec::from_array(&e, [0_084515384]),
        &(e.ledger().timestamp() - asset_config.resolution),
    );
    xlm_usdc_oracle.set_price(&Vec::from_array(&e, [0_110000000]), &e.ledger().timestamp());

    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e, &admin, &settings_config);

    let xlm_price = oracle_aggregator_client.lastprice(&asset);

    match xlm_price {
        Some(_) => {
            assert!(false)
        }
        None => {
            assert!(true)
        }
    }

    assert_eq!(
        oracle_aggregator_client.try_lastprice(&asset).err(),
        Some(Ok(Error::from_contract_error(104)))
    );

    let timeout_ledgers = (settings_config.circuit_breaker_timeout / 5) as u32;
    e.jump(timeout_ledgers);

    xlm_usdc_oracle.set_price(
        &Vec::from_array(&e, [0_100000000]),
        &(e.ledger().timestamp() - asset_config.resolution),
    );
    xlm_usdc_oracle.set_price(&Vec::from_array(&e, [0_098000000]), &e.ledger().timestamp());

    let xlm_price = oracle_aggregator_client.lastprice(&asset);
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

    oracle_aggregator_client.lastprice(&Asset::Other(Symbol::new(&e, "NOT_FOUND")));
}
