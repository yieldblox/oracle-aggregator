#![cfg(test)]

use crate::testutils::{create_oracle_aggregator, setup_default_aggregator, EnvTestUtils};
use sep_40_oracle::Asset;
use soroban_sdk::{testutils::Address as _, Address, Env, Error, Symbol};

#[test]
fn test_lastprice() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));
    let asset_2 = Asset::Other(Symbol::new(&e, "wETH"));

    let (aggregator, oracle_aggregator_client) = create_oracle_aggregator(&e);
    setup_default_aggregator(&e, &aggregator, &admin, &base, &asset_0, &asset_1, &asset_2);

    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    match price_0 {
        Some(price) => {
            assert_eq!(price.price, 0_1100000);
            assert_eq!(price.timestamp, e.ledger().timestamp());
        }
        None => {
            assert!(false)
        }
    }

    let price_1 = oracle_aggregator_client.lastprice(&asset_1);
    match price_1 {
        Some(price) => {
            assert_eq!(price.price, 1_0000000);
            assert_eq!(price.timestamp, e.ledger().timestamp());
        }
        None => {
            assert!(false)
        }
    }

    let price_2 = oracle_aggregator_client.lastprice(&asset_2);
    match price_2 {
        Some(price) => {
            assert_eq!(price.price, 1010_0000000);
            assert_eq!(price.timestamp, e.ledger().timestamp() - 600);
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
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));
    let asset_2 = Asset::Other(Symbol::new(&e, "wETH"));

    let (aggregator, oracle_aggregator_client) = create_oracle_aggregator(&e);
    setup_default_aggregator(&e, &aggregator, &admin, &base, &asset_0, &asset_1, &asset_2);

    oracle_aggregator_client.lastprice(&Asset::Other(Symbol::new(&e, "NOT_FOUND")));
}

#[test]
fn test_lastprice_asset_blocked() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));
    let asset_2 = Asset::Other(Symbol::new(&e, "wETH"));

    let (aggregator, oracle_aggregator_client) = create_oracle_aggregator(&e);
    setup_default_aggregator(&e, &aggregator, &admin, &base, &asset_0, &asset_1, &asset_2);

    oracle_aggregator_client.block(&asset_1);
    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    match price_0 {
        Some(price) => {
            assert_eq!(price.price, 0_1100000);
            assert_eq!(price.timestamp, e.ledger().timestamp());
        }
        None => {
            assert!(false)
        }
    }
    let result = oracle_aggregator_client.try_lastprice(&asset_1);
    assert_eq!(result.err(), Some(Ok(Error::from_contract_error(107))));
}
