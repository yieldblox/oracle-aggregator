#![cfg(test)]

use crate::testutils::{setup_default_aggregator, EnvTestUtils};
use crate::types::Asset;
use soroban_sdk::{testutils::Address as _, vec, Address, Env, Symbol, Vec};

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
fn test_lastprice_retries_with_timestamp() {
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

    let recent_norm_time_1 = e.ledger().timestamp() / 300 * 300;
    oracle_1.set_price(
        &Vec::from_array(&e, [0_120000000, 1_010000000]),
        &(recent_norm_time_1 - 600),
    );
    oracle_1.set_price(&vec![&e], &(recent_norm_time_1 - 300));
    oracle_1.set_price(&vec![&e], &recent_norm_time_1);

    let recent_norm_time_2 = e.ledger().timestamp() / 600 * 600;
    oracle_2.set_price(
        &Vec::from_array(&e, [100_123_456]),
        &(recent_norm_time_2 - 600),
    );
    oracle_2.set_price(&vec![&e], &recent_norm_time_2);

    oracle_aggregator_client.add_asset(&asset_0, &oracle_1.address, &oracle_asset_0, &0);
    oracle_aggregator_client.add_asset(&asset_1, &oracle_1.address, &oracle_asset_1, &0);
    oracle_aggregator_client.add_asset(&asset_2, &oracle_2.address, &oracle_asset_2, &0);

    e.jump(10);

    let price_0 = oracle_aggregator_client.lastprice(&asset_0).unwrap();
    assert_eq!(price_0.price, 0_1200000);
    assert_eq!(price_0.timestamp, recent_norm_time_1 - 600);

    let price_1 = oracle_aggregator_client.lastprice(&asset_2).unwrap();
    assert_eq!(price_1.price, 100_123_456_0);
    assert_eq!(price_1.timestamp, recent_norm_time_2 - 600);
}

#[test]
fn test_lastprice_retry_exceeds_max_timestamp() {
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
        &(recent_norm_time - 1200),
    );
    oracle_1.set_price(
        &Vec::from_array(&e, [0_120000000, 1_010000000]),
        &(recent_norm_time - 900),
    );
    oracle_1.set_price(&Vec::from_array(&e, []), &e.ledger().timestamp());
    assert_eq!(oracle_1.last_timestamp(), recent_norm_time);

    oracle_2.set_price(
        &Vec::from_array(&e, [1010_000000]),
        &(e.ledger().timestamp()),
    );

    oracle_aggregator_client.add_asset(&asset_0, &oracle_1.address, &oracle_asset_0, &0);
    oracle_aggregator_client.add_asset(&asset_1, &oracle_1.address, &oracle_asset_1, &0);
    oracle_aggregator_client.add_asset(&asset_2, &oracle_2.address, &oracle_asset_2, &0);

    // jump 1 block to ensure the most recent price is > 900 seconds old
    e.jump(1);

    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    assert!(price_0.is_none());
}

#[test]
fn test_lastprice_retry_stops_if_over_max_age() {
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

    let recent_norm_time_2 = e.ledger().timestamp() / 600 * 600;
    oracle_2.set_price(&Vec::from_array(&e, [1_000_000]), &(recent_norm_time - 600));

    oracle_aggregator_client.add_asset(&asset_0, &oracle_1.address, &oracle_asset_0, &0);
    oracle_aggregator_client.add_asset(&asset_1, &oracle_1.address, &oracle_asset_1, &0);
    oracle_aggregator_client.add_asset(&asset_2, &oracle_2.address, &oracle_asset_2, &0);

    oracle_1.set_price(&Vec::from_array(&e, []), &recent_norm_time);
    oracle_1.set_price(&Vec::from_array(&e, []), &recent_norm_time);

    // jump 300 seconds
    e.jump_time(300);
    oracle_1.set_price(&Vec::from_array(&e, []), &(recent_norm_time + 300));

    // jump 300 seconds (600 seconds total)
    e.jump_time(300);
    oracle_1.set_price(&Vec::from_array(&e, []), &(recent_norm_time + 600));
    oracle_2.set_price(&Vec::from_array(&e, []), &(recent_norm_time_2 + 600));

    // last prices are not all ~900 seconds old. Jump 1 more to ensure we are over the max age
    e.jump(1);

    // validate both prices are not found, and oracle 2 (longer resolution) checks less
    // entries than oracle 1. This is a bit hacky but shows that the retry mechanism
    // stops based on the resolution of the oracle.
    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    let read_entries_0 = e.cost_estimate().resources().read_entries;
    assert!(price_0.is_none());

    let price_2 = oracle_aggregator_client.lastprice(&asset_2);
    let read_entries_2 = e.cost_estimate().resources().read_entries;
    assert!(price_2.is_none());

    assert!(read_entries_2 < read_entries_0);
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
fn test_lastprice_max_dev_checks_retries() {
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

    e.jump_time(900); // 900 sec between prices
    oracle_1.set_price(
        &Vec::from_array(&e, [1_900_000_000, 0_500_111_000]),
        &(norm_timestamp + 900),
    );

    e.jump_time(900); // 900 sec from frist price

    let price_0 = oracle_aggregator_client.lastprice(&asset_0).unwrap();
    assert_eq!(price_0.price, 1_900_000_0);
    assert_eq!(price_0.timestamp, norm_timestamp + 900);
}

#[test]
fn test_lastprice_max_dev_old_price_too_old() {
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

    e.jump_time(1200); // 1200 sec between prices
    oracle_1.set_price(
        &Vec::from_array(&e, [1_900_000_000, 0_500_111_000]),
        &(norm_timestamp + 1200),
    );

    e.jump_time(900); // 900 sec from frist price

    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    assert!(price_0.is_none());
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

    e.jump_time(900); // 900 sec between prices
    oracle_1.set_price(
        &Vec::from_array(&e, [1_900_000_000, 0_500_111_000]),
        &(norm_timestamp + 900),
    );

    e.jump_time(901); // 901 sec from first price

    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    assert!(price_0.is_none());
}
