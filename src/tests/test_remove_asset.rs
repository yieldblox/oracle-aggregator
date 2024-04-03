#![cfg(test)]
use crate::{
    testutils::{create_oracle_aggregator, default_aggregator_settings, EnvTestUtils},
    types::Asset,
};
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, Error, IntoVal, Symbol,
};

#[test]
fn test_remove_asset() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let (settings_config, _, _) = default_aggregator_settings(&e);
    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e, &admin, &settings_config);

    let xlm = settings_config.assets.get(0).unwrap();
    assert_eq!(
        oracle_aggregator_client.asset_config(&xlm).oracle_id,
        settings_config.asset_configs.get(0).unwrap().oracle_id
    );
    assert_eq!(
        oracle_aggregator_client.asset_config(&xlm).decimals,
        settings_config.asset_configs.get(0).unwrap().decimals
    );
    assert_eq!(
        oracle_aggregator_client.asset_config(&xlm).resolution,
        settings_config.asset_configs.get(0).unwrap().resolution
    );

    oracle_aggregator_client.remove_asset(&xlm);

    assert_eq!(
        e.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    oracle_aggregator_client.address.clone(),
                    Symbol::new(&e, "remove_asset"),
                    vec![&e, xlm.into_val(&e)]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    let result = oracle_aggregator_client.try_asset_config(&xlm).err();
    assert_eq!(result, Some(Ok(Error::from_contract_error(105))));
}

#[test]
#[should_panic(expected = "Error(Contract, #105)")]
fn test_remove_asset_not_found() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let (settings_config, _, _) = default_aggregator_settings(&e);
    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e, &admin, &settings_config);

    let asset = Asset::Other(Symbol::new(&e, "NOT_FOUND"));

    oracle_aggregator_client.remove_asset(&asset);
}
