use ethers::contract::Abigen;

fn main() {
    println!("cargo:rerun-if-changed=./static/abis/**/*.json");
    println!("cargo:rerun-if-changed=./out/**/*.json");
    bindgen("ERC20", "./static/abis/token/ERC20.json");
    bindgen("UniswapV2Factory", "./static/abis/uniswap_v2/UniswapV2Factory.json");
    bindgen("UniswapV2Pair", "./static/abis/uniswap_v2/UniswapV2Pair.json");
    bindgen("UniswapV2Router02", "./static/abis/uniswap_v2/UniswapV2Router02.json");
    bindgen("WETH9", "./static/abis/uniswap_v2/WETH9.json");
    bindgen("MuteSwitchFactory", "./static/abis/mute_switch/factory.json");
    bindgen("Migration", "./out/Migration.sol/Migration.json");
    bindgen("FlashBotsRouter", "./out/FlashBotsRouter.sol/FlashBotsRouter.json");
}

fn bindgen(contract_name: &str, path:&str) {
    let bindings = Abigen::new(contract_name, path)
        .expect("could not instantiate Abigen")
        .generate()
        .expect("could not generage bindings");
    bindings
        .write_to_file(format!("./src/bindings/{}.rs", contract_name.to_lowercase()))
        .expect("could not write bindings to file");
}
