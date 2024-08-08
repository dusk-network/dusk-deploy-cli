# dusk-deploy-cli
Tool for smart contracts' deployment to Dusk blockchain.

Minimal set of arguments, long argument names:
```sh
cargo r -- --contract-path ./test/alice.wasm --seed "spice property autumn primary undo innocent pole legend stereo mom eternal topic"
```

Minimal set of arguments, short argument names:
```sh
cargo r -- -c ./test/alice.wasm -s "spice property autumn primary undo innocent pole legend stereo mom eternal topic"
```

Full set of arguments:
```sh
cargo r -- --contract-path=./test/bob.wasm --seed="spice property autumn primary undo innocent pole legend stereo mom eternal topic" --config-path=./config.toml --gas-limit=100000000 --gas-price=1 --owner="2e3f56b01f7a43c274714a6d22b93164d561f1247a8cfa7a79eede21420438aa" --nonce=0 --args="3e"
```


```

ARGUMENTS:
    -c, --contract-path     Path to contract bytecode file (.wasm) to be deployed
    -s, --seed              Seed mneumonic phrase, a string of 12 words encoding your secret and public keys
      , --config-path       Path to config file containing data needed to establish blockchain connection
      , --gas-limit         Maximum number of gas points allowed to be used when executing the deployment transaction,
                            if omitted, a 500_000_000 default will be used
        --gas-price         Gas price to be used when executing the deployment transaction,
                            if omitted, price value 1 will be used
    -o, --owner             Hexadecimal string representing the owner of the contract
    -n, --nonce             Number used when calculating contract is, used when there is a need to deploy
                            multiple contracts with the same bytecode and owner, and/or to obtain
                            a vanity contract did,
                            if omittted, 0 will be used
    -a, --args              Optional argument passed to contract's constructor. Contract may have a constructor
                            method named 'init' which will be executed automatically upon deployment and may accept 
                            arguments. Argument must be passed in a form of a hexadecimal string representing an
                            rkyv serialization of the argument proper. Multiple arguments are serialized as a tuple.
                            If omitted, no argument will be passed to the constructor. If contract does not have a
                            constructor, this argument may be omitted.
    -b, --block_height      Optional starting block height. Scanning the blockchain for notes will be started from
                            this value. If omitted, scenning will be performed from block height zero. Note that
                            it may take a long time to scan the entire blockchain, so in order to limit the waiting 
                            time, user can enter a height from which the scan is to be started. The user needs to
                            know, at least approximately, above which height her unspent notes are located.
                             
```

Example configuration file for blockchain connection:

```
rusk_address = "http://127.0.0.1:8080"
prover_address = "http://127.0.0.1:8080"
```
