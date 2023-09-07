use ethers::{
    core::types::TransactionRequest,
    providers::{Http, Middleware, Provider},
};
use eyre::Result;
use std::process;

use super::process_logs::process_logs;
use super::types::{MyLog, SimulationResults};

pub async fn simulate(
    tx: TransactionRequest,
    provider: &Provider<Http>,
) -> Result<Vec<SimulationResults>> {
    // send tx
    let pending_tx = provider
        .send_transaction(tx, None)
        .await
        .unwrap_or_else(|e| {
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
        match process_logs(
            MyLog {
                address: log.address,
                topics: log.topics,
                data: log.data,
            },
            provider.clone(),
        )
        .await
        {
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
