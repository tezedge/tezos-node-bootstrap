use std::net::SocketAddr;

use clap::{App, Arg, SubCommand};

pub struct SequentialTestEnv {
    pub cycles: i32,
    pub nodes: Vec<SocketAddr>,
}

impl SequentialTestEnv {
    pub fn from_args(args: &clap::ArgMatches) -> Self {
        let nodes: Vec<SocketAddr> = if let Some(nodes) = args.values_of("nodes") {
            nodes
                .map(|v| {
                    v.parse()
                        .expect("Provided value cannot be converted into valid uri")
                })
                .collect()
        } else {
            panic!("No nodes provided in the --nodes arg")
        };

        SequentialTestEnv {
            cycles: args
                .value_of("cycles")
                .unwrap_or("")
                .parse::<i32>()
                .expect("Provided value cannot be converted into valid i32"),
            nodes,
        }
    }
}

pub struct BootstrapEnv {
    pub level: i32,
    pub nodes: Vec<SocketAddr>,
}

impl BootstrapEnv {
    pub fn from_args(args: &clap::ArgMatches) -> Self {
        let nodes: Vec<SocketAddr> = if let Some(nodes) = args.values_of("nodes") {
            nodes
                .map(|v| {
                    v.parse()
                        .expect("Provided value cannot be converted into valid uri")
                })
                .collect()
        } else {
            panic!("No nodes provided in the --nodes arg")
        };

        BootstrapEnv {
            level: args
                .value_of("level")
                .unwrap_or("")
                .parse::<i32>()
                .expect("Provided value cannot be converted into valid i32"),
            nodes,
        }
    }
}

pub struct PerformanceTestEnv {
    pub level: i32,
    pub ocaml_node: SocketAddr,
    pub tezedge_new_node: SocketAddr,
    pub tezedge_old_node: SocketAddr,
    pub wrk_test_duration: u64,
}

impl PerformanceTestEnv {
    pub fn from_args(args: &clap::ArgMatches) -> Self {
        PerformanceTestEnv {
            level: args
                .value_of("level")
                .unwrap_or("")
                .parse::<i32>()
                .expect("Provided value cannot be converted into valid i32"),
            ocaml_node: args
                .value_of("ocaml-node")
                .unwrap_or("")
                .parse()
                .expect("Provided value cannot be converted into valid uri"),
            tezedge_new_node: args
                .value_of("tezedge-new-node")
                .unwrap_or("")
                .parse()
                .expect("Provided value cannot be converted into valid uri"),
            tezedge_old_node: args
                .value_of("tezedge-old-node")
                .unwrap_or("")
                .parse()
                .expect("Provided value cannot be converted into valid uri"),
            wrk_test_duration: args
                .value_of("wrk-test-duration")
                .unwrap_or("")
                .parse::<u64>()
                .expect("Provided value cannot be converted into valid u64"),
        }
    }
}

pub struct IndexerTestEnv {
    pub level: i32,
    pub ocaml_node: SocketAddr,
    pub tezedge_node: SocketAddr,
    pub tezedge_indexer: SocketAddr,
    pub ocaml_indexer: SocketAddr,
}

impl IndexerTestEnv {
    pub fn from_args(args: &clap::ArgMatches) -> Self {
        IndexerTestEnv {
            level: args
                .value_of("level")
                .unwrap_or("")
                .parse::<i32>()
                .expect("Provided value cannot be converted into valid i32"),
            ocaml_node: args
                .value_of("ocaml-node")
                .unwrap_or("")
                .parse()
                .expect("Provided value cannot be converted into valid uri"),
            tezedge_node: args
                .value_of("tezedge-node")
                .unwrap_or("")
                .parse()
                .expect("Provided value cannot be converted into valid uri"),
            tezedge_indexer: args
                .value_of("tezedge-indexer")
                .unwrap_or("")
                .parse()
                .expect("Provided value cannot be converted into valid uri"),
            ocaml_indexer: args
                .value_of("ocaml-indexer")
                .unwrap_or("")
                .parse()
                .expect("Provided value cannot be converted into valid uri"),
        }
    }
}

