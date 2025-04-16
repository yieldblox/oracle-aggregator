#![cfg(test)]

use crate::testutils::{create_oracle_aggregator, setup_default_aggregator, EnvTestUtils};
use crate::types::Asset;
use soroban_sdk::{testutils::Address as _, vec, Address, Env, Symbol, Vec};
use soroban_sdk::{IntoVal, Val};

use super::snapshot;

#[test]
fn test_lastprice() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));

    let oracle_asset_0 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_1 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_2 = Asset::Other(Symbol::new(&e, "wETH"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));
    let asset_2 = Asset::Stellar(Address::generate(&e));

    let (oracle_aggregator_client, oracle_1, oracle_2) = setup_default_aggregator(
        &e,
        &admin,
        &base,
        &oracle_asset_0,
        &oracle_asset_1,
        &oracle_asset_2,
    );

    oracle_1.set_price(
        &Vec::from_array(&e, [0_110000000, 1_000000000]),
        &e.ledger().timestamp(),
    );

    oracle_2.set_price(
        &Vec::from_array(&e, [1010_000000]),
        &(e.ledger().timestamp() - 600),
    );

    oracle_aggregator_client.add_asset(&asset_0, &oracle_1.address, &oracle_asset_0, &0);
    oracle_aggregator_client.add_asset(&asset_1, &oracle_1.address, &oracle_asset_1, &0);
    oracle_aggregator_client.add_asset(&asset_2, &oracle_2.address, &oracle_asset_2, &0);

    let price_0 = oracle_aggregator_client.lastprice(&asset_0).unwrap();
    assert_eq!(price_0.price, 0_1100000);
    assert_eq!(price_0.timestamp, e.ledger().timestamp());

    let price_1 = oracle_aggregator_client.lastprice(&asset_1).unwrap();
    assert_eq!(price_1.price, 1_0000000);
    assert_eq!(price_1.timestamp, e.ledger().timestamp());

    let price_2 = oracle_aggregator_client.lastprice(&asset_2).unwrap();
    assert_eq!(price_2.price, 1010_0000000);
    assert_eq!(price_2.timestamp, e.ledger().timestamp() - 600);
}

#[test]
#[should_panic(expected = "Error(Contract, #104)")]
fn test_lastprice_asset_not_found() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));

    let oracle_asset_0 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_1 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_2 = Asset::Other(Symbol::new(&e, "wETH"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));
    let asset_2 = Asset::Stellar(Address::generate(&e));

    let (oracle_aggregator_client, oracle_1, oracle_2) = setup_default_aggregator(
        &e,
        &admin,
        &base,
        &oracle_asset_0,
        &oracle_asset_1,
        &oracle_asset_2,
    );

    oracle_1.set_price(
        &Vec::from_array(&e, [0_110000000, 1_000000000]),
        &e.ledger().timestamp(),
    );

    oracle_2.set_price(
        &Vec::from_array(&e, [1010_000000]),
        &(e.ledger().timestamp() - 600),
    );

    oracle_aggregator_client.add_asset(&asset_0, &oracle_1.address, &oracle_asset_0, &0);
    oracle_aggregator_client.add_asset(&asset_1, &oracle_1.address, &oracle_asset_1, &0);
    oracle_aggregator_client.add_asset(&asset_2, &oracle_2.address, &oracle_asset_2, &0);

    oracle_aggregator_client.lastprice(&Asset::Other(Symbol::new(&e, "NOT_FOUND")));
}

