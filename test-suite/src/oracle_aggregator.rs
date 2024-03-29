use oracle_aggregator::{
    types::{Asset, OracleConfig},
    OracleAggregator, OracleAggregatorClient,
};
use soroban_sdk::{Address, Env, Vec};
mod oracle_aggregator_wasm {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/optimized/oracle_aggregator.wasm"
    );
}
use oracle_aggregator_wasm::WASM as OracleAggregatorWasm;

pub fn create_oracle_aggregator<'a>(
    e: &Env,
    admin: &Address,
    oracles: &Vec<Address>,
    oracle_configs: &Vec<OracleConfig>,
    base: &Asset,
    outlier_threshold: &u32,
) -> (Address, OracleAggregatorClient<'a>) {
    let oracle_aggregator_address = e.register_contract(None, OracleAggregator {});
    let oracle_aggregator_client: OracleAggregatorClient<'a> =
        OracleAggregatorClient::new(&e, &oracle_aggregator_address);
    oracle_aggregator_client.initialize(
        admin,
        oracles,
        oracle_configs,
        &7,
        base,
        outlier_threshold,
    );
    return (oracle_aggregator_address, oracle_aggregator_client);
}

pub fn create_oracle_aggregator_wasm<'a>(
    e: &Env,
    admin: &Address,
    oracles: &Vec<Address>,
    oracle_configs: &Vec<OracleConfig>,
    base: &Asset,
    outlier_threshold: &u32,
) -> (Address, OracleAggregatorClient<'a>) {
    let oracle_aggregator_address = e.register_contract_wasm(None, OracleAggregatorWasm);
    let oracle_aggregator_client: OracleAggregatorClient<'a> =
        OracleAggregatorClient::new(&e, &oracle_aggregator_address);
    oracle_aggregator_client.initialize(
        admin,
        oracles,
        oracle_configs,
        &7,
        base,
        outlier_threshold,
    );
    return (oracle_aggregator_address, oracle_aggregator_client);
}
