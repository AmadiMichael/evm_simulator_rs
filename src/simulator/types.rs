use ethers::{
    types::{Address, Bytes, H256, U256},
    utils::parse_ether,
};
use eyre::Result;
use std::process;

#[derive(Debug)]
pub struct MyLog {
    pub address: Address,
    pub topics: Vec<H256>,
    pub data: Bytes,
}

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
    None,
    Eip20,
    Eip721,
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
pub struct SimulationResults {
    pub operation: Operation,
    pub token_info: TokenInfo,
    pub from: Address,
    pub to: Address,
    pub id: Option<U256>,
    pub amount: U256,
}

#[derive(Debug)]
pub enum BlockNumberType {
    Past(u64),
    Latest,
}

#[derive(Debug)]
pub struct SimulationParams {
    pub from: Address,
    pub to: Address,
    pub data: Bytes,
    pub value: U256,
    pub block_number: BlockNumberType,
    pub rpc_url: Option<String>,
    pub persist: bool,
}

impl SimulationParams {
    pub fn new(args: &Vec<String>) -> Result<Self, &str> {
        let from = args[0].parse::<Address>();
        let from = match from {
            Ok(f) => f,
            _ => return Err("invalid 'from' address provided"),
        };

        let to = args[1].parse::<Address>();
        let to = match to {
            Ok(t) => t,
            _ => return Err("Invalid 'to' address provided"),
        };

        let data;
        if args[3] == "" {
            data = "0x".parse::<Bytes>();
        } else {
            data = args[2].parse::<Bytes>();
        }
        let data = match data {
            Ok(d) => d,
            _ => return Err("Invalid 'input data' provided"),
        };

        let value = parse_ether(args[3].as_str());
        let value = match value {
            Ok(val) => val,
            _ => return Err("Invalid ether value provided"),
        };

        let block_number = if args[4].len() == 0 {
            BlockNumberType::Latest
        } else {
            let block_number = args[4].parse::<u64>();
            let block_number = match block_number {
                Ok(num) => BlockNumberType::Past(num),
                _ => return Err("Block number parsed in not a valid number. To use the current block number, parse in an empty string e.g '' or don't specify a block number at all"),
            };
            block_number
        };

        let rpc_url = if args[5].len() == 0 {
            None
        } else {
            Some(args[5].to_owned())
        };

        let persist = match args[6].len() {
            0 => false,
            _ => args[6].parse::<bool>().unwrap_or_else(|_| {
                eprintln!("invalid boolean parameter for field 'persist'.");
                process::exit(1)
            }),
        };

        Ok(SimulationParams {
            from,
            to,
            data,
            value,
            block_number,
            rpc_url,
            persist,
        })
    }
}
