#![cfg(test)]
use crate::{
    storage,
    testutils::{
        assert_asset_config_equal, assert_assets_equal, assert_oracle_config_equal,
        create_oracle_aggregator, EnvTestUtils,
    },
    types::{Asset, AssetConfig, OracleConfig},
};
use sep_40_oracle::testutils::{Asset as MockAsset, MockPriceOracleClient, MockPriceOracleWASM};

use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, Error, IntoVal, Map, Symbol, Vec,
};

#[test]
fn test_admin_actions() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    e.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let decimals = 7;
    let max_age = 900;

    let asset_0 = Address::generate(&e);
    let asset_1 = Address::generate(&e);
    let asset_2 = Address::generate(&e);
    let symbol_2 = Symbol::new(&e, "wETH");

    // deploy two mock oracles
    // - setup oracle with all asset prices
    let oracle_0_1_id = Address::generate(&e);
    e.register_at(&oracle_0_1_id, MockPriceOracleWASM, ());
    let oracle_0_1 = MockPriceOracleClient::new(&e, &oracle_0_1_id);
    oracle_0_1.set_data(
        &Address::generate(&e),
        &MockAsset::Other(Symbol::new(&e, "BASE")),
        &Vec::from_array(
            &e,
            [
                MockAsset::Stellar(asset_0.clone()),
                MockAsset::Stellar(asset_1.clone()),
                // @dev - this is added just to test edge cases but is not used
                MockAsset::Stellar(asset_2.clone()),
            ],
        ),
        &9,
        &300,
    );
    let oracle_0_1_norm_timestamp = e.ledger().timestamp() / 300 * 300;
    oracle_0_1.set_price(
        &vec![&e, 1_300_000_055, 2223_421_213_231, 0_900_120_123],
        &(oracle_0_1_norm_timestamp - 300),
    );
    oracle_0_1.set_price(
        &vec![&e, 1_200_000_055, 2123_421_213_231, 0_924_120_123],
        &oracle_0_1_norm_timestamp,
    );

    // - setup oracle with ASSET_2 price
    let oracle_2_id = Address::generate(&e);
    e.register_at(&oracle_2_id, MockPriceOracleWASM, ());
    let oracle_2 = MockPriceOracleClient::new(&e, &oracle_2_id);
    oracle_2.set_data(
        &Address::generate(&e),
        &MockAsset::Other(Symbol::new(&e, "BASE")),
        &Vec::from_array(&e, [MockAsset::Other(symbol_2.clone())]),
        &6,
        &600,
    );
    let oracle_2_norm_timestamp = e.ledger().timestamp() / 600 * 600;
    oracle_2.set_price(&vec![&e, 0_994_120], &(oracle_2_norm_timestamp - 600));
    oracle_2.set_price(&vec![&e, 1_024_025], &oracle_2_norm_timestamp);

    // deploy oracle aggregator
    let (oracle_aggregator_id, oracle_aggregator_client) =
        create_oracle_aggregator(&e, &admin, &base, &decimals, &max_age);
    assert_assets_equal(oracle_aggregator_client.base(), base);
    assert_eq!(oracle_aggregator_client.decimals(), 7);
    assert_eq!(oracle_aggregator_client.max_age(), 900);

    e.jump(10);

    // add oracle_0_1
    oracle_aggregator_client.add_oracle(&oracle_0_1_id);
    assert_eq!(
        e.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    oracle_aggregator_id.clone(),
                    Symbol::new(&e, "add_oracle"),
                    vec![&e, oracle_0_1_id.to_val()]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    let oracles = oracle_aggregator_client.oracles();
    assert_eq!(oracles.len(), 1);
    assert_oracle_config_equal(
        oracles.get_unchecked(0),
        OracleConfig {
            address: oracle_0_1_id.clone(),
            index: 0,
            resolution: 300,
            decimals: 9,
        },
    );

    // add asset_0
    let result_0 = oracle_aggregator_client.add_asset(
        &Asset::Stellar(asset_0.clone()),
        &oracle_0_1_id,
        &Asset::Stellar(asset_0.clone()),
        &0,
    );
    assert_eq!(
        e.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    oracle_aggregator_id.clone(),
                    Symbol::new(&e, "add_asset"),
                    vec![
                        &e,
                        Asset::Stellar(asset_0.clone()).into_val(&e),
                        oracle_0_1_id.to_val(),
                        Asset::Stellar(asset_0.clone()).into_val(&e),
                        0u32.into_val(&e)
                    ]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    assert_eq!(result_0.price, 1_200_000_0);
    assert_eq!(result_0.timestamp, oracle_0_1_norm_timestamp);
    let assets = oracle_aggregator_client.assets();
    assert_eq!(assets.len(), 1);
    assert_assets_equal(assets.get_unchecked(0), Asset::Stellar(asset_0.clone()));
    let asset_0_config = oracle_aggregator_client
        .asset_configs()
        .get(Asset::Stellar(asset_0.clone()))
        .unwrap();
    assert_asset_config_equal(
        asset_0_config,
        AssetConfig {
            asset: Asset::Stellar(asset_0.clone()),
            oracle_index: 0,
            max_dev: 0,
        },
    );

    // add asset_1
    let result_1 = oracle_aggregator_client.add_asset(
        &Asset::Stellar(asset_1.clone()),
        &oracle_0_1_id,
        &Asset::Stellar(asset_1.clone()),
        &15,
    );
    assert_eq!(result_1.price, 2123_421_213_2);
    assert_eq!(result_1.timestamp, oracle_0_1_norm_timestamp);
    let assets = oracle_aggregator_client.assets();
    assert_eq!(assets.len(), 2);
    assert_assets_equal(assets.get_unchecked(0), Asset::Stellar(asset_0.clone()));
    assert_assets_equal(assets.get_unchecked(1), Asset::Stellar(asset_1.clone()));
    let asset_1_config = oracle_aggregator_client
        .asset_configs()
        .get(Asset::Stellar(asset_1.clone()))
        .unwrap();
    assert_asset_config_equal(
        asset_1_config,
        AssetConfig {
            asset: Asset::Stellar(asset_1.clone()),
            oracle_index: 0,
            max_dev: 15,
        },
    );

    // try and add asset_2 without the oracle existing
    let result_no_oracle = oracle_aggregator_client.try_add_asset(
        &Asset::Stellar(asset_2.clone()),
        &oracle_2_id,
        &Asset::Other(symbol_2.clone()),
        &0,
    );
    assert_eq!(
        result_no_oracle.err(),
        Some(Ok(Error::from_contract_error(107)))
    );

    // add oracle_2
    oracle_aggregator_client.add_oracle(&oracle_2_id);
    let oracles = oracle_aggregator_client.oracles();
    assert_eq!(oracles.len(), 2);
    assert_oracle_config_equal(
        oracles.get_unchecked(1),
        OracleConfig {
            address: oracle_2_id.clone(),
            index: 1,
            resolution: 600,
            decimals: 6,
        },
    );

    // try and add an incorrect config for asset_2
    let result_bad_config = oracle_aggregator_client.try_add_asset(
        &Asset::Stellar(asset_2.clone()),
        &oracle_2_id,
        &Asset::Stellar(asset_2.clone()),
        &0,
    );
    // - mock oracle throws an error during last price fetch
    assert_eq!(
        result_bad_config.err(),
        Some(Ok(Error::from_contract_error(2)))
    );

    // try and add asset_2 when the list is full
    // - manually create a really long asset list to test the max asset limit
    let cached_asset_configs = oracle_aggregator_client.asset_configs();
    e.as_contract(&oracle_aggregator_id, || {
        let mut assets = Map::<Asset, AssetConfig>::new(&e);
        for _ in 0..20 {
            let asset = Asset::Stellar(Address::generate(&e));
            let asset_config = AssetConfig {
                asset: asset.clone(),
                oracle_index: 0,
                max_dev: 0,
            };
            assets.set(asset, asset_config);
        }
        storage::set_asset_configs(&e, &assets);
    });
    // - do test
    let result_full = oracle_aggregator_client.try_add_asset(
        &Asset::Stellar(asset_2.clone()),
        &oracle_2_id,
        &Asset::Other(symbol_2.clone()),
        &0,
    );
    assert_eq!(result_full.err(), Some(Ok(Error::from_contract_error(102))));
    // - revert asset list change
    e.as_contract(&oracle_aggregator_id, || {
        storage::set_asset_configs(&e, &cached_asset_configs);
    });

    // add asset_2
    let result_2 = oracle_aggregator_client.add_asset(
        &Asset::Stellar(asset_2.clone()),
        &oracle_2_id,
        &Asset::Other(symbol_2.clone()),
        &0,
    );
    assert_eq!(result_2.price, 1_024_0250);
    assert_eq!(result_2.timestamp, oracle_2_norm_timestamp);
    let assets = oracle_aggregator_client.assets();
    assert_eq!(assets.len(), 3);
    assert_assets_equal(assets.get_unchecked(0), Asset::Stellar(asset_0.clone()));
    assert_assets_equal(assets.get_unchecked(1), Asset::Stellar(asset_1.clone()));
    assert_assets_equal(assets.get_unchecked(2), Asset::Stellar(asset_2.clone()));
    let asset_2_config = oracle_aggregator_client
        .asset_configs()
        .get(Asset::Stellar(asset_2.clone()))
        .unwrap();
    assert_asset_config_equal(
        asset_2_config,
        AssetConfig {
            asset: Asset::Other(symbol_2.clone()),
            oracle_index: 1,
            max_dev: 0,
        },
    );

    // try and add a duplicate asset
    let result_dupe = oracle_aggregator_client.try_add_asset(
        &Asset::Stellar(asset_2.clone()),
        &oracle_0_1_id,
        &Asset::Stellar(asset_2.clone()),
        &0,
    );
    assert_eq!(result_dupe.err(), Some(Ok(Error::from_contract_error(103))));
}

#[test]
#[should_panic(expected = "Error(Contract, #105)")]
fn test_init_max_age_too_small() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    e.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let decimals = 7;
    let max_age = 359;

    create_oracle_aggregator(&e, &admin, &base, &decimals, &max_age);
}