#[test]
fn test_lastprice_exceeds_max_timestamp() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));

    let oracle_asset_0 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_1 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_2 = Asset::Other(Symbol::new(&e, "wETH"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));
    let asset_2 = Asset::Stellar(Address::generate(&e));

    let (oracle_aggregator_client, oracle_1, oracle_2) = setup_default_aggregator(
        &e,
        &admin,
        &base,
        &oracle_asset_0,
        &oracle_asset_1,
        &oracle_asset_2,
    );

    let recent_norm_time = e.ledger().timestamp() / 300 * 300;
    oracle_1.set_price(
        &Vec::from_array(&e, [0_120000000, 1_010000000]),
        &(recent_norm_time - 300),
    );
    oracle_1.set_price(
        &Vec::from_array(&e, [0_120000000, 1_010000000]),
        &recent_norm_time,
    );

    oracle_2.set_price(&Vec::from_array(&e, [1010_000000]), &e.ledger().timestamp());
    oracle_aggregator_client.add_asset(&asset_0, &oracle_1.address, &oracle_asset_0, &0);
    oracle_aggregator_client.add_asset(&asset_1, &oracle_1.address, &oracle_asset_1, &0);
    oracle_aggregator_client.add_asset(&asset_2, &oracle_2.address, &oracle_asset_2, &0);

    // jump 901 blocks to ensure the most recent price is > 900 seconds old
    e.jump_time(901);

    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    assert!(price_0.is_none());
}

#[test]
fn test_lastprice_base() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));

    let oracle_asset_0 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_1 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_2 = Asset::Other(Symbol::new(&e, "wETH"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));

    let (oracle_aggregator_client, oracle_1, _) = setup_default_aggregator(
        &e,
        &admin,
        &base,
        &oracle_asset_0,
        &oracle_asset_1,
        &oracle_asset_2,
    );

    let norm_timestamp = e.ledger().timestamp() / 300 * 300;
    oracle_1.set_price(
        &Vec::from_array(&e, [0_123_456_789, 2_000_111_000]),
        &norm_timestamp,
    );

    oracle_aggregator_client.add_asset(&asset_0, &oracle_1.address, &oracle_asset_0, &0);
    oracle_aggregator_client.add_base_asset(&asset_1);

    e.jump_time(100);

    let price_0 = oracle_aggregator_client.lastprice(&asset_0).unwrap();
    assert_eq!(price_0.price, 0_123_456_7);
    assert_eq!(price_0.timestamp, norm_timestamp);

    let price_1 = oracle_aggregator_client.lastprice(&asset_1).unwrap();
    assert_eq!(price_1.price, 1_0000000);
    assert_eq!(price_1.timestamp, e.ledger().timestamp());

    let price_base = oracle_aggregator_client.lastprice(&base).unwrap();
    assert_eq!(price_base.price, 1_0000000);
    assert_eq!(price_base.timestamp, e.ledger().timestamp());
}

#[test]
fn test_lastprice_base_no_price_history() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));

    let oracle_asset_0 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_1 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_2 = Asset::Other(Symbol::new(&e, "wETH"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));

    let (oracle_aggregator_client, oracle_1, _) = setup_default_aggregator(
        &e,
        &admin,
        &base,
        &oracle_asset_0,
        &oracle_asset_1,
        &oracle_asset_2,
    );

    let norm_timestamp = e.ledger().timestamp() / 300 * 300;
    oracle_1.set_price(
        &Vec::from_array(&e, [0_123_456_789, 2_000_111_000]),
        &norm_timestamp,
    );

    oracle_aggregator_client.add_asset(&asset_0, &oracle_1.address, &oracle_asset_0, &0);
    oracle_aggregator_client.add_base_asset(&asset_1);

    oracle_1.set_price(&Vec::new(&e), &(norm_timestamp + 300));

    e.jump_time(1800);

    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    assert!(price_0.is_none());

    let price_1 = oracle_aggregator_client.lastprice(&asset_1).unwrap();
    assert_eq!(price_1.price, 1_0000000);
    assert_eq!(price_1.timestamp, e.ledger().timestamp());

    let price_base = oracle_aggregator_client.lastprice(&base).unwrap();
    assert_eq!(price_base.price, 1_0000000);
    assert_eq!(price_base.timestamp, e.ledger().timestamp());
}

