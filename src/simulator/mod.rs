use dotenv::dotenv;
use ethers::{
    core::types::TransactionRequest,
    providers::{Http, Provider},
    types::Address,
    utils::Anvil,
};
use eyre::Result;
use std::convert::TryFrom;
use std::process;

mod constants;
mod fork_simulator;
pub mod print_result;
mod process_logs;
mod trace_simulator;
pub mod types;
mod utils;

use types::{SimulationParams, SimulationResults};

use self::types::BlockNumberType;

pub async fn simulate(
    simulation_params: SimulationParams,
    create_fork: bool,
) -> Result<Vec<SimulationResults>> {
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

    let tx = TransactionRequest::new()
        .from(simulation_params.from)
        .to(simulation_params.to)
        .value(simulation_params.value)
        .data(simulation_params.data);

    let provider;
    let anvil;
    let simulated_infos: Vec<SimulationResults>;
    if simulation_params.persist {
        provider = Provider::<Http>::try_from(rpc_url).unwrap_or_else(|_| {
            eprintln!("could not instantiate HTTP Provider");
            process::exit(1);
        });

        simulated_infos = use_fork_simulator(&provider, simulation_params.from, tx).await?;
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

        simulated_infos = if create_fork {
            use_fork_simulator(&provider, simulation_params.from, tx).await?
        } else {
            trace_simulator::simulate(tx, &provider, simulation_params.block_number)
                .await
                .expect("Fork simulation failed")
        };
    }

    Ok(simulated_infos)
}

async fn use_fork_simulator(
    provider: &Provider<Http>,
    from: Address,
    tx: TransactionRequest,
) -> Result<Vec<SimulationResults>> {
    // impersonate address
    provider.request("anvil_impersonateAccount", [from]).await?;

    let simulated_infos = fork_simulator::simulate(tx, &provider)
        .await
        .expect("Fork simulation failed");

    provider
        .request("anvil_stopImpersonatingAccount", [from])
        .await?;

    Ok(simulated_infos)
}
