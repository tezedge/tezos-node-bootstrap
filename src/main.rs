// PoC, needs refactoring

extern crate reqwest;
extern crate serde;

#[derive(Debug)]
pub enum NodeType {
    Tezedge,
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
                _ => println!("Nope"),
            }
        },
        _ => println!("Invalid argument"),
    }
}

fn start_bootstrap() {
    let measure_tezedge = spawn_monitor_thread(NodeType::Tezedge).unwrap();
    let measure_ocaml = spawn_monitor_thread(NodeType::Ocaml).unwrap();

    measure_tezedge.join().unwrap();
    measure_ocaml.join().unwrap();
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
                        DateTime::parse_from_rfc3339("2019-09-28T08:14:24Z").unwrap();
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

fn run_wrk(branch: Branch) -> Result<serde_json::Value, failure::Error> {
    let output;
    match branch {
        Branch::Master => {
            //output = Command::new("wrk").args(&["-t1", "-c1", "-d30s", "-R1", "http://tezedge-node-run:18732/chains/main/blocks/100/helpers/baking_rights", "-s", "scripts/as_json.lua", "--", "out.json"]).output()?;
            output = Command::new("wrk").args(&["-t1", "-c1", "-d60s", "-R1", "http://127.0.0.1:8732/chains/main/blocks/100/helpers/baking_rights", "-s", "scripts/as_json.lua", "--", "out.json"]).output()?;
        }
        Branch::Modified => {
            //output = Command::new("wrk").args(&["-t1", "-c1", "-d30s", "-R1", "http://tezedge-node-run-master:38732/chains/main/blocks/100/helpers/baking_rights", "-s", "scripts/as_json.lua", "--", "out.json"]).output()?;
            output = Command::new("wrk").args(&["-t1", "-c1", "-d60s", "-R1", "http://127.0.0.1:8732/chains/main/blocks/100/helpers/baking_rights", "-s", "scripts/as_json.lua", "--", "out.json"]).output()?;
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
    let ret = serde_json::from_str(&json_out).expect("JSON wasn not well-formated");
    Ok(ret)
}

fn test_rpc_performance() -> Result<(), failure::Error> {
    let mut success: i32 = 0;

    for _i in 0..10 {
        let output_master = run_wrk(Branch::Master)?;
        let output_modified = run_wrk(Branch::Modified)?;

        let master_latency_mean = &output_master["latency"]["mean"].to_string().parse().unwrap();
        let modified_latency_mean = &output_modified["latency"]["mean"].to_string().parse().unwrap();

        println!("Master mean latency: {}", master_latency_mean);
        println!("Modified mean latency: {}", modified_latency_mean);

        let tolerance = master_latency_mean * 0.1;
        println!("Tolerance (10%): {}", tolerance);

        let delta = modified_latency_mean - master_latency_mean;
        println!("Delta: {}", delta);

        if delta < tolerance {
            success += 1;
        }
    }
    assert!(success >= 7);

    Ok(())
}