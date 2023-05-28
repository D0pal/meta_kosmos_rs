## frequently used cmd
```
cast chain-id --rpc-url ${rpc_url}
cast balance ${account} --rpc-url ${rpc_url}
cast rpc anvil_impersonateAccount ${user}
cast rpc anvil_setStorageAt ${contract_addr} ${slot_in_hex} ${val_in_hex} --rpc-url http://localhost:8545
cast send 0x258B6CB36ea949B5E6B6CcD5887694F26822791d --from 0x70997970c51812dc3a010c7d01b50e0d17dc79c8 "setCompleted(uint256)" 123456 --rpc-url http://localhost:8545
cast call --rpc-url ${url} 0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c \
  "balanceOf(address)(uint256)" 0xf887C32599acB9DA54627160B64339D7694eD5Dc
```