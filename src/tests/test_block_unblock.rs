#![cfg(test)]

use crate::testutils::{create_oracle_aggregator, setup_default_aggregator, EnvTestUtils};
use sep_40_oracle::Asset;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, Error, IntoVal, Symbol,
};

#[test]
fn test_block_unblock() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let address_0 = Address::generate(&e);
    let address_1 = Address::generate(&e);
    let asset_0 = Asset::Stellar(address_0.clone());
    let usdc = Address::generate(&e);

    let (aggregator, oracle_aggregator_client) = create_oracle_aggregator(&e);
    setup_default_aggregator(&e, &aggregator, &admin, &address_0, &address_1, &usdc);

    let price = oracle_aggregator_client.lastprice(&asset_0);
    assert!(price.is_some());

    oracle_aggregator_client.block(&asset_0);
    assert_eq!(
        e.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    oracle_aggregator_client.address.clone(),
                    Symbol::new(&e, "block"),
                    vec![&e, asset_0.into_val(&e)]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    let result = oracle_aggregator_client.try_lastprice(&asset_0).err();
    assert_eq!(result, Some(Ok(Error::from_contract_error(107))));

    oracle_aggregator_client.unblock(&asset_0);
    assert_eq!(
        e.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    oracle_aggregator_client.address.clone(),
                    Symbol::new(&e, "unblock"),
                    vec![&e, asset_0.into_val(&e)]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    let price = oracle_aggregator_client.lastprice(&asset_0);
    assert!(price.is_some());
}

#[test]
fn test_set_admin() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let address_0 = Address::generate(&e);
    let address_1 = Address::generate(&e);
    let asset_0 = Asset::Stellar(address_0.clone());
    let usdc = Address::generate(&e);

    let new_admin = Address::generate(&e);

    let (aggregator, oracle_aggregator_client) = create_oracle_aggregator(&e);
    setup_default_aggregator(&e, &aggregator, &admin, &address_0, &address_1, &usdc);

    let price = oracle_aggregator_client.lastprice(&asset_0);
    assert!(price.is_some());

    oracle_aggregator_client.set_admin(&new_admin);
    assert_eq!(
        e.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    oracle_aggregator_client.address.clone(),
                    Symbol::new(&e, "set_admin"),
                    vec![&e, new_admin.into_val(&e)]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    oracle_aggregator_client.block(&asset_0);
    assert_eq!(
        e.auths()[0],
        (
            new_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    oracle_aggregator_client.address.clone(),
                    Symbol::new(&e, "block"),
                    vec![&e, asset_0.into_val(&e)]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    let result = oracle_aggregator_client.try_lastprice(&asset_0).err();
    assert_eq!(result, Some(Ok(Error::from_contract_error(107))));

    oracle_aggregator_client.unblock(&asset_0);
    assert_eq!(
        e.auths()[0],
        (
            new_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    oracle_aggregator_client.address.clone(),
                    Symbol::new(&e, "unblock"),
                    vec![&e, asset_0.into_val(&e)]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    let price = oracle_aggregator_client.lastprice(&asset_0);
    assert!(price.is_some());
}
