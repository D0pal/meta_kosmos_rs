use anyhow::Result;
use gumdrop::Options;
use meta_address::get_rpc_info;
use meta_alchemy::{EvmSimulator, ReplayTransactionResult};
use meta_common::enums::{Network, RpcProvider};
use meta_util::ether::tx_hash_from_str;

#[derive(Debug, Clone, Options)]
struct Opts {
    help: bool,

    #[options(help = "tx hash")]
    tx_hash: String,

    #[options(help = "blockchain network, such as ETH, ARBI")]
    network: Option<Network>,

    #[options(help = "rpc provider, such as quick, ankr, custom, official")]
    provider: Option<RpcProvider>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse_args_default_or_exit();
    if opts.network.is_none() {
        panic!("must provide netwok");
    }

    if opts.tx_hash.is_empty() {
        panic!("must provide txhash");
    }

    if opts.provider.is_none() {
        panic!("must provide rpc provider");
    }

    let network = opts.network.unwrap();
    let rpc_provider = opts.provider.unwrap();
    let tx_hash = tx_hash_from_str(&opts.tx_hash);

    let rpc = get_rpc_info(network).unwrap();
    let ws_url = rpc.ws_urls.get(&rpc_provider);
    if ws_url.is_none() {
        panic!("provider {:?} not found", rpc_provider);
    }

    let simulator = EvmSimulator::new(ws_url.unwrap(), None).await;
    let ret = simulator.replay_transaction(tx_hash).await;

    match ret {
        Ok(ReplayTransactionResult::Success { gas_used, gas_refunded: _, output: _ }) => {
            println!("succuss gas_used {:?}", gas_used);
        }
        Ok(ReplayTransactionResult::Revert { gas_used, message }) => {
            println!("reverted msg {:?}", message);
        }
        Err(e) => eprintln!("e {:?}", e),
    }

    Ok(())
}
