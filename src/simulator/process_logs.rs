use ethers::{
    abi::{decode_whole, ParamType, Token},
    contract::Multicall,
    prelude::abigen,
    providers::{Http, Provider},
    types::{Address, U256},
};
use eyre::Result;
use std::process;
use std::sync::Arc;

use super::constants::{APPROVAL, APPROVAL_FOR_ALL, CHECKED_TOPICS, TRANSFER, TRANSFER_SINGLE};
use super::types::{MyLog, Operation, SimulationResults, Standard, TokenInfo};

pub async fn process_logs(
    log: MyLog,
    provider: Provider<Http>,
) -> Result<Option<SimulationResults>> {
    let topic0 = log.topics[0]
        .as_bytes()
        .try_into()
        .expect("could not convert topic0 into a uint8 array");

    if CHECKED_TOPICS.contains(&topic0) {
        let amount: U256;
        let id: Option<U256>;
        let standard: Standard;

        if log.data.len() == 64 {
            let decoded =
                match decode_whole(&[ParamType::Uint(256), ParamType::Uint(256)], &log.data) {
                    Ok(x) => x,
                    Err(err) => {
                        eprintln!("decoding failed with err: {}", err);
                        process::exit(1);
                    }
                };
            (id, amount) = match (&decoded[0], &decoded[1]) {
                (Token::Uint(x), Token::Uint(y)) => (Some(*x), *y),
                _ => {
                    eprintln!("Wrong type decoded");
                    process::exit(1);
                }
            };
            standard = Standard::Eip1155;
        } else if log.data.len() == 32 {
            let decoded = match decode_whole(&[ParamType::Uint(256)], &log.data) {
                Ok(x) => x,
                Err(err) => {
                    eprintln!("decoding failed with err: {}", err);
                    process::exit(1);
                }
            };
            amount = match decoded[0] {
                Token::Uint(x) => x,
                _ => {
                    eprintln!("Wrong type decoded");
                    process::exit(1);
                }
            };
            id = None;

            standard = match &topic0 {
                &APPROVAL => Standard::Eip20,
                &TRANSFER => Standard::Eip20,
                &APPROVAL_FOR_ALL => Standard::Eip721,
                _ => Standard::None,
            }
        } else {
            amount = U256::from(1);

            let d: &[u8] = log.topics[3]
                .as_bytes()
                .try_into()
                .expect("could not convert topic4 into a uint8 array");
            id = Some(U256::from(d));

            standard = Standard::Eip721;
        };

        let (name, symbol, decimals) =
            get_token_name_and_symbol(log.address, provider, &standard).await?;

        match_simulation_result(topic0, name, symbol, decimals, amount, id, log, standard)
    } else {
        Ok(None)
    }
}

fn match_simulation_result(
    topic0: [u8; 32],
    name: String,
    symbol: String,
    decimals: U256,
    amount: U256,
    id: Option<U256>,
    log: MyLog,
    standard: Standard,
) -> Result<Option<SimulationResults>> {
    match topic0 {
        APPROVAL => Ok(Some(SimulationResults {
            operation: Operation::Approval,
            token_info: TokenInfo {
                standard,
                name,
                symbol,
                decimals,
                address: log.address,
            },
            from: Address::from(log.topics[1]),
            to: Address::from(log.topics[2]),
            amount,
            id,
        })),
        TRANSFER => Ok(Some(SimulationResults {
            operation: Operation::Transfer,
            token_info: TokenInfo {
                standard,
                name,
                symbol,
                decimals,
                address: log.address,
            },
            from: Address::from(log.topics[1]),
            to: Address::from(log.topics[2]),
            amount,
            id,
        })),
        APPROVAL_FOR_ALL => Ok(Some(SimulationResults {
            operation: Operation::ApprovalForAll,
            token_info: TokenInfo {
                standard,
                name,
                symbol,
                decimals,
                address: log.address,
            },
            from: Address::from(log.topics[1]),
            to: Address::from(log.topics[2]),
            amount,
            id,
        })),
        TRANSFER_SINGLE => Ok(Some(SimulationResults {
            operation: Operation::TransferSingle,
            token_info: TokenInfo {
                standard,
                name,
                symbol,
                decimals,
                address: log.address,
            },
            from: Address::from(log.topics[2]),
            to: Address::from(log.topics[3]),
            amount,
            id,
        })),
        _ => Ok(Some(SimulationResults {
            operation: Operation::TransferBatch,
            token_info: TokenInfo {
                standard,
                name,
                symbol,
                decimals,
                address: log.address,
            },
            from: Address::from(log.topics[2]),
            to: Address::from(log.topics[3]),
            amount,
            id,
        })),
    }
}

async fn get_token_name_and_symbol(
    address: Address,
    provider: Provider<Http>,
    standard: &Standard,
) -> Result<(String, String, U256)> {
    abigen!(
        TokenInstance,
        r#"[
            function name() external view returns (string)
            function symbol() external view returns (string)
            function decimals() external view returns (uint256)
        ]"#,
    );

    let client = Arc::new(provider);
    let token_instance = TokenInstance::new(address, client.clone());

    let name_fn = token_instance.method::<_, String>("name", ())?;
    let symbol_fn = token_instance.method::<_, String>("symbol", ())?;
    let decimals_fn = token_instance.method::<_, U256>("decimals", ())?;

    let mut multicall = Multicall::new(client, None).await?;

    let name: String;
    let symbol: String;
    let decimals: U256;
    match standard {
        Standard::Eip20 => {
            multicall
                .add_call(name_fn, true)
                .add_call(symbol_fn, true)
                .add_call(decimals_fn, true);
            // `await`ing on the `call` method lets us fetch the return values of both the above calls in one single RPC call
            (name, symbol, decimals) = match multicall.call().await {
                Ok((a, b, c)) => (a, b, c),
                Err(_) => ("".to_owned(), "".to_owned(), U256::from_dec_str("0")?),
            };
        }
        Standard::Eip721 => {
            multicall.add_call(name_fn, true).add_call(symbol_fn, true);
            // `await`ing on the `call` method lets us fetch the return values of both the above calls in one single RPC call
            (name, symbol) = match multicall.call().await {
                Ok((a, b)) => (a, b),
                Err(_) => ("".to_owned(), "".to_owned()),
            };
            decimals = U256::from_dec_str("0")?;
        }
        Standard::Eip1155 => {
            multicall.add_call(name_fn, true);
            // `await`ing on the `call` method lets us fetch the return values of both the above calls in one single RPC call
            (name) = match multicall.call().await {
                Ok(a) => a,
                Err(_) => "".to_owned(),
            };
            symbol = "".to_owned();
            decimals = U256::from_dec_str("0")?;
        }
        Standard::None => {
            multicall
                .add_call(name_fn, true)
                .add_call(symbol_fn, true)
                .add_call(decimals_fn, true);
            // `await`ing on the `call` method lets us fetch the return values of both the above calls in one single RPC call
            (name, symbol, decimals) = match multicall.call().await {
                Ok((a, b, c)) => (a, b, c),
                Err(_) => ("".to_owned(), "".to_owned(), U256::from_dec_str("0")?),
            }
        }
    }
    Ok((name, symbol, decimals))
}
