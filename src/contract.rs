use crate::{
    errors::OracleAggregatorErrors,
    price_data::get_price,
    storage,
    types::{Asset, AssetConfig, OracleConfig, PriceData, PriceFeedClient},
};
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env, Map, Vec};

#[contract]
pub struct OracleAggregator;

#[contractimpl]
impl OracleAggregator {
    // Initialize the oracle aggregator contract.
    //
    // ### Arguments
    // * `admin` - The address of the admin
    // * `base` - The address of the base asset the oracle will report in
    // * `decimals` - The decimals the oracle will report in
    // * `max_age` - The maximum time the oracle will look back for a price (in seconds)
    //
    // ### Errors
    // * `InvalidMaxAge` - The max age is not between 360 (6m) and 3600 (60m)
    pub fn __constructor(e: Env, admin: Address, base: Asset, decimals: u32, max_age: u64) {
        storage::extend_instance(&e);
        storage::set_admin(&e, &admin);
        storage::set_base(&e, &base);
        storage::set_decimals(&e, &decimals);
        if max_age < 360 || max_age > 3600 {
            panic_with_error!(&e, OracleAggregatorErrors::InvalidMaxAge);
        }
        storage::set_max_age(&e, &max_age);
    }

    /**** Read Only *****/

    // Fetch the supported oracles
    pub fn oracles(e: Env) -> Vec<OracleConfig> {
        storage::get_oracles(&e)
    }

    // Fetch the configuration of an asset
    pub fn asset_configs(e: Env) -> Map<Asset, AssetConfig> {
        storage::get_asset_configs(&e)
    }

    // Fetch the max age of a price
    pub fn max_age(e: Env) -> u64 {
        storage::get_max_age(&e)
    }

    // Fetch the base asset
    pub fn base(e: Env) -> Asset {
        storage::get_base(&e)
    }

    // Fetch the decimals the oracle will report in
    pub fn decimals(e: Env) -> u32 {
        storage::get_decimals(&e)
    }

    // Fetch the list of assets the oracle supports
    pub fn assets(e: Env) -> Vec<Asset> {
        let asset_configs = storage::get_asset_configs(&e);
        let mut base_assets = storage::get_base_assets(&e);
        base_assets.append(&asset_configs.keys());
        base_assets
    }

    // Fetch the last price of the Asset based on the asset config.
    //
    // ### Arguments
    // * `asset` - The asset to fetch the price for
    //
    // ### Returns
    // * The lastprice from the source oracle
    // * None if the price cannot be resolved, or is outside the configured bounds
    //
    // ### Errors
    // * `AssetNotFound` - The asset is not in the list of assets
    // * `OracleNotFound` - The oracle is not in the list of oracles
    pub fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        storage::extend_instance(&e);

        let base_asset = storage::get_base(&e);
        let base_assets = storage::get_base_assets(&e);
        if base_assets.contains(&asset) || asset == base_asset {
            let decimals = storage::get_decimals(&e);
            return Some(PriceData {
                price: 10i128.pow(decimals),
                timestamp: e.ledger().timestamp(),
            });
        }

