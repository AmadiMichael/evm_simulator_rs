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

```
cargo run 0x448E0F9F42746F6165Dbe6E7B77149bB0F631E6E 0x2Ec705D306b51e486B1bC0D6ebEE708E0661ADd1 0x18cbafe500000000000000000000000000000000000000000000000000394425252270000000000000000000000000000000000000000000000000000035e2b98723e13d00000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000448e0f9f42746f6165dbe6e7b77149bb0f631e6e0000000000000000000000000000000000000000000000000000000064a876b70000000000000000000000000000000000000000000000000000000000000002000000000000000000000000e30bbec87855c8710729e6b8384ef9783c76379c000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2 0 17644319
```

### Your output would be similar to this

```
Simulating transaction with details:
     From:  0x448e0f9f42746f6165dbe6e7b77149bb0f631e6e
     To:  0x2ec705d306b51e486b1bc0d6ebee708e0661add1
     Data:  0x18cbafe500000000000000000000000000000000000000000000000000394425252270000000000000000000000000000000000000000000000000000035e2b98723e13d00000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000448e0f9f42746f6165dbe6e7b77149bb0f631e6e0000000000000000000000000000000000000000000000000000000064a876b70000000000000000000000000000000000000000000000000000000000000002000000000000000000000000e30bbec87855c8710729e6b8384ef9783c76379c000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2
     Value:  0
     Block Number:  17644319





 ---------------------------------------------------- SIMULATION RESULTS -----------------------------------------------------

 Detected 'watched event 0':
                        Operation: Transfer,
                        Token Info:
                            Standard: NONE,
                            Address: 0xe30bbec87855c8710729e6b8384ef9783c76379c,
                            Token Name: "Wrapped Luna",
                            Symbol: "WLUNA",
                            Decimals: 9,
                        From: 0x448e0f9f42746f6165dbe6e7b77149bb0f631e6e,
                        To: 0x7a333329ba40a0999ba1c8b4d56acc1107c7a501,
                        id: "",
                        Amount: "16119000.000000000"

 Detected 'watched event 1':
                        Operation: Approval,
                        Token Info:
                            Standard: NONE,
                            Address: 0xe30bbec87855c8710729e6b8384ef9783c76379c,
                            Token Name: "Wrapped Luna",
                            Symbol: "WLUNA",
                            Decimals: 9,
                        From: 0x448e0f9f42746f6165dbe6e7b77149bb0f631e6e,
                        To: 0x2ec705d306b51e486b1bc0d6ebee708e0661add1,
                        id: "",
                        Amount: "0.000000000"

 Detected 'watched event 2':
                        Operation: Transfer,
                        Token Info:
                            Standard: NONE,
                            Address: 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2,
                            Token Name: "Wrapped Ether",
                            Symbol: "WETH",
                            Decimals: 18,
                        From: 0x7a333329ba40a0999ba1c8b4d56acc1107c7a501,
                        To: 0x2ec705d306b51e486b1bc0d6ebee708e0661add1,
                        id: "",
                        Amount: "0.020210640756165174"
```

### To use:

It takes in input as follows

```
cargo run <From address> <To address> <Input data> <value> <block_number>
```

Leaving block number blank (i.e `""`) uses the latest block number, other values must have explicit values.
