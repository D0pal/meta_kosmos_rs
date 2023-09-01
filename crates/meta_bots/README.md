## Mercury
atomic swap
```
cargo run --bin ${BIN} -- -h
cargo run --bin ${BIN} -- --network ${NETWORK} -q ${QUOTE} -b ${BASE} --dex-a ${DEX_A} --dex-b ${DEX_B} -u ${RPC_URL}
```

## Jupyter
sandwidth 
```
cargo run -p meta_bots --bin jupyter --release -- --dexs PANCAKE,BISWAP --network BSC
```

## Venus
```
cargo build -p meta_bots --release
cp ./target/release/venus ./venus
export ENV=prod 
cargo run -p meta_bots --bin venus --release -- -b ARB -q USD --network ARBI -d UniswapV3 -c BITFINEX
rm venus.log
nohup ./venus -b ARB -q USD --network ARBI -d UniswapV3 -c BITFINEX >venus.log 2>&1 &
```