#[test]
fn test_lastprice_max_dev() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));

    let oracle_asset_0 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_1 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_2 = Asset::Other(Symbol::new(&e, "wETH"));
    let asset_0 = Asset::Stellar(Address::generate(&e));

    let (oracle_aggregator_client, oracle_1, _) = setup_default_aggregator(
        &e,
        &admin,
        &base,
        &oracle_asset_0,
        &oracle_asset_1,
        &oracle_asset_2,
    );

    let norm_timestamp = e.ledger().timestamp() / 300 * 300;
    oracle_1.set_price(
        &Vec::from_array(&e, [2_101_000_000, 0_101_111_000]),
        &(norm_timestamp - 300),
    );

    oracle_1.set_price(
        &Vec::from_array(&e, [2_100_000_000, 0_100_111_000]),
        &norm_timestamp,
    );

    oracle_aggregator_client.add_asset(&asset_0, &oracle_1.address, &oracle_asset_0, &10);

    e.jump_time(300);
    oracle_1.set_price(
        &Vec::from_array(&e, [2_300_000_000, 0_500_111_000]),
        &(norm_timestamp + 300),
    );

    e.jump_time(100);

    let price_0 = oracle_aggregator_client.lastprice(&asset_0).unwrap();
    assert_eq!(price_0.price, 2_300_000_0);
    assert_eq!(price_0.timestamp, norm_timestamp + 300);
}

#[test]
fn test_lastprice_max_dev_too_large() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));

    let oracle_asset_0 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_1 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_2 = Asset::Other(Symbol::new(&e, "wETH"));
    let asset_0 = Asset::Stellar(Address::generate(&e));

    let (oracle_aggregator_client, oracle_1, _) = setup_default_aggregator(
        &e,
        &admin,
        &base,
        &oracle_asset_0,
        &oracle_asset_1,
        &oracle_asset_2,
    );

    let norm_timestamp = e.ledger().timestamp() / 300 * 300;
    oracle_1.set_price(
        &Vec::from_array(&e, [2_101_000_000, 0_101_111_000]),
        &(norm_timestamp - 300),
    );

    oracle_1.set_price(
        &Vec::from_array(&e, [2_100_000_000, 0_100_111_000]),
        &norm_timestamp,
    );

    oracle_aggregator_client.add_asset(&asset_0, &oracle_1.address, &oracle_asset_0, &10);

    e.jump_time(300);
    oracle_1.set_price(
        &Vec::from_array(&e, [2_320_000_000, 0_500_111_000]),
        &(norm_timestamp + 300),
    );

    e.jump_time(100);

    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    assert!(price_0.is_none());
}

#[test]
fn test_lastprice_max_dev_too_large_abs() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));

    let oracle_asset_0 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_1 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_2 = Asset::Other(Symbol::new(&e, "wETH"));
    let asset_0 = Asset::Stellar(Address::generate(&e));

    let (oracle_aggregator_client, oracle_1, _) = setup_default_aggregator(
        &e,
        &admin,
        &base,
        &oracle_asset_0,
        &oracle_asset_1,
        &oracle_asset_2,
    );

    let norm_timestamp = e.ledger().timestamp() / 300 * 300;
    oracle_1.set_price(
        &Vec::from_array(&e, [2_101_000_000, 0_101_111_000]),
        &(norm_timestamp - 300),
    );

    oracle_1.set_price(
        &Vec::from_array(&e, [2_100_000_000, 0_100_111_000]),
        &norm_timestamp,
    );

    oracle_aggregator_client.add_asset(&asset_0, &oracle_1.address, &oracle_asset_0, &10);

    e.jump_time(300);
    oracle_1.set_price(
        &Vec::from_array(&e, [1_880_000_000, 0_500_111_000]),
        &(norm_timestamp + 300),
    );

    e.jump_time(100);

    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    assert!(price_0.is_none());
}

