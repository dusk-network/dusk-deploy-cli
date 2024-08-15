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
    -b, --block_height      Optional starting block height. Scanning the blockchain for notes will start from
                            this value. If omitted, scanning will be performed from block height zero. Note that
                            it may take a long time to scan the entire blockchain, so in order to limit the waiting 
                            time, user can enter a height from which the scan is to be started. The user needs to
                            know, at least approximately, above which height her unspent notes are located.
    -r, --relative_height   Optional relative starting block height. Scanning the blockchain for notes will start 
                            from current height minus this value. If omitted or current block height cannot be
                            obtained, absolute starting block height is assumed. This option, if present, overrides 
                            the absolute block height.

```

Example configuration file for blockchain connection:

```
rusk_address = "http://127.0.0.1:8080"
prover_address = "http://127.0.0.1:8080"
```



#Test Cases for Deployment System (DS)

##DS-01

1. Test Case ID: DS-01
2. Title: Deploying smart contracts to Dusk blockchain using Phoenix funds and Moonlight accounts.
3. Description: To validate that a substantial number of contracts have been correctly deployed and are operational, in a sense that they correctly perform state changing methods as well as queries.
4. You need to have the following:
   - a running Rusk node
   - a wallet with sufficient Phoenix funds and a seed to access them
   - 8 moonlight accounts with sufficient Dusk funds and 8 secret keys to access them
5. Test steps:
   1. Run a test tool which deploys N=2000 contracts using Moonlight funds.
   2. Run a test tool which verifies the contracts from step 1 by calling a state changing method and then calling a query to verify that state has been correctly changed.
   3. Make sure disk memory usage for state is as expected, make sure that Rusk node is stable during and after the test.
   4. Repeat the above procedure using Phoenix funds (it will be more time-consuming so you can assume N < 2000)
6. Teardown:
   1. Stop the Rusk node.

##DS-02

8. Test Case ID: DS-02
9. Title: Calling state-changing method of a non-deployed contract.
10. Description: To validate that a calling a contract which is not deploy does not destabilize the system and produces a correct error scenario.
11. You need to have the following:
    - a running Rusk node
    - a wallet with sufficient Phoenix funds and a seed to access them
    - moonlight account with sufficient Dusk funds and a secret key to access it
12. Test steps:
    1. Run a test tool which calls a state changing method.
    2. Make sure system is stable (by deploying a contract and calling its method) and that you've gotten a correct error message.
    4. Repeat the above procedure using Phoenix funds
13. Teardown:
    1. Stop the Rusk node.
   