pub fn bootstrap_app() -> App<'static, 'static> {
    let app = App::new("CI bootstrap and test app")
        .version("0.1.0")
        .author("Adrian Nagy")
        .about("CI bootstraping and testing app")
        .setting(clap::AppSettings::AllArgsOverrideSelf)
        .subcommand(
            SubCommand::with_name("performance-test")
                .about("Performance test using wrk")
                .setting(clap::AppSettings::AllArgsOverrideSelf)
                .arg(
                    Arg::with_name("level")
                    .long("level")
                    .takes_value(true)
                    .value_name("NUM")
                    .help("Block level which is used in the test")
                )
                .arg(
                    Arg::with_name("ocaml-node")
                    .long("ocaml-node")
                    .takes_value(true)
                    .value_name("STRING")
                    .help("Ocaml node url")
                )
                .arg(
                    Arg::with_name("tezedge-new-node")
                    .long("tezedge-new-node")
                    .takes_value(true)
                    .value_name("STRING")
                    .help("Tezedge node url - with updated code from the pull request")
                )
                .arg(
                    Arg::with_name("tezedge-old-node")
                    .long("tezedge-old-node")
                    .takes_value(true)
                    .value_name("STRING")
                    .help("Tezedge node url - with old code from the target branch of the pull request")
                )
                .arg(
                    Arg::with_name("wrk-duration")
                    .long("wrk-duration")
                    .takes_value(true)
                    .value_name("STRING")
                    .help("Duration of the individual tests")
                )
            )
        .subcommand(
            SubCommand::with_name("indexer-test")
            .about("Indexer correctness test")
            .setting(clap::AppSettings::AllArgsOverrideSelf)
            .arg(
                Arg::with_name("level")
                .long("level")
                .takes_value(true)
                .value_name("NUM")
                .help("Block level which is used in the test as an upper bound")
            )
            .arg(
                Arg::with_name("ocaml-node")
                .long("ocaml-node")
                .takes_value(true)
                .value_name("STRING")
                .help("Ocaml node url")
            )
            .arg(
                Arg::with_name("tezedge-node")
                .long("tezedge-new-node")
                .takes_value(true)
                .value_name("STRING")
                .help("Tezedge node url - with updated code from the pull request")
            )
            .arg(
                Arg::with_name("tezedge-indexer")
                .long("tezedge-old-node")
                .takes_value(true)
                .value_name("STRING")
                .help("Indexer url connected to the tezedge node")
            )
            .arg(
                Arg::with_name("ocaml-indexer")
                .long("wrk-duration")
                .takes_value(true)
                .value_name("STRING")
                .help("Indexer url connected to the ocaml node")
            )
        )
        .subcommand(
            SubCommand::with_name("bootstrap")
            .about("Checks and waits for two nodes to be bootstrapped to the same level")
            .setting(clap::AppSettings::AllArgsOverrideSelf)
            .arg(
                Arg::with_name("level")
                .long("level")
                .takes_value(true)
                .value_name("NUM")
                .help("Block level which is used in the test as an upper bound")
            )
            .arg(
                Arg::with_name("nodes")
                .long("nodes")
                .takes_value(true)
                .multiple(true)
                .min_values(2)
                .value_name("STRING")
                .help("Node urls to be bootstrapped")
            )
        )
        .subcommand(
            SubCommand::with_name("sequential-test")
            .about("Checks and waits for two nodes to be bootstrapped to the same level")
            .setting(clap::AppSettings::AllArgsOverrideSelf)
            .arg(
                Arg::with_name("cycles")
                .long("cycles")
                .takes_value(true)
                .value_name("NUM")
                .help("Number of cycles to test")
            )
            .arg(
                Arg::with_name("nodes")
                .long("nodes")
                .takes_value(true)
                .multiple(true)
                .min_values(2)
                .max_values(2)
                .value_name("STRING")
                .help("Node urls to be bootstrapped")
            )
        );
    app
}
