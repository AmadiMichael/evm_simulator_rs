//// Works for the most part but still WIP

use ethers::{
    core::types::TransactionRequest,
    providers::{Http, Middleware, Provider},
    types::{
        Address, BlockId, BlockNumber, Bytes, GethDebugTracingCallOptions, GethDebugTracingOptions,
        NameOrAddress, StructLog, H256, U256, U64,
    },
};
use eyre::Result;
use std::process;

use super::process_logs::process_logs;
use super::types::{BlockNumberType, MyLog, SimulationResults};
use super::utils::{u256_to_address, u64_array_to_u8_array, write_to_output_file};

pub async fn simulate(
    tx: TransactionRequest,
    provider: &Provider<Http>,
    block: BlockNumberType,
) -> Result<Vec<SimulationResults>> {
    let block = match block {
        BlockNumberType::Past(num) => BlockId::Number(BlockNumber::Number(U64::from(num))),
        BlockNumberType::Latest => BlockId::Number(BlockNumber::Latest),
    };
    let mut tracing_options = GethDebugTracingOptions::default();
    tracing_options.enable_memory = Some(true);
    // tracing_options.tracer = Some(GethDebugTracerType::BuiltInTracer(GethDebugBuiltInTracerType::CallTracer));
    // tracing_options.tracer_config = Some(GethDebugTracerConfig::BuiltInTracer(
    //     GethDebugBuiltInTracerConfig::CallTracer(CallConfig {
    //         with_log: Some(true),
    //         only_top_call: Some(false),
    //     }),
    // ));

    let to: Address = match tx.to.clone().unwrap() {
        NameOrAddress::Address(a) => a,
        NameOrAddress::Name(_) => {
            println!("name unsupported");
            process::exit(1);
        }
    };

    let tx_trace = provider
        .debug_trace_call(
            tx,
            Some(block),
            GethDebugTracingCallOptions {
                tracing_options,
                state_overrides: None,
            },
        )
        .await
        .unwrap_or_else(|e| {
            eprintln!("transaction reverted with err: {}", e);
            process::exit(1);
        });

    // write_to_output_file(&tx_trace);

    let x = match tx_trace {
        ethers::types::GethTrace::Known(a) => match a {
            ethers::types::GethTraceFrame::Default(b) => b,
            _ => todo!(),
        },
        _ => todo!(),
    };

    let mut call_stack: Vec<Address> = vec![to];
    let mut log_call_stack: Vec<Vec<Address>> = Vec::new();

    let struct_logs: Vec<StructLog> = x
        .struct_logs
        .into_iter()
        .filter(|s| {
            // update call stack
            match s.op.as_str() {
                "CALL" | "STATICCALL" | "CREATE" => {
                    let stack = s.stack.as_ref().unwrap();
                    call_stack.push(u256_to_address(stack[stack.len() - 2]));
                    false
                }
                "RETURN" | "REVERT" | "STOP" => {
                    call_stack.pop();
                    false
                }
                "LOG3" => {
                    log_call_stack.push(call_stack.clone());
                    true
                }
                _ => false,
            }
        })
        .collect();

    let v = struct_logs.into_iter().zip(log_call_stack);

    write_to_output_file(&v);

    let mut logs: Vec<MyLog> = Vec::new();
    for (struct_log, call_stack) in v {
        let stack = struct_log.stack.unwrap();
        let stack_length = stack.len();

        let memory = struct_log.memory.unwrap();

        // get data
        let data_word_index = (stack[stack_length - 1] / 32).as_usize();
        let data_offset = (stack[stack_length - 1] % 32).as_usize();

        let data_len = (stack[stack_length - 2]).as_usize();

        let mut data: Vec<u8> = Vec::new();

        let count = (data_len / 32) + 1;
        for i in 0..count {
            let to_push;

            if i == count - 1 {
                if data_offset == 0 {
                    break;
                }
                let x = memory[data_word_index + i].as_str();
                to_push = &x[0..data_offset];
            } else {
                to_push = memory[data_word_index + i].as_str();
            }

            let y = u64_array_to_u8_array(U256::from_str_radix(to_push, 16).expect("aaa").0);

            data.append(&mut y.to_vec());
        }

        // let data = Bytes::from(data.as_bytes().to_vec());
        let data = Bytes::from(data);

        // get 3 topics
        let topics = vec![
            H256::from(u64_array_to_u8_array(stack[stack_length - 3].0)),
            H256::from(u64_array_to_u8_array(stack[stack_length - 4].0)),
            H256::from(u64_array_to_u8_array(stack[stack_length - 5].0)),
        ];

        let address: Address = call_stack[(struct_log.depth - 1) as usize];

        logs.push(MyLog {
            address,
            topics,
            data,
        });
    }

    // println!("{:?}", logs);

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
