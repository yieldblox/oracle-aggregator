#![cfg(test)]

use crate::{
    contract::OracleAggregatorClient,
    types::{Asset, AssetConfig, OracleConfig},
};
use sep_40_oracle::testutils::{Asset as MockAsset, MockPriceOracleClient, MockPriceOracleWASM};
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, Env, Symbol, Vec,
};
pub mod oracle_aggregator {
    soroban_sdk::contractimport!(
        file = "./target/wasm32-unknown-unknown/optimized/oracle_aggregator.wasm"
    );
}

const ONE_DAY_LEDGERS: u32 = 24 * 60 * 60 / 5;

pub trait EnvTestUtils {
    /// Jump the env by the given amount of ledgers. Assumes 5 seconds per ledger.
    fn jump(&self, ledgers: u32);

    /// Jump the env by the given amount of time. Increments the sequence number by 1.
    fn jump_time(&self, time: u64);

    /// Set the ledger to the default LedgerInfo
    ///
    /// Time -> 1441065600 (Sept 1st, 2015 12:00:00 AM UTC)
    /// Sequence -> 100
    fn set_default_info(&self);
}

impl EnvTestUtils for Env {
    fn jump(&self, ledgers: u32) {
        self.ledger().set(LedgerInfo {
            timestamp: self.ledger().timestamp().saturating_add(ledgers as u64 * 5),
            protocol_version: 22,
            sequence_number: self.ledger().sequence().saturating_add(ledgers),
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 50 * ONE_DAY_LEDGERS,
            min_persistent_entry_ttl: 50 * ONE_DAY_LEDGERS,
            max_entry_ttl: 365 * ONE_DAY_LEDGERS,
        });
    }

    fn jump_time(&self, time: u64) {
        self.ledger().set(LedgerInfo {
            timestamp: self.ledger().timestamp().saturating_add(time),
            protocol_version: 22,
            sequence_number: self.ledger().sequence() + 1,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 50 * ONE_DAY_LEDGERS,
            min_persistent_entry_ttl: 50 * ONE_DAY_LEDGERS,
            max_entry_ttl: 365 * ONE_DAY_LEDGERS,
        });
    }

    fn set_default_info(&self) {
        self.ledger().set(LedgerInfo {
            timestamp: 1441065600, // Sept 1st, 2015 12:00:00 AM UTC
            protocol_version: 22,
            sequence_number: 100,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 50 * ONE_DAY_LEDGERS,
            min_persistent_entry_ttl: 50 * ONE_DAY_LEDGERS,
            max_entry_ttl: 365 * ONE_DAY_LEDGERS,
        });
    }
}

/// Deploy an oracle aggreator contract
pub fn create_oracle_aggregator<'a>(
    e: &Env,
    admin: &Address,
    base: &Asset,
    decimals: &u32,
    max_age: &u64,
) -> (Address, OracleAggregatorClient<'a>) {
    let oracle_aggregator_address = Address::generate(&e);
    e.register_at(
        &oracle_aggregator_address,
        oracle_aggregator::WASM,
        (admin, base.clone(), decimals, max_age),
    );
    let oracle_aggregator_client: OracleAggregatorClient<'a> =
        OracleAggregatorClient::new(&e, &oracle_aggregator_address);
    return (oracle_aggregator_address, oracle_aggregator_client);
}

/// Setup an oracle aggregator with default test setttings based on the current env timestamp.
///
/// ### Returns
/// Two oracle aggegator clients:
/// - (0 and 1 oracle, 2 oracle)
pub fn setup_default_aggregator<'a>(
    e: &Env,
    admin: &Address,
    base: &Asset,
    asset_0: &Asset,
    asset_1: &Asset,
    asset_2: &Asset,
) -> (
    OracleAggregatorClient<'a>,
    MockPriceOracleClient<'a>,
    MockPriceOracleClient<'a>,
) {
    // setup oracle with XLM and USDC proce
    let oracle_0_1_id = Address::generate(&e);
    e.register_at(&oracle_0_1_id, MockPriceOracleWASM, ());
    let oracle_0_1 = MockPriceOracleClient::new(&e, &oracle_0_1_id);
    oracle_0_1.set_data(
        &Address::generate(&e),
        &MockAsset::Other(Symbol::new(&e, "BASE")),
        &Vec::from_array(
            &e,
            [
                MockAsset::from(asset_0.clone()),
                MockAsset::from(asset_1.clone()),
            ],
        ),
        &9,
        &300,
    );

    let oracle_2_id = Address::generate(&e);
    e.register_at(&oracle_2_id, MockPriceOracleWASM, ());
    let oracle_2 = MockPriceOracleClient::new(&e, &oracle_2_id);
    oracle_2.set_data(
        &Address::generate(&e),
        &MockAsset::Other(Symbol::new(&e, "BASE")),
        &Vec::from_array(&e, [MockAsset::from(asset_2.clone())]),
        &6,
        &600,
    );

    let (_, aggregator_client) = create_oracle_aggregator(e, admin, base, &7, &900);
    aggregator_client.add_oracle(&oracle_0_1_id);
    aggregator_client.add_oracle(&oracle_2_id);
    return (aggregator_client, oracle_0_1, oracle_2);
}

pub fn assert_assets_equal(a: Asset, b: Asset) {
    match (a, b) {
        (Asset::Stellar(a), Asset::Stellar(b)) => assert_eq!(a, b),
        (Asset::Other(a), Asset::Other(b)) => assert_eq!(a, b),
        _ => assert!(false, "Asset type mismatch"),
    };
}

pub fn assert_asset_config_equal(a: AssetConfig, b: AssetConfig) {
    assert_assets_equal(a.asset, b.asset);
    assert_eq!(a.oracle_index, b.oracle_index);
    assert_eq!(a.max_dev, b.max_dev);
}

pub fn assert_oracle_config_equal(a: OracleConfig, b: OracleConfig) {
    assert_eq!(a.address, b.address);
    assert_eq!(a.index, b.index);
    assert_eq!(a.resolution, b.resolution);
    assert_eq!(a.decimals, b.decimals);
}

impl From<Asset> for MockAsset {
    fn from(asset: Asset) -> Self {
        match asset {
            Asset::Stellar(a) => MockAsset::Stellar(a),
            Asset::Other(a) => MockAsset::Other(a),
        }
    }
}
