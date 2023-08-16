use eyre::Result;
use std::process;

use evm_simulator::{cli, print_result, simulate, SimulationParams};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = cli::cli();
    let simulation_params = SimulationParams::new(&args).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    println!(
        "\n\n\x1b[1m Simulating transaction with details:
    \x1b[92m From: \x1b[0m {:?}
    \x1b[92m To: \x1b[0m {:?}
    \x1b[92m Data: \x1b[0m {}
    \x1b[92m Value: \x1b[0m {}
    \x1b[92m Block Number: \x1b[0m {:?}\n",
        simulation_params.from,
        simulation_params.to,
        simulation_params.data,
        simulation_params.value,
        simulation_params.block_number
    );

    let sim_result = simulate(simulation_params).await?;
    let _ = print_result(sim_result);

    Ok(())
}

// still working on tests
#[cfg(test)]
mod test {
    use ethers::types::{Address, U256};
    use evm_simulator::{
        simulate, Operation, SimulationParams, SimulationResults, Standard, TokenInfo,
    };
    use eyre::Result;

    // test runs
    fn return_erc20_test_case() -> Vec<String> {
        // return a uniswap swap tx data

        let from = "0x448E0F9F42746F6165Dbe6E7B77149bB0F631E6E".to_owned();
        let to = "0x2Ec705D306b51e486B1bC0D6ebEE708E0661ADd1".to_owned();
        let data = "0x18cbafe500000000000000000000000000000000000000000000000000394425252270000000000000000000000000000000000000000000000000000035e2b98723e13d00000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000448e0f9f42746f6165dbe6e7b77149bb0f631e6e0000000000000000000000000000000000000000000000000000000064a876b70000000000000000000000000000000000000000000000000000000000000002000000000000000000000000e30bbec87855c8710729e6b8384ef9783c76379c000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".to_owned();
        let value = "0".to_owned();
        let block_number = "17644319".to_owned();
        let rpc = "".to_owned();

        vec![from, to, data, value, block_number, rpc]
    }

    fn return_nft_test_case() -> Vec<String> {
        // return an erc1155 and erc20 tx

        let from = "0x77c5D44F392DD825A073C417EDe8C2f8bce603F6".to_owned();
        let to = "0x00000000000000ADc04C56Bf30aC9d3c0aAF14dC".to_owned();
        let data = "0xe7acab24000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000005e00000007b02230091a7ed01230072f7006a004d60a8d4e71d599b8104250f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000046000000000000000000000000000000000000000000000000000000000000004c00000000000000000000000000b818dc9d41732617dfc5bc8dff03dac632780e1000000000000000000000000000000e7ec00e7b300774b00001314b8610022b80000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000000000022000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000064ac23690000000000000000000000000000000000000000000000000000000064d501e50000000000000000000000000000000000000000000000000000000000000000360c6ebe0000000000000000000000000000000000000000710e918d59930ae50000007b02230091a7ed01230072f7006a004d60a8d4e71d599b8104250f0000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000d529ae9e86000000000000000000000000000000000000000000000000000000d529ae9e8600000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000300000000000000000000000076be3b62873462d2142405439777e971754e8e77000000000000000000000000000000000000000000000000000000000000282c000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000b818dc9d41732617dfc5bc8dff03dac632780e10000000000000000000000000000000000000000000000000000000000000001000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005543df729c0000000000000000000000000000000000000000000000000000005543df729c0000000000000000000000000000000a26b00c1f0df003000390027140000faa719000000000000000000000000000000000000000000000000000000000000004059577c8e8707f9b8896a85d4a59a2ef30647fb061287f000079b9fe1e5063474597f9bf2b77700bba355bd813f416da1c12048c8b976a222a3fcdbc92a7887aa000000000000000000000000000000000000000000000000000000000000007e0077c5d44f392dd825a073c417ede8c2f8bce603f60000000064add71eaab1b624b2bf2ba4bc33225f4eb7638e22f73aca43287493a1f63311f6c038a5d8ca9631edb8f32f3696d78963d536359f05834d595295a3189b2c0862236f6900000000000000000000000000000000000000000000000000000000000000282c0000000000000000000000000000000000000000000000000000000000000000000000000000360c6ebe".to_owned();
        let value = "0".to_owned();
        let block_number = "17673303".to_owned();
        let rpc = "".to_owned();

        vec![from, to, data, value, block_number, rpc]
    }

    #[tokio::test]
    async fn test_swap_tx_sim_should_detect_expected_logs() -> Result<(), String> {
        let args = return_erc20_test_case();
        let simulation_params = SimulationParams::new(&args)?;

        let sim_result = simulate(simulation_params).await;
        let sim_result = match sim_result {
            Ok(r) => r,
            Err(_) => return Err("Simulation failed".to_owned()),
        };
        let expected_result = vec![
            SimulationResults {
                operation: Operation::Transfer,
                token_info: TokenInfo {
                    standard: Standard::NONE,
                    address: "0xe30bbec87855c8710729e6b8384ef9783c76379c"
                        .parse::<Address>()
                        .unwrap(),
                    name: "Wrapped Luna".to_owned(),
                    symbol: "WLUNA".to_owned(),
                    decimals: U256::from(9),
                },
                from: "0x448e0f9f42746f6165dbe6e7b77149bb0f631e6e"
                    .parse::<Address>()
                    .unwrap(),
                to: "0x7a333329ba40a0999ba1c8b4d56acc1107c7a501"
                    .parse::<Address>()
                    .unwrap(),
                id: None,
                amount: U256::from_dec_str("16119000000000000").unwrap(),
            },
            SimulationResults {
                operation: Operation::Approval,
                token_info: TokenInfo {
                    standard: Standard::NONE,
                    address: "0xe30bbec87855c8710729e6b8384ef9783c76379c"
                        .parse::<Address>()
                        .unwrap(),
                    name: "Wrapped Luna".to_owned(),
                    symbol: "WLUNA".to_owned(),
                    decimals: U256::from(9),
                },
                from: "0x448e0f9f42746f6165dbe6e7b77149bb0f631e6e"
                    .parse::<Address>()
                    .unwrap(),
                to: "0x2ec705d306b51e486b1bc0d6ebee708e0661add1"
                    .parse::<Address>()
                    .unwrap(),
                id: None,
                amount: U256::from(0),
            },
            SimulationResults {
                operation: Operation::Transfer,
                token_info: TokenInfo {
                    standard: Standard::NONE,
                    address: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
                        .parse::<Address>()
                        .unwrap(),
                    name: "Wrapped Ether".to_owned(),
                    symbol: "WETH".to_owned(),
                    decimals: U256::from(18),
                },
                from: "0x7a333329ba40a0999ba1c8b4d56acc1107c7a501"
                    .parse::<Address>()
                    .unwrap(),
                to: "0x2ec705d306b51e486b1bc0d6ebee708e0661add1"
                    .parse::<Address>()
                    .unwrap(),
                id: None,
                amount: U256::from_dec_str("20210640756165174").unwrap(),
            },
        ];

        assert_eq!(sim_result, expected_result);

        Ok(())
    }

    #[tokio::test]
    async fn test_nft_tx_sim_should_detect_expected_logs() -> Result<(), String> {
        let args = return_nft_test_case();
        let simulation_params = SimulationParams::new(&args)?;

        let sim_result = simulate(simulation_params).await;
        let sim_result = match sim_result {
            Ok(r) => r,
            Err(_) => return Err("Simulation failed".to_owned()),
        };
        let expected_result = vec![
            SimulationResults {
                operation: Operation::Transfer,
                token_info: TokenInfo {
                    standard: Standard::NONE,
                    address: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
                        .parse::<Address>()
                        .unwrap(),
                    name: "Wrapped Ether".to_owned(),
                    symbol: "WETH".to_owned(),
                    decimals: U256::from_dec_str("18").unwrap(),
                },
                from: "0x0b818dc9d41732617dfc5bc8dff03dac632780e1"
                    .parse::<Address>()
                    .unwrap(),
                to: "0x77c5d44f392dd825a073c417ede8c2f8bce603f6"
                    .parse::<Address>()
                    .unwrap(),
                id: None,
                amount: U256::from_dec_str("60000000000000000").unwrap(),
            },
            SimulationResults {
                operation: Operation::TransferSingle,
                token_info: TokenInfo {
                    standard: Standard::Eip1155,
                    address: "0x76be3b62873462d2142405439777e971754e8e77"
                        .parse::<Address>()
                        .unwrap(),
                    name: "".to_owned(),
                    symbol: "".to_owned(),
                    decimals: U256::from_dec_str("0").unwrap(),
                },
                from: "0x1e0049783f008a0085193e00003d00cd54003c71"
                    .parse::<Address>()
                    .unwrap(),
                to: "0x77c5d44f392dd825a073c417ede8c2f8bce603f6"
                    .parse::<Address>()
                    .unwrap(),
                id: Some(U256::from_dec_str("10284").unwrap()),
                amount: U256::from_dec_str("2").unwrap(),
            },
            SimulationResults {
                operation: Operation::Transfer,
                token_info: TokenInfo {
                    standard: Standard::NONE,
                    address: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
                        .parse::<Address>()
                        .unwrap(),
                    name: "Wrapped Ether".to_owned(),
                    symbol: "WETH".to_owned(),
                    decimals: U256::from_dec_str("18").unwrap(),
                },
                from: "0x77c5d44f392dd825a073c417ede8c2f8bce603f6"
                    .parse::<Address>()
                    .unwrap(),
                to: "0x0000a26b00c1f0df003000390027140000faa719"
                    .parse::<Address>()
                    .unwrap(),
                id: None,
                amount: U256::from_dec_str("1500000000000000").unwrap(),
            },
        ];

        assert_eq!(sim_result, expected_result);

        Ok(())
    }
}
