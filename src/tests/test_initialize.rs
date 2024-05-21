#![cfg(test)]
use crate::testutils::{
    assert_assets_equal, create_oracle_aggregator, setup_default_aggregator, EnvTestUtils,
};
use sep_40_oracle::{
    testutils::{Asset as MockAsset, MockPriceOracleClient, MockPriceOracleWASM},
    Asset,
};
use soroban_sdk::{testutils::Address as _, Address, Env, Symbol, Vec};

#[test]
fn test_initalize() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    e.budget().reset_unlimited();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let address_0 = Address::generate(&e);
    let address_1 = Address::generate(&e);
    let asset_0 = Asset::Stellar(address_0.clone());
    let asset_1 = Asset::Stellar(address_1.clone());
    let usdc = Address::generate(&e);
    let usdc_asset = Asset::Stellar(usdc.clone());

    let (aggregator, oracle_aggregator_client) = create_oracle_aggregator(&e);
    let (default_oracle, _) =
        setup_default_aggregator(&e, &aggregator, &admin, &address_0, &address_1, &usdc);

    assert!(assert_assets_equal(oracle_aggregator_client.base(), base));

    assert_eq!(oracle_aggregator_client.decimals(), 14);

    let assets = oracle_aggregator_client.assets();
    assert_eq!(oracle_aggregator_client.assets().len(), 3);
    assert_assets_equal(assets.get(0).unwrap(), asset_0.clone());
    assert_assets_equal(assets.get(1).unwrap(), asset_1.clone());
    assert_assets_equal(assets.get(2).unwrap(), usdc_asset.clone());
    assert_eq!(
        default_oracle.decimals(),
        oracle_aggregator_client.decimals()
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_already_initialized() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let address_0 = Address::generate(&e);
    let address_1 = Address::generate(&e);
    let usdc = Address::generate(&e);

    let (aggregator, oracle_aggregator_client) = create_oracle_aggregator(&e);
    let (default_oracle, usdc_oracle) =
        setup_default_aggregator(&e, &aggregator, &admin, &address_0, &address_1, &usdc);

    oracle_aggregator_client.initialize(
        &admin,
        &usdc,
        &default_oracle.address,
        &usdc_oracle.address,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #101)")]
fn test_different_decimals() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let address_0 = Address::generate(&e);
    let address_1 = Address::generate(&e);
    let usdc = Address::generate(&e);

    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e);
    let default_oracle_id = e.register_contract_wasm(None, MockPriceOracleWASM);
    let default_oracle = MockPriceOracleClient::new(&e, &default_oracle_id);
    default_oracle.set_data(
        &Address::generate(&e),
        &MockAsset::Other(Symbol::new(&e, "BASE")),
        &Vec::from_array(
            &e,
            [
                MockAsset::Stellar(address_0.clone()),
                MockAsset::Stellar(address_1.clone()),
            ],
        ),
        &14,
        &300,
    );
    let usdc_oracle_id = e.register_contract_wasm(None, MockPriceOracleWASM);
    let usdc_oracle = MockPriceOracleClient::new(&e, &usdc_oracle_id);
    usdc_oracle.set_data(
        &Address::generate(&e),
        &MockAsset::Other(Symbol::new(&e, "BASE")),
        &Vec::from_array(&e, [MockAsset::Other(Symbol::new(&e, "USDC"))]),
        &13,
        &300,
    );

    oracle_aggregator_client.initialize(
        &admin,
        &usdc,
        &default_oracle.address,
        &usdc_oracle.address,
    );
}
