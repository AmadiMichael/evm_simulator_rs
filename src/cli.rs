use clap::{App, Arg};

pub fn cli() -> Vec<String> {
    let matches = App::new("Rust CLI evm simulator")
        .version("1.0")
        .author("Michael Amadi")
        .about("A CLI app for simulating EVM chain's transactions and viewing transfers and approvals that will happen without actually executing it on mainnet")
        .arg(
            Arg::with_name("from")
                .long("from")
                .value_name("ADDRESS")
                .help("Source address")
                .required(true),
        )
        .arg(
            Arg::with_name("to")
                .long("to")
                .value_name("ADDRESS")
                .help("Destination address or contract address")
                .required(true),
        )
        .arg(
            Arg::with_name("data")
                .long("data")
                .value_name("DATA")
                .help("Transaction input data")
                .required(false),
        )
        .arg(
            Arg::with_name("value")
                .long("value")
                .value_name("VALUE")
                .help("Transaction value")
                .required(false),
        )
        .arg(
            Arg::with_name("block")
                .long("block")
                .value_name("BLOCK")
                .help("Block number")
                .required(false),
        )
        .arg(
            Arg::with_name("rpc")
                .long("rpc")
                .value_name("RPC_URL")
                .help("RPC URL")
                .required(false),
        )
        .arg(
            Arg::with_name("persist")
                .long("persist")
                .value_name("PERSIST")
                .help("Persist the state after simulations")
                .required(false),
        )
        .get_matches();

    let from = matches.value_of("from").unwrap();
    let to = matches.value_of("to").unwrap();
    let data = matches.value_of("data").unwrap_or_else(|| "0x");
    let value = matches.value_of("value").unwrap_or_else(|| "0");
    let block = matches.value_of("block").unwrap_or_default();
    let rpc = matches.value_of("rpc").unwrap_or_default();
    let persist = matches.value_of("persist").unwrap_or_default();

    vec![
        from.to_owned(),
        to.to_owned(),
        data.to_owned(),
        value.to_owned(),
        block.to_owned(),
        rpc.to_owned(),
        persist.to_owned(),
    ]
}