#[test]
fn test_lastprice_max_dev_fetches_4_rounds() {
    // mock oracle does not behave like reflector for `prices`
    // do edge cases tests for `prices` with snapshot here
    let e = snapshot::env_from_snapshot();
    e.mock_all_auths();
    let base = Asset::Other(Symbol::new(&e, "BASE"));

    let xlm_address = Address::from_str(&e, snapshot::XLM);
    let xlm_asset = Asset::Stellar(xlm_address.clone());

    let bombadil = Address::generate(&e);
    let reflector = Address::from_str(&e, snapshot::REFLECTOR);

    let (_, aggregator_client) = create_oracle_aggregator(&e, &bombadil, &base, &7, &900);

    aggregator_client.add_oracle(&reflector);

    aggregator_client.add_asset(&xlm_asset, &reflector, &xlm_asset, &10);
    let mut round_timestamp = snapshot::LAST_UPDATE_TIMESTAMP.clone();
    e.jump_time(300);

    // set starting prices on the reflector contract
    round_timestamp += 300;
    set_reflector_prices(&e, 0_3000000_0000000);

    // TEST: Verify lastprice can verify last price if it is 4 rounds ago

    // skip 3 rounds
    e.jump_time(900);

    round_timestamp += 900;
    set_reflector_prices(&e, 0_3100000_0000000);

    e.jump_time(100);

    let xlm_price = aggregator_client.lastprice(&xlm_asset).unwrap();
    assert_eq!(xlm_price.price, 0_3100000);
    assert_eq!(xlm_price.timestamp, round_timestamp);

    // TEST: Verify lastprice returns none if last price is > 4 rounds ago
    e.jump_time(200);
    e.jump_time(900);
    set_reflector_prices(&e, 0_3200000_0000000);

    let xlm_price = aggregator_client.lastprice(&xlm_asset);
    assert!(xlm_price.is_none());
}

#[test]
fn test_lastprice_max_dev_first_price_too_old() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));

    let oracle_asset_0 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_1 = Asset::Stellar(Address::generate(&e));
    let oracle_asset_2 = Asset::Other(Symbol::new(&e, "wETH"));
    let asset_0 = Asset::Stellar(Address::generate(&e));

    let (oracle_aggregator_client, oracle_1, _) = setup_default_aggregator(
        &e,
        &admin,
        &base,
        &oracle_asset_0,
        &oracle_asset_1,
        &oracle_asset_2,
    );

    let norm_timestamp = e.ledger().timestamp() / 300 * 300;
    oracle_1.set_price(
        &Vec::from_array(&e, [2_101_000_000, 0_101_111_000]),
        &(norm_timestamp - 300),
    );

    oracle_1.set_price(
        &Vec::from_array(&e, [2_100_000_000, 0_100_111_000]),
        &norm_timestamp,
    );

    oracle_aggregator_client.add_asset(&asset_0, &oracle_1.address, &oracle_asset_0, &10);

    e.jump_time(300); // 300 sec between prices
    oracle_1.set_price(
        &Vec::from_array(&e, [1_900_000_000, 0_500_111_000]),
        &(norm_timestamp + 300),
    );

    e.jump_time(900); // 900 sec from first price

    let price_0 = oracle_aggregator_client.lastprice(&asset_0).unwrap();
    assert_eq!(price_0.price, 1_900_000_0);
    assert_eq!(price_0.timestamp, norm_timestamp + 300);

    e.jump_time(1);
    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    assert!(price_0.is_none());
}

/// Set prices on the reflector contract for the most recent round based on the
/// current ledger timestamp.
fn set_reflector_prices(e: &Env, xlm_price: i128) {
    let timestamp: u64 = (e.ledger().timestamp() / 300u64) * 300u64 * 1000u64;
    let reflector = Address::from_str(&e, snapshot::REFLECTOR);
    let price_array: Vec<i128> = vec![
        e, 1, // BTCLN
        1, // AQUA,
        1, // yUSDC
        1, // FIDR
        1, // SSLX
        1, // ARST
        1, // mykobo EURC
        xlm_price, 1, // XRP
        1, // EURC
        1, // XRF
        1, // USDGLO
        1, // CETES
        1, // USTRY
    ];
    let args: Vec<Val> = vec![e, price_array.into_val(e), timestamp.into_val(e)];
    e.invoke_contract::<Val>(&reflector, &Symbol::new(e, "set_price"), args);
}
