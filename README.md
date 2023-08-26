

## test
```
cargo test --package meta_address --lib -- tests::test_token_addr --exact --nocapture
```

## lint
```
cargo +nightly fmt
cargo fix
```

## scripts
- examples
```
cargo run -p meta_bots --example bloxroute
```


## pending issues
1. how to put examples folder under project root other than meta_bots/examples