
use dotenv::dotenv;
use ethers::{
    core::{types::TransactionRequest, utils::Anvil},
    providers::{Http, Provider},
};
use eyre::Result;
use std::convert::TryFrom;
use std::process;

pub mod types;
pub mod print_result;
mod fork_simulator;
mod trace_simulator;
mod constants;

use types::{SimulationParams, SimulationResults, BlockNumberType};

pub async fn simulate(simulation_params: SimulationParams) -> Result<Vec<SimulationResults>> {
    // either use parsed in rpc-url if it exists or use the one in the nev file if that exists, else revert
    let rpc_url = match simulation_params.rpc_url {
        Some(u) => u,
        None => {
            dotenv().ok();
            let url =
                std::env::var("RPC_URL").expect("RPC_URL must be set if rpc flag is not given");
            url
        }
    };

    let provider: Provider<Http>;
    let anvil;
    if simulation_params.persist {
        provider = Provider::<Http>::try_from(rpc_url).unwrap_or_else(|_| {
            eprintln!("could not instantiate HTTP Provider");
            process::exit(1);
        });
    } else {
        // create instance of forked chain using anvil
        anvil = match simulation_params.block_number {
            BlockNumberType::Past(num) => Anvil::new().fork(rpc_url).fork_block_number(num).spawn(),
            BlockNumberType::Latest => Anvil::new().fork(rpc_url).spawn(),
        };
        provider = Provider::<Http>::try_from(anvil.endpoint()).unwrap_or_else(|_| {
            eprintln!("could not instantiate HTTP Provider");
            process::exit(1);
        });
    }

    // // impersonate address
    // provider
    //     .request("anvil_impersonateAccount", [simulation_params.from])
    //     .await?;

    // setup tx
    let tx = TransactionRequest::new()
        .from(simulation_params.from)
        .to(simulation_params.to)
        .value(simulation_params.value)
        .data(simulation_params.data);


    // let simulated_infos = fork_simulator::simulate(tx, &provider).await.expect("Fork simulation failed");
    let simulated_infos = trace_simulator::simulate(tx, &provider).await.expect("Fork simulation failed");

    // // stop impersonate address
    // provider
    //     .request("anvil_stopImpersonatingAccount", [simulation_params.from])
    //     .await?;

    Ok(simulated_infos)
}