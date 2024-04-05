use crate::{errors::OracleAggregatorErrors, storage};
use sep_40_oracle::{Asset, PriceData};
use soroban_sdk::{panic_with_error, Env};

pub fn check_valid_velocity(
    e: &Env,
    asset: &Asset,
    price: &PriceData,
    prev_price: &PriceData,
) -> bool {
    if prev_price.timestamp >= price.timestamp {
        panic_with_error!(&e, OracleAggregatorErrors::InvalidTimestamp)
    }

    let velocity_threshold = storage::get_velocity_threshold(&e);
    let percent_price_delta = (price.price - prev_price.price).abs() * 10000 / prev_price.price;
    let velocity = percent_price_delta * 10000 / (price.timestamp - prev_price.timestamp) as i128;
    if velocity as u32 > velocity_threshold {
        let circuit_breaker_timeout = storage::get_timeout(&e);
        storage::set_circuit_breaker_status(&e, &asset, &true);
        storage::set_circuit_breaker_timeout(
            &e,
            &asset,
            &(e.ledger().timestamp() + circuit_breaker_timeout),
        );
        return false;
    }
    return true;
}

pub fn check_circuit_breaker(e: &Env, asset: &Asset) {
    if storage::has_circuit_breaker(&e) && storage::get_circuit_breaker_status(&e, &asset) {
        if storage::get_circuit_breaker_timeout(&e, &asset) > e.ledger().timestamp() {
            panic_with_error!(&e, OracleAggregatorErrors::CircuitBreakerTripped);
        } else {
            storage::set_circuit_breaker_status(&e, &asset, &false);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::OracleAggregator;

    use super::*;
    use soroban_sdk::{
        testutils::{Ledger, LedgerInfo},
        Symbol,
    };

    #[test]
    fn test_check_valid_velocity() {
        let e = Env::default();
        e.mock_all_auths();
        e.ledger().set(LedgerInfo {
            timestamp: 123456 * 5,
            protocol_version: 20,
            sequence_number: 123456,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 10,
            min_persistent_entry_ttl: 10,
            max_entry_ttl: 3110400,
        });

        let aggregator = e.register_contract(None, OracleAggregator {});
        let asset = Asset::Other(Symbol::new(&e, "TEST"));
        let price = PriceData {
            price: 96000000,
            timestamp: 1234567,
        };
        let prev_price = PriceData {
            price: 80000000,
            timestamp: 1234267,
        };
        // 20% price change in 5 minutes
        let velocity_threshold = 66667;
        let timeout = 3600;
        e.as_contract(&aggregator, || {
            storage::set_velocity_threshold(&e, &velocity_threshold);
            storage::set_timeout(&e, &timeout);

            assert_eq!(check_valid_velocity(&e, &asset, &price, &prev_price), true);
            assert_eq!(storage::get_circuit_breaker_status(&e, &asset), false);
            assert_eq!(storage::get_circuit_breaker_timeout(&e, &asset), 0);

            let prev_price = PriceData {
                price: 79989999,
                timestamp: 1234267,
            };
            assert_eq!(check_valid_velocity(&e, &asset, &price, &prev_price), false);
            assert_eq!(storage::get_circuit_breaker_status(&e, &asset), true);
            assert_eq!(
                storage::get_circuit_breaker_timeout(&e, &asset),
                e.ledger().timestamp() + timeout
            );
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #106)")]
    fn test_check_valid_velocity_same_timestamp() {
        let e = Env::default();
        e.mock_all_auths();
        e.ledger().set(LedgerInfo {
            timestamp: 123456 * 5,
            protocol_version: 20,
            sequence_number: 123456,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 10,
            min_persistent_entry_ttl: 10,
            max_entry_ttl: 3110400,
        });

        let aggregator = e.register_contract(None, OracleAggregator {});
        let asset = Asset::Other(Symbol::new(&e, "TEST"));
        let price = PriceData {
            price: 96000000,
            timestamp: 1234567,
        };
        let prev_price = PriceData {
            price: 80000000,
            timestamp: 1234567,
        };
        // 20% price change in 5 minutes
        let velocity_threshold = 66667;
        let timeout = 3600;
        e.as_contract(&aggregator, || {
            storage::set_velocity_threshold(&e, &velocity_threshold);
            storage::set_timeout(&e, &timeout);

            check_valid_velocity(&e, &asset, &price, &prev_price);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #106)")]
    fn test_check_valid_velocity_prev_timestamp_greater() {
        let e = Env::default();
        e.mock_all_auths();
        e.ledger().set(LedgerInfo {
            timestamp: 123456 * 5,
            protocol_version: 20,
            sequence_number: 123456,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 10,
            min_persistent_entry_ttl: 10,
            max_entry_ttl: 3110400,
        });

        let aggregator = e.register_contract(None, OracleAggregator {});
        let asset = Asset::Other(Symbol::new(&e, "TEST"));
        let price = PriceData {
            price: 96000000,
            timestamp: 1234567,
        };
        let prev_price = PriceData {
            price: 80000000,
            timestamp: 1234568,
        };
        // 20% price change in 5 minutes
        let velocity_threshold = 66667;
        let timeout = 3600;
        e.as_contract(&aggregator, || {
            storage::set_velocity_threshold(&e, &velocity_threshold);
            storage::set_timeout(&e, &timeout);

            check_valid_velocity(&e, &asset, &price, &prev_price);
        });
    }

    #[test]
    fn test_check_circuit_breaker_no_action() {
        let e = Env::default();
        e.mock_all_auths();
        e.ledger().set(LedgerInfo {
            timestamp: 123456 * 5,
            protocol_version: 20,
            sequence_number: 123456,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 10,
            min_persistent_entry_ttl: 10,
            max_entry_ttl: 3110400,
        });

        let aggregator = e.register_contract(None, OracleAggregator {});
        let asset = Asset::Other(Symbol::new(&e, "TEST"));

        e.as_contract(&aggregator, || {
            storage::set_circuit_breaker(&e, &true);
            storage::set_timeout(&e, &3600);
            storage::set_circuit_breaker_status(&e, &asset, &false);
            check_circuit_breaker(&e, &asset);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #104)")]
    fn test_check_circuit_breaker_active_expect_panic() {
        let e = Env::default();
        e.mock_all_auths();
        e.ledger().set(LedgerInfo {
            timestamp: 123456 * 5,
            protocol_version: 20,
            sequence_number: 123456,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 10,
            min_persistent_entry_ttl: 10,
            max_entry_ttl: 3110400,
        });

        let aggregator = e.register_contract(None, OracleAggregator {});
        let asset = Asset::Other(Symbol::new(&e, "TEST"));

        e.as_contract(&aggregator, || {
            storage::set_circuit_breaker(&e, &true);
            storage::set_timeout(&e, &3600);
            storage::set_circuit_breaker_status(&e, &asset, &true);
            storage::set_circuit_breaker_timeout(&e, &asset, &(e.ledger().timestamp() + 1000));
            check_circuit_breaker(&e, &asset);
        });
    }

    #[test]
    fn test_check_circuit_breaker_timeout_over() {
        let e = Env::default();
        e.mock_all_auths();
        e.ledger().set(LedgerInfo {
            timestamp: 123456 * 5,
            protocol_version: 20,
            sequence_number: 123456,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 10,
            min_persistent_entry_ttl: 10,
            max_entry_ttl: 3110400,
        });

        let aggregator = e.register_contract(None, OracleAggregator {});
        let asset = Asset::Other(Symbol::new(&e, "TEST"));

        e.as_contract(&aggregator, || {
            storage::set_circuit_breaker(&e, &true);
            storage::set_timeout(&e, &3600);
            storage::set_circuit_breaker_status(&e, &asset, &true);
            storage::set_circuit_breaker_timeout(&e, &asset, &(e.ledger().timestamp() - 1));
            check_circuit_breaker(&e, &asset);
            assert_eq!(storage::get_circuit_breaker_status(&e, &asset), false);
        });
    }
}