#[test]
#[should_panic(expected = "Error(Contract, #105)")]
fn test_init_max_age_too_large() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    e.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let decimals = 7;
    let max_age = 359;

    create_oracle_aggregator(&e, &admin, &base, &decimals, &max_age);
}

#[test]
fn test_oracles() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    e.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let decimals = 7;
    let max_age = 900;

    let asset_0 = Address::generate(&e);
    let asset_1 = Address::generate(&e);
    let asset_2 = Address::generate(&e);
    let symbol_2 = Symbol::new(&e, "wETH");

    // deploy two mock oracles
    // - setup oracle with all asset prices
    let oracle_0_1_id = Address::generate(&e);
    e.register_at(&oracle_0_1_id, MockPriceOracleWASM, ());
    let oracle_0_1 = MockPriceOracleClient::new(&e, &oracle_0_1_id);
    oracle_0_1.set_data(
        &Address::generate(&e),
        &MockAsset::Other(Symbol::new(&e, "BASE")),
        &Vec::from_array(
            &e,
            [
                MockAsset::Stellar(asset_0.clone()),
                MockAsset::Stellar(asset_1.clone()),
                // @dev - this is added just to test edge cases but is not used
                MockAsset::Stellar(asset_2.clone()),
            ],
        ),
        &9,
        &300,
    );
    let oracle_0_1_norm_timestamp = e.ledger().timestamp() / 300 * 300;
    oracle_0_1.set_price(
        &vec![&e, 1_300_000_055, 2223_421_213_231, 0_900_120_123],
        &(oracle_0_1_norm_timestamp - 300),
    );
    oracle_0_1.set_price(
        &vec![&e, 1_200_000_055, 2123_421_213_231, 0_924_120_123],
        &oracle_0_1_norm_timestamp,
    );

    // - setup oracle with ASSET_2 price
    let oracle_2_id = Address::generate(&e);
    e.register_at(&oracle_2_id, MockPriceOracleWASM, ());
    let oracle_2 = MockPriceOracleClient::new(&e, &oracle_2_id);
    oracle_2.set_data(
        &Address::generate(&e),
        &MockAsset::Other(Symbol::new(&e, "BASE")),
        &Vec::from_array(&e, [MockAsset::Other(symbol_2.clone())]),
        &6,
        &600,
    );
    let oracle_2_norm_timestamp = e.ledger().timestamp() / 600 * 600;
    oracle_2.set_price(&vec![&e, 0_994_120], &(oracle_2_norm_timestamp - 600));
    oracle_2.set_price(&vec![&e, 1_024_025], &oracle_2_norm_timestamp);

    // deploy oracle aggregator
    let (oracle_aggregator_id, oracle_aggregator_client) =
        create_oracle_aggregator(&e, &admin, &base, &decimals, &max_age);
    assert_assets_equal(oracle_aggregator_client.base(), base.clone());
    assert_eq!(oracle_aggregator_client.decimals(), 7);
    assert_eq!(oracle_aggregator_client.max_age(), 900);

    e.jump(10);

    // add oracle_0_1
    oracle_aggregator_client.add_oracle(&oracle_0_1_id);
    assert_eq!(
        e.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    oracle_aggregator_id.clone(),
                    Symbol::new(&e, "add_oracle"),
                    vec![&e, oracle_0_1_id.to_val()]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    let oracles = oracle_aggregator_client.oracles();
    assert_eq!(oracles.len(), 1);
    assert_oracle_config_equal(
        oracles.get_unchecked(0),
        OracleConfig {
            address: oracle_0_1_id.clone(),
            index: 0,
            resolution: 300,
            decimals: 9,
        },
    );

    // try and oracle when the list is full
    e.as_contract(&oracle_aggregator_id, || {
        let mut temp_oracles = Vec::<OracleConfig>::new(&e);
        for i in 0..20 {
            temp_oracles.push_back(OracleConfig {
                address: Address::generate(&e),
                index: i,
                resolution: 0,
                decimals: 0,
            });
        }
        storage::set_oracles(&e, &temp_oracles);
    });
    // - do test
    let result_full = oracle_aggregator_client.try_add_oracle(&oracle_2_id);
    assert_eq!(result_full.err(), Some(Ok(Error::from_contract_error(108))));
    // - revert asset list change
    e.as_contract(&oracle_aggregator_id, || {
        storage::set_oracles(&e, &oracles);
    });

    // add oracle_2
    oracle_aggregator_client.add_oracle(&oracle_2_id);
    let oracles = oracle_aggregator_client.oracles();
    assert_eq!(oracles.len(), 2);
    assert_oracle_config_equal(
        oracles.get_unchecked(1),
        OracleConfig {
            address: oracle_2_id.clone(),
            index: 1,
            resolution: 600,
            decimals: 6,
        },
    );

    // attempt to add an existing oracle
    let result_existing = oracle_aggregator_client.try_add_oracle(&oracle_0_1_id);
    assert_eq!(
        result_existing.err(),
        Some(Ok(Error::from_contract_error(106)))
    );

    // attempt to add an oracle that does not implement the interface
    let (oracle_aggregator_id_2, _) =
        create_oracle_aggregator(&e, &admin, &base, &decimals, &max_age);
    let result_bad_interface = oracle_aggregator_client.try_add_oracle(&oracle_aggregator_id_2);
    assert!(result_bad_interface.is_err());
}