        let configs = storage::get_asset_configs(&e);
        let oracles = storage::get_oracles(&e);
        if let Some(config) = configs.get(asset) {
            if let Some(oracle) = oracles.get(config.oracle_index) {
                crate::price_data::get_price(&e, &oracle, &config)
            } else {
                panic_with_error!(&e, OracleAggregatorErrors::OracleNotFound);
            }
        } else {
            panic_with_error!(&e, OracleAggregatorErrors::AssetNotFound);
        }
    }

    /***** Admin Functions *****/

    // (Admin Only) Set the admin address
    //
    // ### Arguments
    // * `admin` - The address of the new admin
    pub fn set_admin(e: Env, admin: Address) {
        storage::get_admin(&e).require_auth();
        storage::set_admin(&e, &admin);
    }

    // (Admin Only) Add an oracle to the oracle aggregator
    //
    // ### Arguments
    // * `oracle_id` - The address of the oracle
    //
    // ### Errors
    // * `OracleAlreadyExists` - The oracle already exists
    pub fn add_oracle(e: Env, oracle_id: Address) {
        storage::get_admin(&e).require_auth();
        storage::extend_instance(&e);
        let mut oracles = storage::get_oracles(&e);
        for oracle in oracles.iter() {
            if oracle.address == oracle_id {
                panic_with_error!(&e, OracleAggregatorErrors::OracleExists);
            }
        }
        if oracles.len() >= 10 {
            panic_with_error!(&e, OracleAggregatorErrors::MaxOraclesExceeded);
        }

        let oracle_client = PriceFeedClient::new(&e, &oracle_id);
        let oracle_config = OracleConfig {
            address: oracle_id.clone(),
            index: oracles.len() as u32,
            resolution: oracle_client.resolution(),
            decimals: oracle_client.decimals(),
        };
        oracles.push_back(oracle_config);
        storage::set_oracles(&e, &oracles);
    }

    // (Admin Only) Add an asset to the oracle aggregator
    //
    // ### Arguments
    // * `asset` - The asset to add
    // * `oracle_id` - The address of the oracle
    // * `oracle_asset` - The asset used to fetch the oracle price
    // * `max_dev` - The maximum deviation allowed for a price, as a percentage with 0 decimals. If this is 0,
    //               the oracle will just fetch the last price within the resolution time.
    //
    // ### Errors
    // * `InvalidAssetOracle` - Unable to fetch a price for the oracle asset
    // * `AssetExists` - The asset already exists
    //
    // ### Returns
    // The price data of the asset as would be returned by `self.lastprice`. This is useful
    // for simulation to verify the asset was added correctly.
    pub fn add_asset(
        e: Env,
        asset: Asset,
        oracle_id: Address,
        oracle_asset: Asset,
        max_dev: u32,
    ) -> PriceData {
        storage::get_admin(&e).require_auth();
        storage::extend_instance(&e);

        // verify asset list is not full and the asset has not already been added
        let mut configs = storage::get_asset_configs(&e);
        if configs.contains_key(asset.clone()) {
            panic_with_error!(&e, OracleAggregatorErrors::AssetExists);
        } else if configs.len() >= 20 {
            panic_with_error!(&e, OracleAggregatorErrors::MaxAssetsExceeded);
        }

        let base_asset = storage::get_base(&e);
        if asset == base_asset {
            panic_with_error!(&e, OracleAggregatorErrors::AssetExists);
        }

        let oracles = storage::get_oracles(&e);
        let mut oracle_config: Option<OracleConfig> = None;
        for oracle in oracles.iter() {
            if oracle.address == oracle_id {
                oracle_config = Some(oracle);
                break;
            }
        }
        let oracle_config = oracle_config.unwrap_or_else(|| {
            panic_with_error!(&e, OracleAggregatorErrors::OracleNotFound);
        });

        let config = AssetConfig {
            asset: oracle_asset,
            oracle_index: oracle_config.index,
            max_dev,
        };
        let price = get_price(&e, &oracle_config, &config);
        if let Some(price) = price {
            // able to fetch a price for the asset, add asset and return price
            // if asset is currently on the base assets list, remove it
            let mut base_assets = storage::get_base_assets(&e);
            if let Some(index) = base_assets.first_index_of(asset.clone()) {
                base_assets.remove(index);
                storage::set_base_assets(&e, &base_assets);
            }
            configs.set(asset.clone(), config);
            storage::set_asset_configs(&e, &configs);
            return price;
        } else {
            panic_with_error!(&e, OracleAggregatorErrors::InvalidAssetOracle);
        }
    }

    // (Admin Only) Add an asset that reports a price of 1 base asset. This should be used
    // sparingly and only for assets that are redeemable for the base asset, and no safe oracle exists.
    //
    // ### Arguments
    // * `base` - The asset to add
    pub fn add_base_asset(e: Env, base: Asset) {
        storage::get_admin(&e).require_auth();
        storage::extend_instance(&e);

        let base_asset = storage::get_base(&e);
        if base == base_asset {
            panic_with_error!(&e, OracleAggregatorErrors::AssetExists);
        }

        let mut base_assets = storage::get_base_assets(&e);
        if base_assets.contains(&base) {
            panic_with_error!(&e, OracleAggregatorErrors::AssetExists);
        }
        if base_assets.len() >= 10 {
            panic_with_error!(&e, OracleAggregatorErrors::MaxAssetsExceeded);
        }

        let configs = storage::get_asset_configs(&e);
        if configs.contains_key(base.clone()) {
            panic_with_error!(&e, OracleAggregatorErrors::AssetExists);
        }

        base_assets.push_back(base);
        storage::set_base_assets(&e, &base_assets);
    }
}
