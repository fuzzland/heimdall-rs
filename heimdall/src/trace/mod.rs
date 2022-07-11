use std::{time::Instant, str::FromStr};
use clap::{AppSettings, Parser};
use ethers::{
    prelude::{Provider, Http, Middleware, Trace},
    types::{H256},
};
use heimdall_common::{io::logging::Logger, consts::TRANSACTION_HASH_REGEX};


#[derive(Debug, Clone, Parser)]
#[clap(about = "Trace the execution of an EVM transaction hash",
       after_help = "For more information, read the wiki: https://jbecker.dev/r/heimdall-rs/wiki",
       global_setting = AppSettings::DeriveDisplayOrder, 
       override_usage = "heimdall trace <TRANSACTION_HASH> [OPTIONS]")]
pub struct TraceArgs {
    
    /// The transaction hash to trace.
    #[clap(required=true)]
    pub transaction_hash: String,

    /// Set the output verbosity level, 1 - 5.
    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,

    /// The RPC provider to use for fetching target bytecode.
    #[clap(long="rpc-url", short, default_value = "", hide_default_value = true)]
    pub rpc_url: String,

    /// When prompted, always select the default value.
    #[clap(long, short)]
    pub default: bool,

}


#[allow(deprecated)]
pub fn trace(args: TraceArgs) {
    let now = Instant::now();
    let (logger, mut trace)= Logger::new(args.verbose.log_level().unwrap().as_str());

    let traces: Vec<Trace>;

    // determine whether or not the target is a transaction hash
    if TRANSACTION_HASH_REGEX.is_match(&args.transaction_hash) {

        // create new runtime block
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        
        // Fetch the raw traces from the RPC provider.
        traces = rt.block_on(async {

            // make sure the RPC provider isn't empty
            if &args.rpc_url.len() <= &0 {
                logger.error("tracing an on-chain transaction requires an RPC provider. Use `heimdall decode --help` for more information.");
                std::process::exit(1);
            }

            // create new provider
            let provider = match Provider::<Http>::try_from(&args.rpc_url) {
                Ok(provider) => provider,
                Err(_) => {
                    logger.error(&format!("failed to connect to RPC provider '{}' .", &args.rpc_url).to_string());
                    std::process::exit(1)
                }
            };

            // safely unwrap the transaction hash
            let transaction_hash = match H256::from_str(&args.transaction_hash) {
                Ok(transaction_hash) => transaction_hash,
                Err(_) => {
                    logger.error(&format!("failed to parse transaction hash '{}' .", &args.transaction_hash));
                    std::process::exit(1)
                }
            };

            // fetch the transaction from the node
            match provider.trace_transaction(transaction_hash).await {
                Ok(trace) => trace,
                Err(err) => {
                    println!("{:#?}", err);
                    logger.error(&format!("failed to fetch traces for '{}' . does your provider support 'debug.trace_transaction' ?", &args.transaction_hash));
                    std::process::exit(1)
                }
            }
            
        });
    }
    else {
        logger.error(&format!("invalid transaction hash '{}' .", &args.transaction_hash));
        std::process::exit(1)
    }

    for trace in traces {
        println!("{:#?}", trace);
    }

    let elapsed = now.elapsed();
    logger.debug(&format!("disassembly completed in {} ms.", elapsed.as_millis()).to_string());

}