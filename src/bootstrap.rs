use std::net::SocketAddr;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use reqwest;

// use crate::types::NodeType;
use crate::configuration::BootstrapEnv;

pub(crate) fn start_bootstrap(env: BootstrapEnv) {
    let BootstrapEnv { nodes, level } = env;

    let mut joins = Vec::new();
    for node in nodes {
        joins.push(spawn_monitor_thread(node, level).unwrap())
    }

    for join in joins {
        join.join().unwrap();
    }
}

fn spawn_monitor_thread(
    node: SocketAddr,
    bootstrap_level: i32,
) -> Result<JoinHandle<()>, failure::Error> {
    Ok(thread::spawn(move || {
        let now = Instant::now();

        let bootstrapping_tezedge = create_monitor_node_thread(node, bootstrap_level);
        bootstrapping_tezedge.join().unwrap();

        let elapsed = now.elapsed();
        let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
        println!("[{}] Duration in seconds: {}", node, sec);
    }))
}

fn create_monitor_node_thread(node: SocketAddr, bootstrap_level: i32) -> JoinHandle<()> {
    let mut active = false;
    let bootstrap_monitoring_thread = thread::spawn(move || loop {
        match is_bootstrapped(&node) {
            Ok(response_string) => {
                active = true;
                // empty string means, the rpc server is running, but the bootstraping has not started yet
                if response_string != "" {
                    // let block_timestamp = DateTime::parse_from_rfc3339(&response_string).unwrap();
                    let block_level: i32 = response_string.parse().unwrap();

                    if block_level >= bootstrap_level {
                        println!("[{}] Done Bootstrapping", node);
                        break;
                    } else {
                        println!("[{}] Bootstrapping . . . level: {}", node, response_string);
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
            Err(e) => {
                // panic!("Error in bootstrap check: {}", e);
                // NOTE: This should be handled more carefully
                if !active {
                    println!("[{}] Waiting for node to run", node);
                    println!("[{}] Error: {}", node, e);
                } else {
                    // when the node was 'active, i.e. was responding to the head reqeusts, and suddenly there is an error in the request
                    // it means the node encounterred some error and exited
                    panic!("[{}] The watched node has exited: {}", node, e)
                }

                thread::sleep(Duration::from_secs(10));
            }
        }
    });
    bootstrap_monitoring_thread
}

#[allow(dead_code)]
fn is_bootstrapped(node: &SocketAddr) -> Result<String, reqwest::Error> {
    let response = reqwest::blocking::get(&format!("{}/chains/main/blocks/head", node))?;

    // if there is no response, the node has not started bootstrapping
    if response.status().is_success() {
        let response_node: serde_json::value::Value =
            serde_json::from_str(&response.text()?).expect("JSON was not well-formatted");

        Ok(response_node["header"]["level"]
            .to_string()
            .replace("\"", ""))
    } else {
        Ok(String::new())
    }
}
