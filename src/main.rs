// PoC, needs refactoring
extern crate reqwest;
extern crate serde;

#[derive(Debug)]
pub enum NodeType {
    Tezedge,
    TezedgeMaster,
    Ocaml,
}

pub enum Branch {
    Master,
    Modified,
}

use chrono::DateTime;
use std::fmt;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use std::env;
use std::process::Command;
use failure::bail;
use assert_json_diff::assert_json_eq;

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            println!("No argument passed! Exiting");
        },
        2 => {
            match &args[1][..] {
                "-b" | "--bootstrap" => {
                    start_bootstrap();
                },
                "-p" | "--performance-test"=> test_rpc_performance().unwrap(),
                "-i" | "--indexer-test" => test_indexer().unwrap(),
                _ => println!("Argument not recognized"),
            }
        },
        _ => println!("Invalid argument"),
    }
}

fn start_bootstrap() {
    let measure_tezedge = spawn_monitor_thread(NodeType::Tezedge).unwrap();
    let measure_ocaml = spawn_monitor_thread(NodeType::Ocaml).unwrap();
    let measure_tezedge_master = spawn_monitor_thread(NodeType::TezedgeMaster).unwrap();

    measure_tezedge.join().unwrap();
    measure_ocaml.join().unwrap();
    measure_tezedge_master.join().unwrap();
}

fn spawn_monitor_thread(node_type: NodeType) -> Result<JoinHandle<()>, failure::Error> {
    Ok(thread::spawn(move || {
        let now = Instant::now();

        let bootstrapping_tezedge = create_monitor_node_thread(node_type);
        bootstrapping_tezedge.join().unwrap();

        let elapsed = now.elapsed();
        let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
        println!("[tezedge] Duration in seconds: {}", sec);
    }))
}

fn create_monitor_node_thread(node: NodeType) -> JoinHandle<()> {
    let bootstrap_monitoring_thread = thread::spawn(move || loop {
        match is_bootstrapped(&node) {
            Ok(s) => {
                // empty string means, the rpc server is running, but the bootstraping has not started yet
                if s != "" {
                    let desired_timestamp =
                        DateTime::parse_from_rfc3339("2019-10-12T22:23:06Z").unwrap();
                    let block_timestamp = DateTime::parse_from_rfc3339(&s).unwrap();

                    if block_timestamp >= desired_timestamp {
                        println!("[{}] Done Bootstrapping", node.to_string());
                        break;
                    } else {
                        println!(
                            "[{}] Bootstrapping . . . timestamp: {}",
                            node.to_string(),
                            s
                        );
                        thread::sleep(Duration::from_secs(10));
                    }
                } else {
                    println!(
                        "[{}] Waiting for node to start bootstrapping...",
                        node.to_string()
                    );
                    thread::sleep(Duration::from_secs(10));
                }
            }
            Err(_e) => {
                // panic!("Error in bootstrap check: {}", e);
                // NOTE: This should be handled more carefully
                println!("[{}] Waiting for node to run", node.to_string());
                println!("[{}] Error: {}", node.to_string(), _e);

                thread::sleep(Duration::from_secs(10));
            }
        }
    });
    bootstrap_monitoring_thread
}

#[allow(dead_code)]
fn is_bootstrapped(node: &NodeType) -> Result<String, reqwest::Error> {
    let response;
    match node {
        NodeType::Tezedge => {
            response =
                reqwest::blocking::get("http://tezedge-node-run:18732/chains/main/blocks/head")?;
        }
        NodeType::Ocaml => {
            response =
                reqwest::blocking::get("http://ocaml-node-run:8732/chains/main/blocks/head")?;
        }
        NodeType::TezedgeMaster => {
            response = 
                reqwest::blocking::get("http:/tezedge-master-node-run:28732/chains/main/blocks/head")?;
        }
    }
    // if there is no response, the node has not started bootstrapping
    if response.status().is_success() {
        let response_node: serde_json::value::Value =
            serde_json::from_str(&response.text()?).expect("JSON was not well-formatted");

        Ok(response_node["header"]["timestamp"]
            .to_string()
            .replace("\"", ""))
    } else {
        Ok(String::new())
    }
}

