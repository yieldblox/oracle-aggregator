# Oracle Aggregator

Example Oracle Aggregator that can be used with Blend pools.

Maps `Assets` to a respective oracle, so that multiple oracles can be used from a single contract. This contract is optimized for use with a Blend pool, so only `lastprice` is supported.

The oracle aggregator also contains an `admin` such that the `admin` has the ability to `block` and `unblock` price reads for an `Asset`.

 * `block`
    * Causes a price read of `Asset` to panic
 * `unblock`
    * Removes a block on `Asset`, and allows price reads to 

## Safety

Oracle Aggregator has not had an audit conducted. If an audit is conducted, it will appear here.

Oracle Aggregator is made available under the MIT License, which disclaims all warranties in relation to the project and which limits the liability of those that contribute and maintain the project, including Script3. You acknowledge that you are solely responsible for any use of Oracle Aggregator and you assume all risks associated with any such use.
