use dotenv::dotenv;
use ethers::{
    core::{types::TransactionRequest},
    providers::{Http, Provider},
};
use eyre::Result;
use std::convert::TryFrom;
use std::process;

mod constants;
mod fork_simulator;
pub mod print_result;
mod trace_simulator;
pub mod types;

use types::{SimulationParams, SimulationResults};

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

    let provider = Provider::<Http>::try_from(rpc_url).unwrap_or_else(|_| {
        eprintln!("could not instantiate HTTP Provider");
        process::exit(1);
    });

    let tx = TransactionRequest::new()
        .from(simulation_params.from)
        .to(simulation_params.to)
        .value(simulation_params.value)
        .data(simulation_params.data);

    let simulated_infos;
    if simulation_params.persist {
        // impersonate address
        provider
            .request("anvil_impersonateAccount", [simulation_params.from])
            .await?;

        simulated_infos = fork_simulator::simulate(tx, &provider)
            .await
            .expect("Fork simulation failed");

        provider
            .request("anvil_stopImpersonatingAccount", [simulation_params.from])
            .await?;
    } else {
        simulated_infos = trace_simulator::simulate(tx, &provider, simulation_params.block_number)
            .await
            .expect("Fork simulation failed");
    }

    Ok(simulated_infos)
}
