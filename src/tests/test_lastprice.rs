#![cfg(test)]

use crate::testutils::{create_oracle_aggregator, setup_default_aggregator, EnvTestUtils};
use sep_40_oracle::Asset;
use soroban_sdk::{testutils::Address as _, Address, Env, Error};

#[test]
fn test_lastprice() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let address_0 = Address::generate(&e);
    let address_1 = Address::generate(&e);
    let asset_0 = Asset::Stellar(address_0.clone());
    let asset_1 = Asset::Stellar(address_1.clone());
    let usdc = Address::generate(&e);
    let usdc_asset = Asset::Stellar(usdc.clone());

    let (aggregator, oracle_aggregator_client) = create_oracle_aggregator(&e);
    setup_default_aggregator(&e, &aggregator, &admin, &address_0, &address_1, &usdc);

    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    match price_0 {
        Some(price) => {
            assert_eq!(price.price, 0_11000000000000);
            assert_eq!(price.timestamp, e.ledger().timestamp());
        }
        None => {
            assert!(false)
        }
    }

    let price_1 = oracle_aggregator_client.lastprice(&asset_1);
    match price_1 {
        Some(price) => {
            assert_eq!(price.price, 124_12000000000000);
            assert_eq!(price.timestamp, e.ledger().timestamp());
        }
        None => {
            assert!(false)
        }
    }

    let price_2 = oracle_aggregator_client.lastprice(&usdc_asset);
    match price_2 {
        Some(price) => {
            assert_eq!(price.price, 1_00010231000000);
            assert_eq!(price.timestamp, e.ledger().timestamp() - 600);
        }
        None => {
            assert!(false)
        }
    }
}

#[test]
fn test_lastprice_asset_blocked() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let address_0 = Address::generate(&e);
    let address_1 = Address::generate(&e);
    let asset_0 = Asset::Stellar(address_0.clone());
    let asset_1 = Asset::Stellar(address_1.clone());
    let usdc = Address::generate(&e);

    let (aggregator, oracle_aggregator_client) = create_oracle_aggregator(&e);
    setup_default_aggregator(&e, &aggregator, &admin, &address_0, &address_1, &usdc);

    oracle_aggregator_client.block(&asset_1);
    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    match price_0 {
        Some(price) => {
            assert_eq!(price.price, 0_11000000000000);
            assert_eq!(price.timestamp, e.ledger().timestamp());
        }
        None => {
            assert!(false)
        }
    }
    let result = oracle_aggregator_client.try_lastprice(&asset_1);
    assert_eq!(result.err(), Some(Ok(Error::from_contract_error(107))));
}
