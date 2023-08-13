A personal project i'm using to improve in rust.

# EVM SIMULATOR WRITTEN IN RUST

Evm simulator with suppport for ERC20, ERC721 and ERC1155

## It detects the following in each simulated transaction

- ERC20 Transfers and TransferFrom
- ERC20 Approvals
- ERC721 Tranfers and TransferFrom
- ERC721 ApprovalForAll
- ERC1155 TransferSingle and TransferBatch
- ERC1155 ApprovalForAll

### To test, run this in your terminal

```zsh
cargo run 0x3B059f15059d976cA189165ee36d75Cb18249daf 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D 0x791ac94700000000000000000000000000000000000000000000021e19e0c9bab24000000000000000000000000000000000000000000000000000000191112d9c55be9100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000003b059f15059d976ca189165ee36d75cb18249daf0000000000000000000000000000000000000000000000000000000064d8924a0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000930dac667ca8ac9166c93ae2eec3fb118a83c05f000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2 0 17904698
```

### Your output would be similar to this

```zsh
Finished dev [unoptimized + debuginfo] target(s) in 1.30s
     Running `target/debug/evm_simulator 0x3B059f15059d976cA189165ee36d75Cb18249daf 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D 0x791ac94700000000000000000000000000000000000000000000021e19e0c9bab24000000000000000000000000000000000000000000000000000000191112d9c55be9100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000003b059f15059d976ca189165ee36d75cb18249daf0000000000000000000000000000000000000000000000000000000064d8924a0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000930dac667ca8ac9166c93ae2eec3fb118a83c05f000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2 0 17904698`


 Simulating transaction with details:
     From:  0x3b059f15059d976ca189165ee36d75cb18249daf
     To:  0x7a250d5630b4cf539739df2c5dacb4c659f2488d
     Data:  0x791ac94700000000000000000000000000000000000000000000021e19e0c9bab24000000000000000000000000000000000000000000000000000000191112d9c55be9100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000003b059f15059d976ca189165ee36d75cb18249daf0000000000000000000000000000000000000000000000000000000064d8924a0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000930dac667ca8ac9166c93ae2eec3fb118a83c05f000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2
     Value:  0
     Block Number:  17904698





 ---------------------------------------------------- SIMULATION RESULTS -----------------------------------------------------

 Detected 'watched event 0':
                        Operation: Approval,
                        Token Info:
                            Standard: NONE,
                            Address: 0x930dac667ca8ac9166c93ae2eec3fb118a83c05f,
                            Token Name: "Nuclear Pump",
                            Symbol: "NUMP",
                            Decimals: 18,
                        From: 0x930dac667ca8ac9166c93ae2eec3fb118a83c05f,
                        To: 0x7a250d5630b4cf539739df2c5dacb4c659f2488d,
                        id: "",
                        Amount: "1000.000000000000000000"

 Detected 'watched event 1':
                        Operation: Transfer,
                        Token Info:
                            Standard: NONE,
                            Address: 0x930dac667ca8ac9166c93ae2eec3fb118a83c05f,
                            Token Name: "Nuclear Pump",
                            Symbol: "NUMP",
                            Decimals: 18,
                        From: 0x930dac667ca8ac9166c93ae2eec3fb118a83c05f,
                        To: 0x405a49c1ae3836205edadb873946f5925812ec72,
                        id: "",
                        Amount: "1000.000000000000000000"

 Detected 'watched event 2':
                        Operation: Approval,
                        Token Info:
                            Standard: NONE,
                            Address: 0x930dac667ca8ac9166c93ae2eec3fb118a83c05f,
                            Token Name: "Nuclear Pump",
                            Symbol: "NUMP",
                            Decimals: 18,
                        From: 0x930dac667ca8ac9166c93ae2eec3fb118a83c05f,
                        To: 0x7a250d5630b4cf539739df2c5dacb4c659f2488d,
                        id: "",
                        Amount: "0.000000000000000000"

 Detected 'watched event 3':
                        Operation: Transfer,
                        Token Info:
                            Standard: NONE,
                            Address: 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2,
                            Token Name: "Wrapped Ether",
                            Symbol: "WETH",
                            Decimals: 18,
                        From: 0x405a49c1ae3836205edadb873946f5925812ec72,
                        To: 0x7a250d5630b4cf539739df2c5dacb4c659f2488d,
                        id: "",
                        Amount: "0.012324693073573815"

 Detected 'watched event 4':
                        Operation: Transfer,
                        Token Info:
                            Standard: NONE,
                            Address: 0x930dac667ca8ac9166c93ae2eec3fb118a83c05f,
                            Token Name: "Nuclear Pump",
                            Symbol: "NUMP",
                            Decimals: 18,
                        From: 0x3b059f15059d976ca189165ee36d75cb18249daf,
                        To: 0x405a49c1ae3836205edadb873946f5925812ec72,
                        id: "",
                        Amount: "9700.000000000000000000"

 Detected 'watched event 5':
                        Operation: Transfer,
                        Token Info:
                            Standard: NONE,
                            Address: 0x930dac667ca8ac9166c93ae2eec3fb118a83c05f,
                            Token Name: "Nuclear Pump",
                            Symbol: "NUMP",
                            Decimals: 18,
                        From: 0x3b059f15059d976ca189165ee36d75cb18249daf,
                        To: 0x930dac667ca8ac9166c93ae2eec3fb118a83c05f,
                        id: "",
                        Amount: "300.000000000000000000"

 Detected 'watched event 6':
                        Operation: Approval,
                        Token Info:
                            Standard: NONE,
                            Address: 0x930dac667ca8ac9166c93ae2eec3fb118a83c05f,
                            Token Name: "Nuclear Pump",
                            Symbol: "NUMP",
                            Decimals: 18,
                        From: 0x3b059f15059d976ca189165ee36d75cb18249daf,
                        To: 0x7a250d5630b4cf539739df2c5dacb4c659f2488d,
                        id: "",
                        Amount: "115792089237316195423570985008687907853269984665640564019757.584007913129639935"

 Detected 'watched event 7':
                        Operation: Transfer,
                        Token Info:
                            Standard: NONE,
                            Address: 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2,
                            Token Name: "Wrapped Ether",
                            Symbol: "WETH",
                            Decimals: 18,
                        From: 0x405a49c1ae3836205edadb873946f5925812ec72,
                        To: 0x7a250d5630b4cf539739df2c5dacb4c659f2488d,
                        id: "",
                        Amount: "0.115192375116522393"
```

### To use:

It takes in input as follows

```zsh
cargo run <From address> <To address> <Input data> <value> <block_number>
```

Leaving block number blank (i.e `""`) uses the latest block number, other values must have explicit values.
