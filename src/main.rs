use ethers::{
    prelude::{abigen},
    core::{types::TransactionRequest, utils::Anvil},
    providers::{Http, Middleware, Provider},
    types::{Bytes, Address, U256, H256, Log}, utils::{parse_ether, format_units}, abi::{decode_whole, ParamType, Token},
};
use eyre::Result;
use std::convert::TryFrom;
use dotenv::dotenv;
use std::sync::Arc;

#[derive(Debug)]
enum Operation {
    Approval,
    Transfer,
}

#[derive(Debug)]
struct SimulatedInfo {
    operation: Operation,
    token: Address,
    from: Address,
    to: Address,
    amount: U256,
}

/// In Ethereum, transactions must be signed with a private key before they can be broadcast to the
/// network. Ethers-rs provides a way to customize this process by allowing
/// you to define a signer, called to sign transactions before they are sent.
#[tokio::main]
async fn main() -> Result<()> {
    let from = "0x448E0F9F42746F6165Dbe6E7B77149bB0F631E6E";
    let to = "0x2Ec705D306b51e486B1bC0D6ebEE708E0661ADd1";
    let data = "0x18cbafe500000000000000000000000000000000000000000000000000394425252270000000000000000000000000000000000000000000000000000035e2b98723e13d00000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000448e0f9f42746f6165dbe6e7b77149bb0f631e6e0000000000000000000000000000000000000000000000000000000064a876b70000000000000000000000000000000000000000000000000000000000000002000000000000000000000000e30bbec87855c8710729e6b8384ef9783c76379c000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
    let value: u64 = 0;
    let block_number: u64 = 17644319;

    simulate(from, to, data, value, block_number).await?;

    Ok(())
}

async fn simulate(from: &str, to: &str, data: &str, value: u64, block_number: u64) -> Result<()> {
    dotenv().ok();
    let alchemy_api_key = std::env::var("ALCHEMY_API_KEY").expect("ALCHEMY_API_KEY must be set.");
    let rpc_url: &str = &("https://eth-mainnet.g.alchemy.com/v2/".to_owned() + &alchemy_api_key); // "http://127.0.0.1:8545"; //
    let anvil = Anvil::new().fork(rpc_url).fork_block_number(block_number).spawn();
    let provider = Provider::<Http>::try_from(anvil.endpoint()).expect("could not instantiate HTTP Provider");


    // convert to required types and revert if any fails
    let from: Address = from.parse()?;
    let to: Address = to.parse()?;
    let data: Bytes = data.parse()?;

    // impersonate address
    provider.request("anvil_impersonateAccount", [from]).await?;

    // setup tx
    let tx = TransactionRequest::new().from(from).to(to).value(parse_ether(value)?).data(data);

    // send tx
    let pending_tx = provider.send_transaction(tx, None).await?;

    // await and get receipt and tx
    let receipt = pending_tx.await?.ok_or_else(|| eyre::format_err!("Failed"))?;
    // let tx = provider.get_transaction(receipt.transaction_hash).await?;

    // println!("tx: {:?}", serde_json::to_string(&tx)?);
    // println!("receipt: {:?}", serde_json::to_string(&receipt)?);

    let logs = receipt.logs;
    // println!("logs: {:?}", logs);


    let mut simulated_infos: Vec<SimulatedInfo> = Vec::new();

    for log in logs.iter() {
        match checks(log) {
            Some(x) => simulated_infos.push(x),
            None => {}
        }
    }

    for (index, simulated_info) in simulated_infos.iter().enumerate() {
        let (name, symbol, decimals) = get_token_name_and_symbol(simulated_info.token, &provider).await;
        let decimals: u32 = decimals.to_string().parse()?;
        let amount = format_units(simulated_info.amount, decimals).unwrap();

        println!("detected {index}: 
                                    Opeartion: {:?},
                                    Token Address: {:?}: 
                                    Token Name, Symbol, Decimals: {name:?}, {symbol:?}, {decimals:?},
                                    From: {:?},
                                    To: {:?},
                                    Amount: {:?}", 
                                    simulated_info.operation, simulated_info.token, simulated_info.from, simulated_info.to, amount);
    }

    // stop impersonate address
    provider.request("anvil_stopImpersonatingAccount", [from]).await?;


    Ok(())
}


fn checks(log: &Log) -> Option<SimulatedInfo> {
    let approval: H256 = "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925".parse().unwrap();
    let transfer: H256 = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".parse().unwrap();
    let checked_topics: [H256; 2] = [approval, transfer];

    if checked_topics.contains(&log.topics[0]) {
        let mut amount: U256 = U256([0, 0, 0, 0]);
        let decoded = decode_whole(&[ParamType::Uint(256)], &log.data).unwrap();

        for token in decoded.iter() {
            amount = match token {
                Token::Uint(x) => *x,
                _ => panic!("Wrong type decoded")
            }
        }

        if approval == log.topics[0] {
            Some(SimulatedInfo { operation: Operation::Approval, token: log.address, from: Address::from(log.topics[1]), to: Address::from(log.topics[2]), amount: amount })
        } else {
            Some(SimulatedInfo { operation: Operation::Transfer, token: log.address, from: Address::from(log.topics[1]), to: Address::from(log.topics[2]), amount: amount })
        }
    } else {
        None
    }
}


async fn get_token_name_and_symbol(address: Address, provider: &Provider<Http>) -> (String, String, U256){
    abigen!(
        IERC20,
        r#"[
            function name() external view returns (string)
            function symbol() external view returns (string)
            function decimals() external view returns (uint256)
        ]"#,
    );

    let client = Arc::new(provider);
    let contract = IERC20::new(address, client);

    let name = contract.name().call().await;
    let symbol = contract.symbol().call().await;
    let decimals = contract.decimals().call().await;

    (name.unwrap().to_owned(), symbol.unwrap().to_owned(), decimals.unwrap())
}