#[test]
fn test_base_assets() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    e.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let decimals = 7;
    let max_age = 900;

    let asset_0 = Address::generate(&e);
    let asset_1 = Address::generate(&e);
    let asset_2 = Address::generate(&e);
    let symbol_2 = Symbol::new(&e, "wETH");

    // deploy two mock oracles
    // - setup oracle with all asset prices
    let oracle_0_1_id = Address::generate(&e);
    e.register_at(&oracle_0_1_id, MockPriceOracleWASM, ());
    let oracle_0_1 = MockPriceOracleClient::new(&e, &oracle_0_1_id);
    oracle_0_1.set_data(
        &Address::generate(&e),
        &MockAsset::Other(Symbol::new(&e, "BASE")),
        &Vec::from_array(
            &e,
            [
                MockAsset::Stellar(asset_0.clone()),
                MockAsset::Stellar(asset_1.clone()),
            ],
        ),
        &9,
        &300,
    );
    let oracle_0_1_norm_timestamp = e.ledger().timestamp() / 300 * 300;
    oracle_0_1.set_price(
        &vec![&e, 1_300_000_055, 2223_421_213_231, 0_900_120_123],
        &(oracle_0_1_norm_timestamp - 300),
    );
    oracle_0_1.set_price(
        &vec![&e, 1_200_000_055, 2123_421_213_231, 0_924_120_123],
        &oracle_0_1_norm_timestamp,
    );

    // - setup oracle with ASSET_2 price
    let oracle_2_id = Address::generate(&e);
    e.register_at(&oracle_2_id, MockPriceOracleWASM, ());
    let oracle_2 = MockPriceOracleClient::new(&e, &oracle_2_id);
    oracle_2.set_data(
        &Address::generate(&e),
        &MockAsset::Other(Symbol::new(&e, "BASE")),
        &Vec::from_array(&e, [MockAsset::Other(symbol_2.clone())]),
        &6,
        &600,
    );
    let oracle_2_norm_timestamp = e.ledger().timestamp() / 600 * 600;
    oracle_2.set_price(&vec![&e, 0_994_120], &(oracle_2_norm_timestamp - 600));
    oracle_2.set_price(&vec![&e, 1_024_025], &oracle_2_norm_timestamp);

    // deploy oracle aggregator
    let (oracle_aggregator_id, oracle_aggregator_client) =
        create_oracle_aggregator(&e, &admin, &base, &decimals, &max_age);
    assert_assets_equal(oracle_aggregator_client.base(), base.clone());
    assert_eq!(oracle_aggregator_client.decimals(), 7);
    assert_eq!(oracle_aggregator_client.max_age(), 900);

    e.jump(10);

    // add both oracles
    oracle_aggregator_client.add_oracle(&oracle_0_1_id);
    oracle_aggregator_client.add_oracle(&oracle_2_id);

    // add asset_0
    oracle_aggregator_client.add_asset(
        &Asset::Stellar(asset_0.clone()),
        &oracle_0_1_id,
        &Asset::Stellar(asset_0.clone()),
        &0,
    );

    // add asset_1 as a base_asset
    oracle_aggregator_client.add_base_asset(&Asset::Stellar(asset_1.clone()));
    assert_eq!(
        e.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    oracle_aggregator_id.clone(),
                    Symbol::new(&e, "add_base_asset"),
                    vec![&e, Asset::Stellar(asset_1.clone()).into_val(&e)]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    e.as_contract(&oracle_aggregator_id, || {
        let base_assets = storage::get_base_assets(&e);
        assert_eq!(base_assets.len(), 1);
        assert_assets_equal(
            base_assets.get_unchecked(0),
            Asset::Stellar(asset_1.clone()),
        );
    });
    let assets = oracle_aggregator_client.assets();
    assert_eq!(assets.len(), 2);
    assert_assets_equal(assets.get_unchecked(0), Asset::Stellar(asset_1.clone()));
    assert_assets_equal(assets.get_unchecked(1), Asset::Stellar(asset_0.clone()));

    e.jump(1);

    let base_price = oracle_aggregator_client
        .lastprice(&Asset::Stellar(asset_1.clone()))
        .unwrap();
    assert_eq!(base_price.price, 1_0000000);
    assert_eq!(base_price.timestamp, e.ledger().timestamp());

    // verify duplcate base asset cannot be added
    let result_dupe = oracle_aggregator_client.try_add_base_asset(&Asset::Stellar(asset_1.clone()));
    assert_eq!(result_dupe.err(), Some(Ok(Error::from_contract_error(103))));

    // verify real base asset cannot be added
    let result_base = oracle_aggregator_client.try_add_base_asset(&base);
    assert_eq!(result_base.err(), Some(Ok(Error::from_contract_error(103))));

    let result_base_2 = oracle_aggregator_client.try_add_asset(&base, &oracle_0_1_id, &base, &0);
    assert_eq!(
        result_base_2.err(),
        Some(Ok(Error::from_contract_error(103)))
    );

    // verify max base assets is checked
    e.as_contract(&oracle_aggregator_id, || {
        let mut base_assets = vec![&e];
        for _ in 0..10 {
            base_assets.push_back(Asset::Stellar(Address::generate(&e)));
        }
        storage::set_base_assets(&e, &base_assets);
    });
    let result_full = oracle_aggregator_client.try_add_base_asset(&Asset::Stellar(asset_2.clone()));
    assert_eq!(result_full.err(), Some(Ok(Error::from_contract_error(102))));
    e.as_contract(&oracle_aggregator_id, || {
        // reset base asset list
        storage::set_base_assets(&e, &vec![&e, Asset::Stellar(asset_1.clone())]);
    });

    // add asset_2 as a base_asset
    oracle_aggregator_client.add_base_asset(&Asset::Stellar(asset_2.clone()));
    e.as_contract(&oracle_aggregator_id, || {
        let base_assets = storage::get_base_assets(&e);
        assert_eq!(base_assets.len(), 2);
        assert_assets_equal(
            base_assets.get_unchecked(0),
            Asset::Stellar(asset_1.clone()),
        );
        assert_assets_equal(
            base_assets.get_unchecked(1),
            Asset::Stellar(asset_2.clone()),
        );
    });
    let assets = oracle_aggregator_client.assets();
    assert_eq!(assets.len(), 3);
    assert_assets_equal(assets.get_unchecked(0), Asset::Stellar(asset_1.clone()));
    assert_assets_equal(assets.get_unchecked(1), Asset::Stellar(asset_2.clone()));
    assert_assets_equal(assets.get_unchecked(2), Asset::Stellar(asset_0.clone()));

    // verify asset_1 can be given a config
    oracle_aggregator_client.add_asset(
        &Asset::Stellar(asset_1.clone()),
        &oracle_0_1_id,
        &Asset::Stellar(asset_1.clone()),
        &0,
    );
    let asset_1_config = oracle_aggregator_client
        .asset_configs()
        .get(Asset::Stellar(asset_1.clone()))
        .unwrap();
    assert_asset_config_equal(
        asset_1_config,
        AssetConfig {
            asset: Asset::Stellar(asset_1.clone()),
            oracle_index: 0,
            max_dev: 0,
        },
    );
    e.as_contract(&oracle_aggregator_id, || {
        let base_assets = storage::get_base_assets(&e);
        assert_eq!(base_assets.len(), 1);
        assert_assets_equal(
            base_assets.get_unchecked(0),
            Asset::Stellar(asset_2.clone()),
        );
    });
    let assets = oracle_aggregator_client.assets();
    assert_eq!(assets.len(), 3);
    assert_assets_equal(assets.get_unchecked(0), Asset::Stellar(asset_2.clone()));
    assert_assets_equal(assets.get_unchecked(1), Asset::Stellar(asset_0.clone()));
    assert_assets_equal(assets.get_unchecked(2), Asset::Stellar(asset_1.clone()));

    e.jump(1);

    // verify lastprice uses oracle
    let oracle_price = oracle_aggregator_client
        .lastprice(&Asset::Stellar(asset_1.clone()))
        .unwrap();
    assert_eq!(oracle_price.price, 2123_421_213_2);
    assert_eq!(oracle_price.timestamp, oracle_0_1_norm_timestamp);

    // verify asset_1 cannot be reset as a base asset
    let result_base = oracle_aggregator_client.try_add_base_asset(&Asset::Stellar(asset_1.clone()));
    assert_eq!(result_base.err(), Some(Ok(Error::from_contract_error(103))));

    // verify asset_2 can be given a config
    oracle_aggregator_client.add_asset(
        &Asset::Stellar(asset_2.clone()),
        &oracle_2_id,
        &Asset::Other(symbol_2.clone()),
        &20,
    );
    let asset_2_config = oracle_aggregator_client
        .asset_configs()
        .get(Asset::Stellar(asset_2.clone()))
        .unwrap();
    assert_asset_config_equal(
        asset_2_config,
        AssetConfig {
            asset: Asset::Other(symbol_2.clone()),
            oracle_index: 1,
            max_dev: 20,
        },
    );
    e.as_contract(&oracle_aggregator_id, || {
        let base_assets = storage::get_base_assets(&e);
        assert_eq!(base_assets.len(), 0);
    });
    let assets = oracle_aggregator_client.assets();
    assert_eq!(assets.len(), 3);
    assert_assets_equal(assets.get_unchecked(0), Asset::Stellar(asset_0.clone()));
    assert_assets_equal(assets.get_unchecked(1), Asset::Stellar(asset_1.clone()));
    assert_assets_equal(assets.get_unchecked(2), Asset::Stellar(asset_2.clone()));

    e.jump(1);

    // verify lastprice uses oracle
    let oracle_price = oracle_aggregator_client
        .lastprice(&Asset::Stellar(asset_2.clone()))
        .unwrap();
    assert_eq!(oracle_price.price, 1_024_0250);
    assert_eq!(oracle_price.timestamp, oracle_2_norm_timestamp);
}
