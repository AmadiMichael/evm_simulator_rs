
use ethers::utils::format_units;
use super::types::SimulationResults;


pub fn print_result(simulated_infos: Vec<SimulationResults>) {
    if simulated_infos.len() == 0 {
        println!("No watched events detected!");
        return;
    }

    println!("\n\n\n\n\x1b[92m _____________________________________________________________________ SIMULATION RESULTS _____________________________________________________________________\n");
    for (index, simulated_info) in simulated_infos.iter().enumerate() {
        let decimals: u32 = simulated_info.token_info.decimals.to_string().parse().expect("failed to convert to u32");
        let amount = match decimals > 0 {
            true => format_units(simulated_info.amount, decimals).unwrap(),
            false => format!("{}", simulated_info.amount),
        };
        let id = match simulated_info.id {
            Some(id) => format!("{}", id),
            None => "".to_owned(),
        };

        println!(
            "  \x1b[94m{}. \x1b[0m{:?}
            Token Info:
                Standard: {:?},
                Address: {:?},  
                Token Name: {:?}, 
                Symbol: {:?}, 
                Decimals: {:?},
                
            Operation Info:
                From: {:?},
                To: {:?},
                id: {:?},
                Amount: {:?}
\n\x1b[92m________________________________________________________________________________________________________________________________________________________________\n",
            index + 1,
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
}