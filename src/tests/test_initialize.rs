#![cfg(test)]
use crate::testutils::{
    assert_assets_equal, create_oracle_aggregator, default_aggregator_settings, EnvTestUtils,
};

use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

#[test]
fn test_initalize() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let (settings_config, _, _) = default_aggregator_settings(&e);
    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e, &admin, &settings_config);

    assert!(assert_assets_equal(
        oracle_aggregator_client.base(),
        settings_config.base
    ));

    assert_eq!(
        oracle_aggregator_client.decimals(),
        settings_config.decimals
    );

    let assets = oracle_aggregator_client.assets();
    assert_eq!(
        oracle_aggregator_client.assets().len(),
        settings_config.assets.len()
    );
    for index in 0..assets.len() {
        assert_assets_equal(
            assets.get(index).unwrap(),
            settings_config.assets.get(index).unwrap(),
        );
    }
    for (index, asset) in settings_config.assets.iter().enumerate() {
        let config = oracle_aggregator_client.asset_config(&asset);
        let expected_config = settings_config.asset_configs.get(index as u32).unwrap();

        assert_eq!(expected_config.oracle_id, config.oracle_id);
        assert_eq!(expected_config.decimals, config.decimals);
        assert_eq!(expected_config.resolution, config.resolution);
    }
}

#[test]
#[should_panic(expected = "Error(Contract, #100)")]
fn test_already_initialized() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let (settings_config, _, _) = default_aggregator_settings(&e);
    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e, &admin, &settings_config);

    oracle_aggregator_client.initialize(&admin, &settings_config);
}

#[test]
#[should_panic(expected = "Error(Contract, #102)")]
fn test_initalize_no_assets() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let (mut settings_config, _, _) = default_aggregator_settings(&e);
    settings_config.assets = Vec::new(&e);
    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e, &admin, &settings_config);

    oracle_aggregator_client.initialize(&admin, &settings_config);
}

#[test]
#[should_panic(expected = "Error(Contract, #101)")]
fn test_initalize_missing_configs() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let (mut settings_config, _, _) = default_aggregator_settings(&e);
    settings_config.asset_configs = Vec::new(&e);
    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e, &admin, &settings_config);

    oracle_aggregator_client.initialize(&admin, &settings_config);
}
