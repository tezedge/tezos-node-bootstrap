// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

// PoC, needs refactoring
use crate::configuration::{
    bootstrap_app, BootstrapEnv, IndexerTestEnv, RpcPerformanceTestEnv, RpcLatencyTestEnv, SequentialTestEnv,
};

mod bootstrap;
mod configuration;
mod indexer_test;
mod sequential_request_test;
mod types;
mod utils;
mod wrk;
mod wrk2;

fn main() {
    let matches = bootstrap_app().get_matches();

    if let Some(subcommand) = matches.subcommand_matches("bootstrap") {
        let env = BootstrapEnv::from_args(subcommand);
        bootstrap::start_bootstrap(env);
    } else if let Some(ref subcommand) = matches.subcommand_matches("performance-test") {
        let env = RpcPerformanceTestEnv::from_args(subcommand);
        if let Err(e) = wrk::test_rpc_performance(env) {
            panic!("Error in wrk tests: {}", e)
        }
    } else if let Some(ref subcommand) = matches.subcommand_matches("latency-test") {
        let env = RpcLatencyTestEnv::from_args(subcommand);
        if let Err(e) = wrk2::test_rpc_performance(env) {
            panic!("Error in wrk tests: {}", e)
        }
    } else if let Some(ref subcommand) = matches.subcommand_matches("indexer-test") {
        let env = IndexerTestEnv::from_args(subcommand);
        if let Err(e) = indexer_test::test_indexer(env) {
            panic!("Error in indexer tests: {}", e)
        }
    } else if let Some(ref subcommand) = matches.subcommand_matches("sequential-test") {
        let env = SequentialTestEnv::from_args(subcommand);
        sequential_request_test::test_sequential_requests(env)
    }
}
