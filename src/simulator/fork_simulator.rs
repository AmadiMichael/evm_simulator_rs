
use ethers::{
    abi::{decode_whole, ParamType, Token},
    contract::Multicall,
    core::types::TransactionRequest,
    prelude::abigen,
    providers::{Http, Middleware, Provider},
    types::{Address, Log, U256},
};
use eyre::Result;
use std::process;
use std::sync::Arc;

use super::types::{SimulationResults, Operation, TokenInfo, Standard};
use super::constants::{CHECKED_TOPICS, APPROVAL, APPROVAL_FOR_ALL, TRANSFER, TRANSFER_SINGLE};

pub async fn simulate(tx: TransactionRequest, provider: &Provider<Http>) -> Result<Vec<SimulationResults>> {
    // send tx
    let pending_tx = provider.send_transaction(tx, None).await.unwrap_or_else(|e| {
        eprintln!("transaction reverted with err: {}", e);
        process::exit(1);
    });

    // await and get receipt and tx
    let receipt = pending_tx
        .await?
        .ok_or_else(|| eyre::format_err!("Transaction Failed"))?;

    // let tx = provider.get_transaction(receipt.transaction_hash).await?;

    // println!("tx: {:?}", serde_json::to_string(&tx)?);
    // println!("receipt: {:?}", serde_json::to_string(&receipt)?);

    let logs = receipt.logs;
    // println!("logs: {:?}", logs);

    let mut simulated_infos: Vec<SimulationResults> = Vec::new();

    for log in logs.into_iter() {
        match process_logs(log, provider.clone()).await {
            Ok(Some(x)) => simulated_infos.push(x),
            Ok(None) => {}
            Err(err) => {
                eprintln!("Err {}", err);
                process::exit(1)
            }
        }
    }

    Ok(simulated_infos)
}


async fn process_logs(log: Log, provider: Provider<Http>) -> Result<Option<SimulationResults>> {
    let topic0 = log.topics[0]
        .as_bytes()
        .try_into()
        .expect("could not convert topic0 into a uint8 array");

    if CHECKED_TOPICS.contains(&topic0) {
        let amount: U256;
        let id: Option<U256>;

        if log.data.len() > 32 {
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
        } else {
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
        }

        let (name, symbol, decimals) = get_token_name_and_symbol(log.address, provider).await?;

        match_simulation_result(topic0, name, symbol, decimals, amount, id, log)
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
    log: Log,
) -> Result<Option<SimulationResults>> {
    match topic0 {
        APPROVAL => Ok(Some(SimulationResults {
            operation: Operation::Approval,
            token_info: TokenInfo {
                standard: Standard::NONE,
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
                standard: Standard::NONE,
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
                standard: Standard::NONE,
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
                standard: Standard::Eip1155,
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
        _ => Ok(Some(SimulationResults {
            operation: Operation::TransferBatch,
            token_info: TokenInfo {
                standard: Standard::Eip1155,
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
    }
}

async fn get_token_name_and_symbol(
    address: Address,
    provider: Provider<Http>,
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

    let name = token_instance.method::<_, String>("name", ())?;
    let symbol = token_instance.method::<_, String>("symbol", ())?;
    let decimals = token_instance.method::<_, U256>("decimals", ())?;

    let mut multicall = Multicall::new(client, None).await?;
    multicall
        .add_call(name, true)
        .add_call(symbol, true)
        .add_call(decimals, true);

    // `await`ing on the `call` method lets us fetch the return values of both the above calls in one single RPC call
    let (name, symbol, decimals): (String, String, U256) = match multicall.call().await {
        Ok((a, b, c)) => (a, b, c),
        Err(_) => ("".to_owned(), "".to_owned(), U256::from_dec_str("0")?),
    };

    Ok((name, symbol, decimals))
}