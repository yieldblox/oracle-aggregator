#![cfg(test)]

use crate::types::Asset;

use crate::testutils::{create_oracle_aggregator, EnvTestUtils};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, IntoVal, Symbol, Val, Vec};

use super::snapshot;

#[test]
fn test_reflector() {
    let e = snapshot::env_from_snapshot();
    e.mock_all_auths();

    let usdc_address = Address::from_str(&e, snapshot::USDC);
    let usdc_oracle_asset = Asset::Stellar(usdc_address.clone());
    let usdc_asset = usdc_oracle_asset.clone();

    let xlm_address = Address::from_str(&e, snapshot::XLM);
    let xlm_oracle_asset = Asset::Stellar(xlm_address.clone());
    let xlm_asset = Asset::Stellar(Address::generate(&e));

    let eurc_address = Address::from_str(&e, snapshot::EURC);
    let eurc_oracle_asset = Asset::Stellar(eurc_address.clone());
    let eurc_asset = Asset::Other(Symbol::new(&e, "EURC"));

    let aqua_address = Address::from_str(&e, snapshot::AQUA);
    let aqua_oracle_asset = Asset::Stellar(aqua_address.clone());
    let aqua_asset = Asset::Stellar(Address::generate(&e));

    let usdglo_address = Address::from_str(&e, snapshot::USDGLO);
    let usdglo_oracle_asset = Asset::Stellar(usdglo_address.clone());
    let usdglo_asset = usdglo_oracle_asset.clone();

    let bombadil = Address::generate(&e);
    let reflector = Address::from_str(&e, snapshot::REFLECTOR);

    let (_, aggregator_client) =
        create_oracle_aggregator(&e, &bombadil, &usdc_oracle_asset, &7, &900);

    // setup oracle to use CALI reflector oracle
    aggregator_client.add_oracle(&reflector);

    // setup XLM to just fetch last price
    aggregator_client.add_asset(&xlm_asset, &reflector, &xlm_oracle_asset, &0);
    // setup EURC to verify the price did not deviate more than 10%
    aggregator_client.add_asset(&eurc_asset, &reflector, &eurc_oracle_asset, &10);
    // setup AQUA to just fetch the last price
    aggregator_client.add_asset(&aqua_asset, &reflector, &aqua_oracle_asset, &0);
    // setup USDGLO as a base asset
    aggregator_client.add_base_asset(&usdglo_asset);

    let mut round_timestamp = snapshot::LAST_UPDATE_TIMESTAMP.clone();
    e.jump_time(300);

    // set starting prices on the reflector contract
    round_timestamp += 300;
    set_reflector_prices(
        &e,
        0_3000000_0000000,
        1_1000000_0000000,
        0_0007123_5678900,
        1_0000001_0000000,
    );

    e.jump_time(60);

    // validate most recent prices used
    let usdc_price = aggregator_client.lastprice(&usdc_asset).unwrap();
    assert_eq!(usdc_price.price, 1_0000000);
    assert_eq!(usdc_price.timestamp, e.ledger().timestamp());
    let xlm_price = aggregator_client.lastprice(&xlm_asset).unwrap();
    assert_eq!(xlm_price.price, 0_3000000);
    assert_eq!(xlm_price.timestamp, round_timestamp);
    let eurc_price = aggregator_client.lastprice(&eurc_asset).unwrap();
    assert_eq!(eurc_price.price, 1_1000000);
    assert_eq!(eurc_price.timestamp, round_timestamp);
    let aqua_price = aggregator_client.lastprice(&aqua_asset).unwrap();
    assert_eq!(aqua_price.price, 0_0007123);
    assert_eq!(aqua_price.timestamp, round_timestamp);
    let usdglo_price = aggregator_client.lastprice(&usdglo_asset).unwrap();
    assert_eq!(usdglo_price.price, 1_0000000);
    assert_eq!(usdglo_price.timestamp, e.ledger().timestamp());

    e.jump_time(240);

    // set round 1 prices on the reflector contract
    // -> EURC and AQUA no price data
    round_timestamp += 300;
    set_reflector_prices(&e, 0_3100000_0000000, 0, 0, 1_0000011_0000000);

    // validate lastprice
    // -> EURC and AQUA did not have price data last round, but the prev round is in `max_age`
    let usdc_price = aggregator_client.lastprice(&usdc_asset).unwrap();
    assert_eq!(usdc_price.price, 1_0000000);
    assert_eq!(usdc_price.timestamp, e.ledger().timestamp());
    let xlm_price = aggregator_client.lastprice(&xlm_asset).unwrap();
    assert_eq!(xlm_price.price, 0_3100000);
    assert_eq!(xlm_price.timestamp, round_timestamp);
    let eurc_price = aggregator_client.lastprice(&eurc_asset).unwrap();
    assert_eq!(eurc_price.price, 1_1000000);
    assert_eq!(eurc_price.timestamp, round_timestamp - 300);
    let aqua_price = aggregator_client.lastprice(&aqua_asset).unwrap();
    assert_eq!(aqua_price.price, 0_0007123);
    assert_eq!(aqua_price.timestamp, round_timestamp - 300);
    let usdglo_price = aggregator_client.lastprice(&usdglo_asset).unwrap();
    assert_eq!(usdglo_price.price, 1_0000000);
    assert_eq!(usdglo_price.timestamp, e.ledger().timestamp());

    // set round 2 prices on the reflector contract
    e.jump_time(300);
    round_timestamp += 300;
    set_reflector_prices(
        &e,
        0_2900000_0000000,
        1_1100000_0000000,
        0_0007523_5678900,
        1_0000002_0000000,
    );

    // validate lastprice
    // -> EURC able to search more than 1 round in the past for old price
    let usdc_price = aggregator_client.lastprice(&usdc_asset).unwrap();
    assert_eq!(usdc_price.price, 1_0000000);
    assert_eq!(usdc_price.timestamp, e.ledger().timestamp());
    let xlm_price = aggregator_client.lastprice(&xlm_asset).unwrap();
    assert_eq!(xlm_price.price, 0_2900000);
    assert_eq!(xlm_price.timestamp, round_timestamp);
    let eurc_price = aggregator_client.lastprice(&eurc_asset).unwrap();
    assert_eq!(eurc_price.price, 1_1100000);
    assert_eq!(eurc_price.timestamp, round_timestamp);
    let aqua_price = aggregator_client.lastprice(&aqua_asset).unwrap();
    assert_eq!(aqua_price.price, 0_0007523);
    assert_eq!(aqua_price.timestamp, round_timestamp);
    let usdglo_price = aggregator_client.lastprice(&usdglo_asset).unwrap();
    assert_eq!(usdglo_price.price, 1_0000000);
    assert_eq!(usdglo_price.timestamp, e.ledger().timestamp());

    // set round 3 prices on the reflector contract
    // -> EURC price spiked past 10% deviation
    e.jump_time(300);
    round_timestamp += 300;
    set_reflector_prices(
        &e,
        0_2800000_0000000,
        1_2300000_0000000,
        0_0008023_5678900,
        1_0000003_0000000,
    );

    // set round 4 prices to ensure EURC can be fetched again
    e.jump_time(300);
    round_timestamp += 300;
    set_reflector_prices(
        &e,
        0_2700000_0000000,
        1_2400000_0000000,
        0_0008123_5678900,
        1_0000004_0000000,
    );

    // jump exactly max_age to ensure prices can still be fetched
    e.jump_time(900);

    // validate lastprice
    let usdc_price = aggregator_client.lastprice(&usdc_asset).unwrap();
    assert_eq!(usdc_price.price, 1_0000000);
    assert_eq!(usdc_price.timestamp, e.ledger().timestamp());
    let xlm_price = aggregator_client.lastprice(&xlm_asset).unwrap();
    assert_eq!(xlm_price.price, 0_2700000);
    assert_eq!(xlm_price.timestamp, round_timestamp);
    let eurc_price = aggregator_client.lastprice(&eurc_asset).unwrap();
    assert_eq!(eurc_price.price, 1_2400000);
    assert_eq!(eurc_price.timestamp, round_timestamp);
    let aqua_price = aggregator_client.lastprice(&aqua_asset).unwrap();
    assert_eq!(aqua_price.price, 0_0008123);
    assert_eq!(aqua_price.timestamp, round_timestamp);
    let usdglo_price = aggregator_client.lastprice(&usdglo_asset).unwrap();
    assert_eq!(usdglo_price.price, 1_0000000);
    assert_eq!(usdglo_price.timestamp, e.ledger().timestamp());

    // jump 1 sec over max age
    e.jump_time(1);

    // validate all lastprice using oracle price not found
    let usdc_price = aggregator_client.lastprice(&usdc_asset).unwrap();
    assert_eq!(usdc_price.price, 1_0000000);
    assert_eq!(usdc_price.timestamp, e.ledger().timestamp());
    let xlm_price = aggregator_client.lastprice(&xlm_asset);
    assert!(xlm_price.is_none());
    let eurc_price = aggregator_client.lastprice(&eurc_asset);
    assert!(eurc_price.is_none());
    let aqua_price = aggregator_client.lastprice(&aqua_asset);
    assert!(aqua_price.is_none());
    let usdglo_price = aggregator_client.lastprice(&usdglo_asset).unwrap();
    assert_eq!(usdglo_price.price, 1_0000000);
    assert_eq!(usdglo_price.timestamp, e.ledger().timestamp());

    // run another update
    round_timestamp += 900;
    set_reflector_prices(
        &e,
        0_2600000_0000000,
        1_2500000_0000000,
        0_0008223_5678900,
        1_0000005_0000000,
    );

    e.jump_time(100);

    // validate lastprice
    let usdc_price = aggregator_client.lastprice(&usdc_asset).unwrap();
    assert_eq!(usdc_price.price, 1_0000000);
    assert_eq!(usdc_price.timestamp, e.ledger().timestamp());
    let xlm_price = aggregator_client.lastprice(&xlm_asset).unwrap();
    assert_eq!(xlm_price.price, 0_2600000);
    assert_eq!(xlm_price.timestamp, round_timestamp);
    let eurc_price = aggregator_client.lastprice(&eurc_asset).unwrap();
    assert_eq!(eurc_price.price, 1_2500000);
    assert_eq!(eurc_price.timestamp, round_timestamp);
    let aqua_price = aggregator_client.lastprice(&aqua_asset).unwrap();
    assert_eq!(aqua_price.price, 0_0008223);
    assert_eq!(aqua_price.timestamp, round_timestamp);
    let usdglo_price = aggregator_client.lastprice(&usdglo_asset).unwrap();
    assert_eq!(usdglo_price.price, 1_0000000);
    assert_eq!(usdglo_price.timestamp, e.ledger().timestamp());

    // move usdglo from base to normal asset
    aggregator_client.add_asset(&usdglo_asset, &reflector, &usdglo_oracle_asset, &5);
    let usdglo_price = aggregator_client.lastprice(&usdglo_asset).unwrap();
    assert_eq!(usdglo_price.price, 1_0000005);
    assert_eq!(usdglo_price.timestamp, round_timestamp);

    // assert it can't be moved back
    let result = aggregator_client.try_add_base_asset(&usdglo_asset);
    assert!(result.is_err());
}

/// Set prices on the reflector contract for the most recent round based on the
/// current ledger timestamp.
fn set_reflector_prices(
    e: &Env,
    xlm_price: i128,
    eurc_price: i128,
    aqua_price: i128,
    usdglo_price: i128,
) {
    let timestamp: u64 = (e.ledger().timestamp() / 300u64) * 300u64 * 1000u64;
    let reflector = Address::from_str(&e, snapshot::REFLECTOR);
    let price_array: Vec<i128> = vec![
        e,
        1, // BTCLN
        aqua_price,
        1, // yUSDC
        1, // FIDR
        1, // SSLX
        1, // ARST
        1, // mykobo EURC
        xlm_price,
        1, // XRP
        eurc_price,
        1, // XRF
        usdglo_price,
        1, // CETES
        1, // USTRY
    ];
    let args: Vec<Val> = vec![e, price_array.into_val(e), timestamp.into_val(e)];
    e.invoke_contract::<Val>(&reflector, &Symbol::new(e, "set_price"), args);
}
