/*
 * Snapshot of mainnet for the CALI reflector contract.
 *
 * Timestamp: 1744044900
 * Ledger: 56510201
 *
 * Last price update: 1744044900
 * Resolution: 300 seconds
 * Assets:
 * BTCLN,AQUA,yUSDC,FIDR,SSLX,ARST,EURC,XLM,XRP,EURC,XRF,USDGLO,CETES,USTRY
 */

use soroban_sdk::{testutils::EnvTestConfig, Env};

pub const LAST_UPDATE_TIMESTAMP: u64 = 1744044900;

pub const REFLECTOR: &str = "CALI2BYU2JE6WVRUFYTS6MSBNEHGJ35P4AVCZYF3B6QOE3QKOB2PLE6M";

pub const USDC: &str = "CCW67TSZV3SSS2HXMBQ5JFGCKJNXKZM7UQUWUZPUTHXSTZLEO7SJMI75";
pub const EURC: &str = "CDTKPWPLOURQA2SGTKTUQOWRCBZEORB4BWBOMJ3D3ZTQQSGE5F6JBQLV";
pub const XLM: &str = "CAS3J7GYLGXMF6TDJBBYYSE3HQ6BBSMLNUQ34T6TZMYMW2EVH34XOWMA";
pub const AQUA: &str = "CAUIKL3IYGMERDRUN6YSCLWVAKIFG5Q4YJHUKM4S4NJZQIA3BAS6OJPK";
pub const USDGLO: &str = "CB226ZOEYXTBPD3QEGABTJYSKZVBP2PASEISLG3SBMTN5CE4QZUVZ3CE";

pub fn env_from_snapshot() -> Env {
    let mut env = Env::from_ledger_snapshot_file("./src/tests/reflector_snapshot.json");
    env.set_config(EnvTestConfig {
        capture_snapshot_at_drop: false,
    });
    env
}
