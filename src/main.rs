use dotenv::dotenv;
use ethers::{
    abi::{decode_whole, ParamType, Token},
    contract::Multicall,
    core::{types::TransactionRequest, utils::Anvil},
    prelude::abigen,
    providers::{Http, Middleware, Provider},
    types::{Address, Bytes, Log, H256, U256},
    utils::{format_units, parse_ether},
};
use eyre::Result;
use std::convert::TryFrom;
use std::sync::Arc;

#[derive(Debug)]
enum Operation {
    Approval,
    Transfer,
    ApprovalForAll,
}

#[derive(Debug)]
struct TokenInfo {
    address: Address,
    name: String,
    symbol: String,
    decimals: U256,
}

#[derive(Debug)]
struct SimulatedInfo {
    operation: Operation,
    token_info: TokenInfo,
    from: Address,
    to: Address,
    amount: U256,
}

/// In Ethereum, transactions must be signed with a private key before they can be broadcast to the
/// network. Ethers-rs provides a way to customize this process by allowing
/// you to define a signer, called to sign transactions before they are sent.
#[tokio::main]
async fn main() -> Result<()> {
    let (from, to, data, value, block_number) = return_eip20_test_case();
    simulate(from, to, data, value, block_number).await?;

    let (from, to, data, value, block_number) = return_eip721_test_case();
    simulate(from, to, data, value, block_number).await?;

    Ok(())
}

async fn simulate(from: String, to: String, data: String, value: u64, block_number: u64) -> Result<()> {
    println!("Starting simulation...");
    dotenv().ok();
    let alchemy_api_key = std::env::var("ALCHEMY_API_KEY").expect("ALCHEMY_API_KEY must be set.");
    let rpc_url: &str = &("https://eth-mainnet.g.alchemy.com/v2/".to_owned() + &alchemy_api_key); // "http://127.0.0.1:8545"; //
    let anvil = Anvil::new()
        .fork(rpc_url)
        .fork_block_number(block_number)
        .spawn();
    let provider =
        Provider::<Http>::try_from(anvil.endpoint()).expect("could not instantiate HTTP Provider");

    // convert to required types and revert if any fails
    let from: Address = from.parse()?;
    let to: Address = to.parse()?;
    let data: Bytes = data.parse()?;

    // impersonate address
    provider.request("anvil_impersonateAccount", [from]).await?;

    // setup tx
    let tx = TransactionRequest::new()
        .from(from)
        .to(to)
        .value(parse_ether(value)?)
        .data(data);

    // send tx
    let pending_tx = provider.send_transaction(tx, None).await?;

    // await and get receipt and tx
    let receipt = pending_tx
        .await?
        .ok_or_else(|| eyre::format_err!("Failed"))?;
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
            Err(err) => panic!("{}", err),
        }
    }

    for (index, simulated_info) in simulated_infos.iter().enumerate() {
        let decimals: u32 = simulated_info.token_info.decimals.to_string().parse()?;
        let amount = format_units(simulated_info.amount, decimals).unwrap();

        println!(
            "detected {index}: 
                                    Opeartion: {:?},
                                    Token Info:
                                        Address: {:?},  
                                        Token Name: {:?}, 
                                        Symbol: {:?}, 
                                        Decimals: {:?},
                                    From: {:?},
                                    To: {:?},
                                    Amount: {:?}",
            simulated_info.operation,
            simulated_info.token_info.address,
            simulated_info.token_info.name,
            simulated_info.token_info.symbol,
            simulated_info.token_info.decimals,
            simulated_info.from,
            simulated_info.to,
            amount
        );
    }

    // stop impersonate address
    provider
        .request("anvil_stopImpersonatingAccount", [from])
        .await?;

    Ok(())
}