fn run_wrk(branch: Branch, rpc: &str) -> Result<serde_json::Value, failure::Error> {
    let output;
    match branch {
        Branch::Master => {
            output = Command::new("wrk").args(&["-t1", "-c1", "-d30s", "-R1", &format!("{}/{}", "http://tezedge-node-run:18732", rpc), "-s", "/scripts/as_json.lua", "--", "out.json"])
            .current_dir("/")
            .output()?;
            // output = Command::new("wrk").args(&["-t1", "-c1", "-d30s", "-R1", &format!("{}/{}", "http://116.202.128.230:8732", rpc), "-s", "scripts/as_json.lua", "--", "out.json"])
            // .current_dir("/")
            // .output()?;
        }
        Branch::Modified => {
            output = Command::new("wrk").args(&["-t1", "-c1", "-d30s", "-R1", &format!("{}/{}", "http://tezedge-master-node-run:28732", rpc), "-s", "/scripts/as_json.lua", "--", "out.json"])
            .current_dir("/")
            .output()?;
            // output = Command::new("wrk").args(&["-t1", "-c1", "-d30s", "-R1", &format!("{}/{}", "http://116.202.128.230:8732", rpc), "-s", "scripts/as_json.lua", "--", "out.json"])
            // .current_dir("/")
            // .output()?;
        }
    }

    if !output.status.success() {
        bail!("Command executed with failing error code");
    }

    // get the last line of the output, which is a json object
    let json_out = String::from_utf8(output.stdout)?
        .lines()
        .last()
        .unwrap().to_string();

    // println!("{}", json_out);
    let ret = serde_json::from_str(&json_out).expect("JSON was not well-formated");
    Ok(ret)
}

fn test_rpc_performance() -> Result<(), failure::Error> {
    let rpcs = vec![
        "chains/main/blocks/100/helpers/baking_rights",
        "chains/main/blocks/100/helpers/endorsing_rights",
        "chains/main/blocks/100/context/constants",
        "chains/main/blocks/100/votes/listings",
        "chains/main/blocks/100",
        "chains/main/blocks/100/header",
        "chains/main/blocks/100/context/raw/bytes/cycle"
    ];

    let path = env::current_dir()?;
    println!("The current directory is {}", path.display());

    for rpc in rpcs.into_iter() {
        println!("Running wrk for rpc: {}", rpc);
        let output_master = run_wrk(Branch::Master, &rpc)?;
        let output_modified = run_wrk(Branch::Modified, &rpc)?;

        let master_latency_max = &output_master["latency"]["max"].to_string().parse().unwrap();
        let modified_latency_max = &output_modified["latency"]["max"].to_string().parse().unwrap();

        println!("Master max latency: {}", master_latency_max);
        println!("Modified max latency: {}", modified_latency_max);

        let tolerance = master_latency_max * 0.1;
        println!("Tolerance (10%): {}", tolerance);

        let delta = modified_latency_max - master_latency_max;
        println!("Delta: {}", delta);

        // TODO: fail the test if the d
        if delta < tolerance {
            println!("OK");
        } else {
            println!("FAIL");
        }
        println!("");
    }

    Ok(())
}

fn get_indexer_data(node_type: NodeType) -> Result<reqwest::blocking::Response, failure::Error> {
    let mut response;

    loop {
        match node_type {
            NodeType::Ocaml => response = reqwest::blocking::get("http://tz-indexer-ocaml:8002/explorer/block/10000"),
            NodeType::Tezedge => response = reqwest::blocking::get("http://tz-indexer-tezedge:8002/explorer/block/10000"),
            // we never make a request to an indexer with the TezedgeMaster node
            NodeType::TezedgeMaster =>  response = reqwest::blocking::get("http://tz-indexer-tezedge:8002/explorer/block/1"),
        }

        match response {
            Ok(res) => {
                if !res.status().is_success() {
                    println!("[{}] Indexer still indexing. Sleeping for 10s", node_type.to_string());
                    thread::sleep(Duration::from_secs(10));
                    continue;
                } else {
                    return Ok(res)
                }
            },
            Err(_e) => {
                println!("[{:?}] Service not started yet", node_type.to_string());
                thread::sleep(Duration::from_secs(10));
                continue;
            }
        }
    }
}

fn test_indexer() -> Result<(), failure::Error> {
    
    let mut response_tezedge;
    let mut response_ocaml;

    // wait for the indexer to be fully indexed to the chosen point
    get_indexer_data(NodeType::Ocaml)?;
    get_indexer_data(NodeType::Tezedge)?;

    for n in 0..30000 {
        println!("Checking and comparing indexed block {}", n);
        response_ocaml = reqwest::blocking::get(&format!("http://tz-indexer-ocaml:8002/explorer/block/{}", n))?;
        response_tezedge = reqwest::blocking::get(&format!("http://tz-indexer-tezedge:8002/explorer/block/{}", n))?;

        let tezedge_json: serde_json::value::Value =
            serde_json::from_str(&response_tezedge.text()?).expect("JSON was not well-formatted");

        let ocaml_json: serde_json::value::Value =
            serde_json::from_str(&response_ocaml.text()?).expect("JSON was not well-formatted");
        
        assert_json_eq!(tezedge_json, ocaml_json);
    }
    println!("Json responses are identical!");
    Ok(())
}