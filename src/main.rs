mod commitment;
mod prover;
mod verifier;
mod challenge;
mod confidential_tx;
mod inner_product;
mod network;

use std::env;
use tokio::runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: cargo run -- <prover | verifier>");
        std::process::exit(1);
    }

    let mode = &args[1];
    let runtime = Runtime::new()?;

    match mode.as_str() {
        "prover" => {
            println!("Starting Prover...");
            runtime.block_on(prover::prover_main())?;
        }
        "verifier" => {
            println!("Starting Verifier...");
            runtime.block_on(verifier::verifier_main())?;
        }
        _ => {
            eprintln!("Invalid mode. Use 'prover' or 'verifier'.");
            std::process::exit(1);
        }
    }

    Ok(())
}