async fn checks(log: &Log, provider: Provider<Http>) -> Result<Option<SimulatedInfo>> {
    let approval: H256 =
        "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925".parse()?;
    let transfer: H256 =
        "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".parse()?;
    let approval_for_all: H256 =
        "0x17307eab39ab6107e8899845ad3d59bd9653f200f220920489ca2b5937696c31".parse()?;

    let checked_topics: [H256; 3] = [approval, transfer, approval_for_all];

    if checked_topics.contains(&log.topics[0]) {
        let decoded = match decode_whole(&[ParamType::Uint(256)], &log.data) {
            Ok(x) => x,
            Err(err) => panic!("decoding failed with err: {}", err),
        };

        let amount = match decoded[0] {
            Token::Uint(x) => x,
            _ => panic!("Wrong type decoded"),
        };

        let (name, symbol, decimals) = get_token_name_and_symbol(log.address, provider).await?;

        if log.topics[0] == approval {
            Ok(Some(SimulatedInfo {
                operation: Operation::Approval,
                token_info: TokenInfo {
                    name,
                    symbol,
                    decimals,
                    address: log.address,
                },
                from: Address::from(log.topics[1]),
                to: Address::from(log.topics[2]),
                amount,
            }))
        } else if log.topics[0] == transfer {
            Ok(Some(SimulatedInfo {
                operation: Operation::Transfer,
                token_info: TokenInfo {
                    name,
                    symbol,
                    decimals,
                    address: log.address,
                },
                from: Address::from(log.topics[1]),
                to: Address::from(log.topics[2]),
                amount,
            }))
        } else {
            Ok(Some(SimulatedInfo {
                operation: Operation::ApprovalForAll,
                token_info: TokenInfo {
                    name,
                    symbol,
                    decimals,
                    address: log.address,
                },
                from: Address::from(log.topics[1]),
                to: Address::from(log.topics[2]),
                amount,
            }))
        }
    } else {
        Ok(None)
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

    Ok((name.to_owned(), symbol.to_owned(), decimals))
}



fn return_eip20_test_case() -> (String, String, String, u64, u64) {
    // return a uniswap swap tx data

    let from = "0x448E0F9F42746F6165Dbe6E7B77149bB0F631E6E".to_owned();
    let to = "0x2Ec705D306b51e486B1bC0D6ebEE708E0661ADd1".to_owned();
    let data = "0x18cbafe500000000000000000000000000000000000000000000000000394425252270000000000000000000000000000000000000000000000000000035e2b98723e13d00000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000448e0f9f42746f6165dbe6e7b77149bb0f631e6e0000000000000000000000000000000000000000000000000000000064a876b70000000000000000000000000000000000000000000000000000000000000002000000000000000000000000e30bbec87855c8710729e6b8384ef9783c76379c000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".to_owned();
    let value: u64 = 0;
    let block_number: u64 = 17644319;

    (from, to, data, value, block_number)
}

fn return_eip721_test_case() -> (String, String, String, u64, u64) {
    // return a uniswap swap tx data

    let from = "0x77c5D44F392DD825A073C417EDe8C2f8bce603F6".to_owned();
    let to = "0x00000000000000ADc04C56Bf30aC9d3c0aAF14dC".to_owned();
    let data = "0xe7acab24000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000005e00000007b02230091a7ed01230072f7006a004d60a8d4e71d599b8104250f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000046000000000000000000000000000000000000000000000000000000000000004c00000000000000000000000000b818dc9d41732617dfc5bc8dff03dac632780e1000000000000000000000000000000e7ec00e7b300774b00001314b8610022b80000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000000000022000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000064ac23690000000000000000000000000000000000000000000000000000000064d501e50000000000000000000000000000000000000000000000000000000000000000360c6ebe0000000000000000000000000000000000000000710e918d59930ae50000007b02230091a7ed01230072f7006a004d60a8d4e71d599b8104250f0000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000d529ae9e86000000000000000000000000000000000000000000000000000000d529ae9e8600000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000300000000000000000000000076be3b62873462d2142405439777e971754e8e77000000000000000000000000000000000000000000000000000000000000282c000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000b818dc9d41732617dfc5bc8dff03dac632780e10000000000000000000000000000000000000000000000000000000000000001000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005543df729c0000000000000000000000000000000000000000000000000000005543df729c0000000000000000000000000000000a26b00c1f0df003000390027140000faa719000000000000000000000000000000000000000000000000000000000000004059577c8e8707f9b8896a85d4a59a2ef30647fb061287f000079b9fe1e5063474597f9bf2b77700bba355bd813f416da1c12048c8b976a222a3fcdbc92a7887aa000000000000000000000000000000000000000000000000000000000000007e0077c5d44f392dd825a073c417ede8c2f8bce603f60000000064add71eaab1b624b2bf2ba4bc33225f4eb7638e22f73aca43287493a1f63311f6c038a5d8ca9631edb8f32f3696d78963d536359f05834d595295a3189b2c0862236f6900000000000000000000000000000000000000000000000000000000000000282c0000000000000000000000000000000000000000000000000000000000000000000000000000360c6ebe".to_owned();
    let value: u64 = 0;
    let block_number: u64 = 17673303;

    (from, to, data, value, block_number)
}