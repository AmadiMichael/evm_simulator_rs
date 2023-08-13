use dotenv::dotenv;
use ethers::{
    abi::{decode_whole, ParamType, Token},
    contract::Multicall,
    core::{types::TransactionRequest, utils::Anvil},
    prelude::abigen,
    providers::{Http, Middleware, Provider},
    types::{Address, Bytes, Log, /* H256, */ U256},
    utils::{format_units, parse_ether},
};
use eyre::Result;
use std::convert::TryFrom;
use std::process;
use std::sync::Arc;

// Constants
const APPROVAL: [u8; 32] = [
    140, 91, 225, 229, 235, 236, 125, 91, 209, 79, 113, 66, 125, 30, 132, 243, 221, 3, 20, 192,
    247, 178, 41, 30, 91, 32, 10, 200, 199, 195, 185, 37,
]; // 0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925
const TRANSFER: [u8; 32] = [
    221, 242, 82, 173, 27, 226, 200, 155, 105, 194, 176, 104, 252, 55, 141, 170, 149, 43, 167, 241,
    99, 196, 161, 22, 40, 245, 90, 77, 245, 35, 179, 239,
]; // 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
const APPROVAL_FOR_ALL: [u8; 32] = [
    23, 48, 126, 171, 57, 171, 97, 7, 232, 137, 152, 69, 173, 61, 89, 189, 150, 83, 242, 0, 242,
    32, 146, 4, 137, 202, 43, 89, 55, 105, 108, 49,
]; // 0x17307eab39ab6107e8899845ad3d59bd9653f200f220920489ca2b5937696c31
const TRANSFER_SINGLE: [u8; 32] = [
    195, 213, 129, 104, 197, 174, 115, 151, 115, 29, 6, 61, 91, 191, 61, 101, 120, 84, 66, 115, 67,
    244, 192, 131, 36, 15, 122, 172, 170, 45, 15, 98,
]; // 0xc3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62
const TRANSFER_BATCH: [u8; 32] = [
    74, 57, 220, 6, 212, 192, 219, 198, 75, 112, 175, 144, 253, 105, 138, 35, 58, 81, 138, 165,
    208, 126, 89, 93, 152, 59, 140, 5, 38, 200, 247, 251,
]; // 0x4a39dc06d4c0dbc64b70af90fd698a233a518aa5d07e595d983b8c0526c8f7fb

const CHECKED_TOPICS: [[u8; 32]; 5] = [
    APPROVAL,
    TRANSFER,
    APPROVAL_FOR_ALL,
    TRANSFER_SINGLE,
    TRANSFER_BATCH,
];

// Types
#[derive(Debug, PartialEq)]
pub enum Operation {
    Approval,
    Transfer,
    ApprovalForAll,
    TransferSingle,
    TransferBatch,
}

#[derive(Debug, PartialEq)]
pub enum Standard {
    // This is mostly to specify eip1155 contracts but can be expanded to specify eip20 and 721 contracts later
    NONE,
    Eip1155,
}

#[derive(Debug, PartialEq)]
pub struct TokenInfo {
    pub standard: Standard,
    pub address: Address,
    pub name: String,
    pub symbol: String,
    pub decimals: U256,
}

#[derive(Debug, PartialEq)]
pub struct SimulatedInfo {
    pub operation: Operation,
    pub token_info: TokenInfo,
    pub from: Address,
    pub to: Address,
    pub id: Option<U256>,
    pub amount: U256,
}

#[derive(Debug)]
pub struct SimulationParams {
    pub from: Address,
    pub to: Address,
    pub data: Bytes,
    pub value: U256,
    pub block_number: Option<u64>,
}

impl SimulationParams {
    pub fn new(args: &Vec<String>) -> Result<Self, &str> {
        if args.len() < 6 {
            return Err("Not enough arguments");
        }

        let from = args[1].parse::<Address>();
        let from = match from {
            Ok(f) => f,
            _ => return Err("invalid 'from' address provided"),
        };

        let to = args[2].parse::<Address>();
        let to = match to {
            Ok(t) => t,
            _ => return Err("Invalid 'to' address provided"),
        };

        let data;
        if args[3] == "" {
            data = "0x".parse::<Bytes>();
        } else {
            data = args[3].parse::<Bytes>();
        }
        let data = match data {
            Ok(d) => d,
            _ => return Err("Invalid 'input data' provided"),
        };

        let value = parse_ether(args[4].as_str());
        let value = match value {
            Ok(val) => val,
            _ => return Err("Invalid ether value provided"),
        };

        let block_number = if args[5] == "" {
            None
        } else {
            let block_number = args[5].parse::<u64>();
            let block_number = match block_number {
                Ok(num) => Some(num),
                _ => return Err("Block number parsed in not a valid number. To use the current block number, parse in an empty string e.g ''"),
            };
            block_number
        };

        Ok(SimulationParams {
            from,
            to,
            data,
            value,
            block_number,
        })
    }
}

