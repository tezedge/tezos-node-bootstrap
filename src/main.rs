// PoC, needs refactoring
use std::env;
use std::str::FromStr;

use crate::types::NodeType;

mod types;
mod wrk_test;
mod bootstrap;
mod indexer_test;
mod sequential_request_test;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("No argument passed! Exiting")
    }

    match &args[1][..] {
        "-p" | "--performance-test" => {
            let nodes = nodes(&args);
            if nodes.len() < 2 {
                panic!("Expecting <2, 3> nodes!");
            }
            wrk_test::test_rpc_performance(level(&args), nodes, wrk_duration(&args)).unwrap()
        },
        "-i" | "--indexer-test" => {
            let nodes = nodes(&args);
            if nodes.len() != 2 {
                panic!("Expecting exact two nodes!");
            }
            indexer_test::test_indexer(
                level(&args),
                &nodes[0], indexer_url(&args, &nodes[0]),
                &nodes[1], indexer_url(&args, &nodes[1]),
            ).unwrap()
        },
        "-b" | "--bootstrap" => bootstrap::start_bootstrap(level(&args), nodes(&args)),
        "-s" | "--sequence" => sequential_request_test::test_sequential_requests(10, &nodes(&args)),
        _ => panic!("Argument not recognized"),
    }
}

fn level(args: &Vec<String>) -> i32 {
    let level = args
        .iter()
        .filter(|a| a.starts_with("--level="))
        .map(|a| a.replace("--level=", ""))
        .max().expect("No level arg: --level=");
    i32::from_str(&level).expect("Invalid level arg")
}

fn nodes(args: &Vec<String>) -> Vec<NodeType> {
    let mut nodes = Vec::new();
    for a in args {
        if a.starts_with("--node_") {
            let tmp = a.replace("--node_", "");
            let tmp: Vec<&str> = tmp.split("=").collect();
            nodes.push(NodeType {
                name: tmp[0].to_string(),
                url: tmp[1].to_string(),
            })
        }
    }
    if nodes.is_empty() {
        panic!("no nodes '--node_<name>=<url>' in args");
    }
    nodes
}

fn indexer_url(args: &Vec<String>, node: &NodeType) -> String {
    let indexer_param = &format!("--indexer_{}=", node.name);
    let url = args
        .iter()
        .filter(|a| a.starts_with(indexer_param))
        .map(|a| a.replace(indexer_param, ""))
        .max().expect(&format!("No indexer arg: {}", indexer_param));
    url
}

fn wrk_duration(args: &Vec<String>) -> i32 {
    let duration = args
        .iter()
        .filter(|a| a.starts_with("--wrk-duration="))
        .map(|a| a.replace("--wrk-duration=", ""))
        .max().expect("No wrk-duration arg: --wrk-duration=");
    i32::from_str(&duration).expect("Invalid level arg")
}