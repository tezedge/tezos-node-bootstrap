use std::thread;
use std::time::Duration;

use assert_json_diff::assert_json_eq;

use crate::types::NodeType;

fn get_indexer_data(to_block_header: i32, node_type: &NodeType, indexer_url: String) -> Result<reqwest::blocking::Response, failure::Error> {
    loop {
        let response = reqwest::blocking::get(&format!("{}/explorer/block/{}", indexer_url, to_block_header));

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
                println!("[{:?} or {}] Service not started yet", node_type.to_string(), indexer_url);
                thread::sleep(Duration::from_secs(10));
                continue;
            }
        }
    }
}

pub(crate) fn test_indexer(
    to_block_header: i32,
    node1: &NodeType, node1_indexer_url: String,
    node2: &NodeType, node2_indexer_url: String) -> Result<(), failure::Error> {

    // wait for the indexer to be fully indexed to the chosen point
    get_indexer_data(to_block_header, node1, node1_indexer_url)?;
    get_indexer_data(to_block_header, node2, node2_indexer_url)?;

    for n in 0..to_block_header {
        println!("Checking and comparing indexed block {}", n);
        let response_node1 = reqwest::blocking::get(&format!("{}/explorer/block/{}", node1_indexer_url, n))?;
        let response_node2 = reqwest::blocking::get(&format!("{}/explorer/block/{}", node2_indexer_url, n))?;

        let tezedge_json: serde_json::value::Value =
            serde_json::from_str(&response_node1.text()?).expect("JSON was not well-formatted");

        let ocaml_json: serde_json::value::Value =
            serde_json::from_str(&response_node2.text()?).expect("JSON was not well-formatted");

        assert_json_eq!(tezedge_json, ocaml_json);
    }
    println!("Json responses are identical!");
    Ok(())
}