pub async fn simulate(simulation_params: SimulationParams) -> Result<Vec<SimulatedInfo>> {
    dotenv().ok();
    let alchemy_api_key = std::env::var("ALCHEMY_API_KEY").expect("ALCHEMY_API_KEY must be set.");
    let mut url = String::from("https://eth-mainnet.g.alchemy.com/v2/"); // "http://127.0.0.1:8545";
    let rpc_url: &str = {
        url.push_str(&alchemy_api_key);
        url.as_str()
    };

    let anvil = match simulation_params.block_number {
        Some(num) => Anvil::new().fork(rpc_url).fork_block_number(num).spawn(),
        None => Anvil::new().fork(rpc_url).spawn(),
    };
    let provider =
        Provider::<Http>::try_from(anvil.endpoint()).expect("could not instantiate HTTP Provider");

    // impersonate address
    provider
        .request("anvil_impersonateAccount", [simulation_params.from])
        .await?;

    // setup tx
    let tx = TransactionRequest::new()
        .from(simulation_params.from)
        .to(simulation_params.to)
        .value(simulation_params.value)
        .data(simulation_params.data);

    // send tx
    let pending_tx = provider.send_transaction(tx, None).await?;

    // await and get receipt and tx
    let receipt = pending_tx
        .await?
        .ok_or_else(|| eyre::format_err!("Transaction Failed"))?;

    // let tx = provider.get_transaction(receipt.transaction_hash).await?;

    // println!("tx: {:?}", serde_json::to_string(&tx)?);
    // println!("receipt: {:?}", serde_json::to_string(&receipt)?);

    let logs = receipt.logs;
    // println!("logs: {:?}", logs);

    let mut simulated_infos: Vec<SimulatedInfo> = Vec::new();

    for log in logs.iter() {
        match checks(log, provider.clone()).await {
            Ok(Some(x)) => simulated_infos.push(x),
            Ok(None) => {}
            Err(err) => {
                eprintln!("Err {}", err);
                process::exit(1)
            }
        }
    }

    // stop impersonate address
    provider
        .request("anvil_stopImpersonatingAccount", [simulation_params.from])
        .await?;

    Ok(simulated_infos)
}

pub fn print_result(simulated_infos: Vec<SimulatedInfo>) -> Result<()> {
    println!("\n\n\n\n\x1b[92m ---------------------------------------------------- SIMULATION RESULTS -----------------------------------------------------");
    for (index, simulated_info) in simulated_infos.iter().enumerate() {
        let decimals: u32 = simulated_info.token_info.decimals.to_string().parse()?;
        let amount = match decimals > 0 {
            true => format_units(simulated_info.amount, decimals).unwrap(),
            false => format!("{}", simulated_info.amount),
        };
        let id = match simulated_info.id {
            Some(id) => format!("{}", id),
            None => "".to_owned(),
        };

        println!(
            "\n\x1b[94m\x1b[1m Detected 'watched event {index}'\x1b[0m: 
                        Operation: {:?},
                        Token Info:
                            Standard: {:?},
                            Address: {:?},  
                            Token Name: {:?}, 
                            Symbol: {:?}, 
                            Decimals: {:?},
                        From: {:?},
                        To: {:?},
                        id: {:?},
                        Amount: {:?}",
            simulated_info.operation,
            simulated_info.token_info.standard,
            simulated_info.token_info.address,
            simulated_info.token_info.name,
            simulated_info.token_info.symbol,
            simulated_info.token_info.decimals,
            simulated_info.from,
            simulated_info.to,
            id,
            amount
        );
    }
    Ok(())
}

async fn checks(log: &Log, provider: Provider<Http>) -> Result<Option<SimulatedInfo>> {
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

        match_sim_res(topic0, name, symbol, decimals, amount, id, log)
    } else {
        Ok(None)
    }
}

fn match_sim_res(
    topic0: [u8; 32],
    name: String,
    symbol: String,
    decimals: U256,
    amount: U256,
    id: Option<U256>,
    log: &Log,
) -> Result<Option<SimulatedInfo>> {
    match topic0 {
        APPROVAL => Ok(Some(SimulatedInfo {
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
        TRANSFER => Ok(Some(SimulatedInfo {
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
        APPROVAL_FOR_ALL => Ok(Some(SimulatedInfo {
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
        TRANSFER_SINGLE => Ok(Some(SimulatedInfo {
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
        _ => Ok(Some(SimulatedInfo {
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
        IERC20,
        r#"[
            function name() external view returns (string)
            function symbol() external view returns (string)
            function decimals() external view returns (uint256)
        ]"#,
    );

    let client = Arc::new(provider);
    let contract = IERC20::new(address, client.clone());

    let name = contract.method::<_, String>("name", ())?;
    let symbol = contract.method::<_, String>("symbol", ())?;
    let decimals = contract.method::<_, U256>("decimals", ())?;

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
