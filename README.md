# Oracle Aggregator

Example Oracle Aggregator that can be used with Blend pools. This contract allows one Blend pool to access multiple oracle prices sources from one location via `lastprice` method.

### Supported Oracles

This Oracle aggregator is made specifically to work with Reflector-like oracles.

### Config

The oracle aggregator uses some global configuration defined through the constructor:

* admin `Address` - The admin has the ability to add additional assets to the oracle aggregator. This should be done cautiosly, as they can never be removed or edited.
* base `Asset` - The base asset the oracle aggregator will report prices in
* decimals `u32` - The decimals the oracle aggregator will report prices in
* max_age `u64` - The maximum age (in seconds) of a fetched price the oracle aggregator will return from the current ledger timestamp. This must be between 360s (6m) and 3600s (60m).

Each supported asset is defined via an AssetConfig:

**Asset Config**
* asset `Asset` - The asset to be used when fetching the price from the source oracle
* oracle_index `u32` - The index of the source oracle used
* max_dev `u32` - The maximum deviation allowed for two consecutive price updates, as a percentage with 0 decimals (e.g. 5 => 5%). If this is 0 or >=100. the oracle will just fetch the last price from the source oracle.

Up to 20 additional assets can be supported.

**Base-like Assets**

The agggregator can also support multiple base assets. These are tokens that, while they might not have a safe oracle price yet, can be redeemed 1-to-1 for the base asset. Thus, the aggregator will always report a fixed point price of `1` with `decimals` decimals.

This should be used sparingly, as it assumes a lot of trust for the issuer of the asset that it can always be redeemed 1-to-1 for the asset the oracle reports the price in.

If an additional base asset is added, it can be given an `AssetConfig` at any time by the admin.

Assets that have an `AssetConfig` cannot be set as base assets.

### Last Price Method

The aggregator attempts to fetch the price from the source oracle as defined by the internal `oracles` and the `oracle_index` within the `AssetConfig`.

1. If the `Asset` is the base asset, or is a base-like asset, a price of 1 is returned, as a fixed point number with `decimals` decimals, and the current timestamp.
2. The last round timestamp is fetched from the source oracle with `last_timestamp()`
3. If last round timestamp is older than `max_age`, `None` will be returned
4. If the asset is not configured to check max_dev, the price for `Asset` is attempted to be fetched from the source oracle based on the last round timestamp with `price(asset, last_timestamp)`.
5. If the asset is configured to check max_dev, the last 4 rounds will be fetched with `prices(asset, 4)`
    * If the aggregator does not get at least 2 prices or the most recent 2 prices break the deviation check, return `None`
6. If the price from 4/5'stimestamp is within `max_age`, return `price`

## Safety

Oracle Aggregator has not had an audit conducted. If an audit is conducted, it will appear here.

Oracle Aggregator is made available under the MIT License, which disclaims all warranties in relation to the project and which limits the liability of those that contribute and maintain the project, including Script3. You acknowledge that you are solely responsible for any use of Oracle Aggregator and you assume all risks associated with any such use.
