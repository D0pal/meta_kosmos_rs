pub mod codec;
pub use codec::*;

pub mod hash;
pub use hash::*;

pub mod protocol;
pub use protocol::*;

mod wei;
pub use wei::*;

mod address;
pub use address::*;

use ethers::prelude::*;
use meta_common::enums::Network;

pub fn get_network_scan_url(network: Network, hash: TxHash) -> String {
    match network {
        Network::ARBI => format!("https://arbiscan.io/tx/{:?}", hash),
        _ => unimplemented!(),
    }
}
