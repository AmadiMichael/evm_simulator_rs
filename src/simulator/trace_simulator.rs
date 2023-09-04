
use dotenv::dotenv;
use ethers::{
    abi::{decode_whole, ParamType, Token},
    contract::Multicall,
    core::{types::TransactionRequest},
    prelude::abigen,
    providers::{Http, Middleware, Provider},
    types::{Address, Bytes, GethDebugTracingCallOptions, Log, H256, U256, GethDebugTracingOptions, DefaultFrame, StructLog},
    utils::{format_units, parse_ether},
};
use eyre::Result;
use std::convert::TryFrom;
use std::process;
use std::sync::Arc;

use super::types::{SimulationParams, SimulationResults, BlockNumberType, Operation, TokenInfo, Standard};
use super::constants::{CHECKED_TOPICS, APPROVAL, APPROVAL_FOR_ALL, TRANSFER, TRANSFER_SINGLE};


#[derive(Debug)]
struct MyLog {
    address: Address,
    topics: Vec<H256>,
    data: Bytes
}


fn u64_array_to_u8_array(input: [u64; 4]) -> [u8; 32] {
    let mut output = [0; 32];

    for (i, &u64_value) in input.iter().enumerate() {
        let bytes = u64_value.swap_bytes().to_le_bytes();

        let u = 3 - i;

        output[u * 8..(u + 1) * 8].copy_from_slice(&bytes);
    }

    output
}

pub async fn simulate(tx: TransactionRequest, provider: &Provider<Http>) -> Result<Vec<SimulationResults>> {
    // let block = match simulation_params.block_number {
    //     BlockNumberType::Past(n) => n,
    //     BlockNumberType::Latest => provider.get_block_number().await.unwrap().as_u64()
    // };

    let mut tracing_options = GethDebugTracingOptions::default();
    tracing_options.enable_memory = Some(true);

    let tx_trace = provider
        .debug_trace_call(
            tx,
            None, //Some(block.into()),
            GethDebugTracingCallOptions { tracing_options, state_overrides: None },
        )
        .await
        .unwrap_or_else(|e| {
            eprintln!("transaction reverted with err: {}", e);
            process::exit(1);
        });


    let x = match tx_trace {
        ethers::types::GethTrace::Known(a) => {
            match a {
                ethers::types::GethTraceFrame::Default(b) => b,
                _ => todo!()
            }
        },
        _ => todo!()
    };


    let log_opcodes: Vec<StructLog> = x.struct_logs.into_iter().filter(|s| s.op == "LOG3").collect();

    // Specify the file path you want to write to
    let file_path: &str = "output.txt";

    // Open the file for writing (creates the file if it doesn't exist)
    let mut file = std::fs::File::create(file_path)?;

    let st = format!("{:?}", &log_opcodes);

    // Write the data to the file
    std::io::Write::write_all(&mut file, st.as_bytes())?;


    let mut logs: Vec<MyLog> = Vec::new();
    for struct_log in log_opcodes.into_iter() {
        let stack = struct_log.stack.unwrap();
        let stack_length = stack.len();

        let memory = struct_log.memory.unwrap();

        // get data
        let data_word_index = ((stack[stack_length - 1] / 32)).as_usize();
        let data_offset = (stack[stack_length - 1] % 32).as_usize();

        let data_len = (stack[stack_length - 2]).as_usize();

        let mut data:Vec<u8> = Vec::new();
        
        let count = (data_len / 32) + 1;
        for i in 0..count {
            let to_push;

            if i == count - 1 {
                if data_offset == 0 { break; }
                let x = memory[data_word_index + i].as_str();
                to_push = &x[0..data_offset];
            } else {
                to_push = memory[data_word_index + i].as_str();
            }

            let y = u64_array_to_u8_array(U256::from_str_radix(to_push, 16).expect("aaa").0);

            data.append(&mut y.to_vec());
        };

        // let data = Bytes::from(data.as_bytes().to_vec());
        let data = Bytes::from(data);

        // get 3 topics
        let topics = vec![H256::from(u64_array_to_u8_array(stack[stack_length - 3].0)), H256::from(u64_array_to_u8_array(stack[stack_length - 4].0)), H256::from(u64_array_to_u8_array(stack[stack_length - 5].0))];
        
        let address = Address::zero();

        println!("{:?}", &data);

        logs.push(MyLog{address, topics, data});
    }

    println!("{:?}", logs);

    let mut simulated_infos: Vec<SimulationResults> = Vec::new();

    for log in logs.into_iter() {
        match checks(log, provider.clone()).await {
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

async fn checks(log: MyLog, provider: Provider<Http>) -> Result<Option<SimulationResults>> {
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
    log: MyLog,
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
