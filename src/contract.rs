use crate::{errors::OracleAggregatorErrors, storage};
use sep_40_oracle::{Asset, PriceData, PriceFeedClient, PriceFeedTrait};
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env, Symbol, Vec};

#[contract]
pub struct OracleAggregator;

#[contractimpl]
impl PriceFeedTrait for OracleAggregator {
    fn resolution(e: Env) -> u32 {
        panic_with_error!(e, OracleAggregatorErrors::NotImplemented);
    }

    fn price(e: Env, _asset: Asset, _timestamp: u64) -> Option<PriceData> {
        panic_with_error!(e, OracleAggregatorErrors::NotImplemented);
    }

    fn prices(e: Env, _asset: Asset, _records: u32) -> Option<Vec<PriceData>> {
        panic_with_error!(e, OracleAggregatorErrors::NotImplemented);
    }

    fn base(e: Env) -> Asset {
        storage::get_base(&e)
    }

    fn decimals(e: Env) -> u32 {
        storage::get_decimals(&e)
    }

    fn assets(e: Env) -> Vec<Asset> {
        let usdc = storage::get_usdc(&e);
        let mut oracle_assets = PriceFeedClient::new(&e, &storage::get_default_oracle(&e)).assets();
        oracle_assets.push_back(Asset::Stellar(usdc));
        oracle_assets
    }

    fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        storage::extend_instance(&e);
        if storage::get_blocked_status(&e, &asset) {
            panic_with_error!(&e, OracleAggregatorErrors::AssetBlocked);
        }
        let usdc = storage::get_usdc(&e);
        match asset {
            Asset::Stellar(ref a) if a.clone() == usdc => {
                let oracle = PriceFeedClient::new(&e, &storage::get_usdc_oracle(&e));
                oracle.lastprice(&Asset::Other(Symbol::new(&e, "USDC")))
            }
            _ => {
                let oracle = PriceFeedClient::new(&e, &storage::get_default_oracle(&e));
                oracle.lastprice(&asset)
            }
        }
    }
}

#[contractimpl]
impl OracleAggregator {
    /// Initialize the contract with the admin and the oracle configurations
    ///
    /// ### Arguments
    /// * `admin` - The address of the admin
    /// * `usdc` - The address of the USDC token
    /// * `usdc_oracle` - The address of the USDC oracle
    /// * `default_oracle` - The address of the oracle for all non-USDC assets
    ///
    /// ### Errors
    /// * `AlreadyInitialized` - The contract has already been initialized
    pub fn initialize(
        e: Env,
        admin: Address,
        usdc: Address,
        usdc_oracle: Address,
        default_oracle: Address,
    ) {
        if storage::get_is_init(&e) {
            panic_with_error!(&e, OracleAggregatorErrors::AlreadyInitialized);
        }

        storage::extend_instance(&e);
        storage::set_is_init(&e);
        storage::set_admin(&e, &admin);
        storage::set_usdc(&e, &usdc);
        storage::set_usdc_oracle(&e, &usdc_oracle);
        storage::set_default_oracle(&e, &default_oracle);

        let default_oracle = PriceFeedClient::new(&e, &default_oracle);
        let base = default_oracle.base();
        let decimals = default_oracle.decimals();
        let usdc_decimals = PriceFeedClient::new(&e, &usdc_oracle).decimals();
        if usdc_decimals != decimals {
            panic_with_error!(&e, OracleAggregatorErrors::InvalidOracleConfig);
        }

        storage::set_base(&e, &base);
        storage::set_decimals(&e, &decimals);
    }

    /// (Admin only) Block an asset
    ///
    /// ### Arguments
    /// * `asset` - The asset to block
    ///
    /// ### Errors
    /// * `AssetNotFound` - The asset is not found
    pub fn block(e: Env, asset: Asset) {
        let admin = storage::get_admin(&e);
        admin.require_auth();

        storage::set_blocked_status(&e, &asset, &true);
    }

    /// (Admin only) Unblock an asset
    ///
    /// ### Arguments
    /// * `asset` - The asset to unblock
    ///
    /// ### Errors
    /// * `AssetNotFound` - The asset is not found
    pub fn unblock(e: Env, asset: Asset) {
        let admin = storage::get_admin(&e);
        admin.require_auth();

        storage::set_blocked_status(&e, &asset, &false);
    }

    /// (Admin only) Set the admin address
    ///
    /// ### Arguments
    /// * `admin` - The new admin address
    pub fn set_admin(e: Env, admin: Address) {
        let current_admin = storage::get_admin(&e);
        current_admin.require_auth();

        storage::set_admin(&e, &admin);
    }
}
