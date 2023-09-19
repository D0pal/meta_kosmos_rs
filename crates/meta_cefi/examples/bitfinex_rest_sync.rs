use meta_address::enums::Asset;
use meta_cefi::bitfinex::{account::WalletType, api::*};

fn main() {
    let AK = std::env::var("BTF_AK").expect("must provide BTF_AK");
    let SK: String = std::env::var("BTF_SK").expect("must provide BTF_SK");

    let btf = Bitfinex::new(Some(AK), Some(SK));
    // get_wallet_balance(&btf, vec![Asset::ARB, Asset::USD]);
    get_summary(&btf);

    // let start = Instant::now();

    // let elapsed = Instant::now().duration_since(start).as_millis();
    // println!("total elapsed {:?} ms, resp {:?}", elapsed, resp);
    // let now = std::time::Instant::now();
    // std::thread::sleep(Duration::from_secs(10));
    // println!("elapsed {:?}", now.elapsed());
}

fn get_summary(bitfinex: &Bitfinex) {
    bitfinex.account.get_summary();
}

fn get_wallet_balance(bitfinex: &Bitfinex, assets: Vec<Asset>) {
    let wallets = bitfinex.account.get_wallets().unwrap();
    for wallet in wallets {
        let asset = wallet.currency.parse::<Asset>().unwrap();
        if wallet.wallet_type == WalletType::Exchange && assets.contains(&asset) {
            println!("asset: {:?}, balance: {:?}", asset, wallet.balance);
        }
    }
}

fn submit_order(bitfinex: &Bitfinex) {
    let resp = bitfinex.orders.submit_market_order("tARBUSD", 200);
    println!("resp: {:?}", resp);
}
