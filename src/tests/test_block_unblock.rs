#![cfg(test)]
use crate::testutils::{create_oracle_aggregator, default_aggregator_settings, EnvTestUtils};
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
    let (settings_config, _, _) = default_aggregator_settings(&e);
    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e, &admin, &settings_config);

    let xlm = settings_config.assets.get(0).unwrap();
    let price = oracle_aggregator_client.lastprice(&xlm);
    assert!(price.is_some());

    oracle_aggregator_client.block_asset(&xlm);
    assert_eq!(
        e.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    oracle_aggregator_client.address.clone(),
                    Symbol::new(&e, "block_asset"),
                    vec![&e, xlm.into_val(&e)]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    let result = oracle_aggregator_client.try_lastprice(&xlm).err();
    assert_eq!(result, Some(Ok(Error::from_contract_error(107))));

    oracle_aggregator_client.unblock_asset(&xlm);
    assert_eq!(
        e.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    oracle_aggregator_client.address.clone(),
                    Symbol::new(&e, "unblock_asset"),
                    vec![&e, xlm.into_val(&e)]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    let price = oracle_aggregator_client.lastprice(&xlm);
    assert!(price.is_some());
}

#[test]
#[should_panic(expected = "Error(Contract, #105)")]
fn test_block_asset_not_found() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let (settings_config, _, _) = default_aggregator_settings(&e);
    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e, &admin, &settings_config);

    let asset = Asset::Other(Symbol::new(&e, "NOT_FOUND"));

    oracle_aggregator_client.block_asset(&asset);
}

#[test]
#[should_panic(expected = "Error(Contract, #105)")]
fn test_unblock_asset_not_found() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let (settings_config, _, _) = default_aggregator_settings(&e);
    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e, &admin, &settings_config);

    let asset = Asset::Other(Symbol::new(&e, "NOT_FOUND"));

    oracle_aggregator_client.unblock_asset(&asset);
}
