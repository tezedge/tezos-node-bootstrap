// PoC, needs refactoring
use std::env;

mod types;
mod wrk_test;
mod bootstrap;
mod indexer_test;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            println!("No argument passed! Exiting");
        },
        2 => {
            match &args[1][..] {
                "-p" | "--performance-test"=> wrk_test::test_rpc_performance().unwrap(),
                "-i" | "--indexer-test" => indexer_test::test_indexer().unwrap(),
                _ => println!("Argument not recognized"),
            }
        },
        3 => {
            match &args[1][..] {
                "-b" | "--bootstrap" => {
                    bootstrap::start_bootstrap(args[2].parse().unwrap());
                },
                _ => println!("Argument not recognized"),
            }
        }
        _ => println!("Invalid argument"),
    }
}