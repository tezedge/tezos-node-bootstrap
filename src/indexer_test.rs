use assert_json_diff::assert_json_eq;
use std::{thread};
use std::time::Duration;

use crate::types::NodeType;
use crate::environment::{to_block_header, ocaml_node_indexer_root, tezedge_node_indexer_root};

fn get_indexer_data(node_type: NodeType) -> Result<reqwest::blocking::Response, failure::Error> {
    let mut response;

    let ocaml_node_indexer_root = ocaml_node_indexer_root();
    let tezedge_node_indexer_root = tezedge_node_indexer_root();
    let to_block_header = to_block_header();

    loop {
        match node_type {
            NodeType::Ocaml => response = reqwest::blocking::get(&format!("{}/explorer/block/{}", ocaml_node_indexer_root, to_block_header)),
            NodeType::Tezedge => response = reqwest::blocking::get(&format!("{}/explorer/block/{}", tezedge_node_indexer_root, to_block_header)),
            // we never make a request to an indexer with the TezedgeMaster node
            NodeType::TezedgeMaster => response = reqwest::blocking::get(&format!("{}/explorer/block/{}", tezedge_node_indexer_root, to_block_header)),
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

pub(crate) fn test_indexer() -> Result<(), failure::Error> {
    
    let mut response_tezedge;
    let mut response_ocaml;

    // wait for the indexer to be fully indexed to the chosen point
    get_indexer_data(NodeType::Ocaml)?;
    get_indexer_data(NodeType::Tezedge)?;

    let ocaml_node_indexer_root = ocaml_node_indexer_root();
    let tezedge_node_indexer_root = tezedge_node_indexer_root();
    let to_block_header = to_block_header();

    for n in 0..to_block_header {
        println!("Checking and comparing indexed block {}", n);
        response_ocaml = reqwest::blocking::get(&format!("{}/explorer/block/{}", ocaml_node_indexer_root, n))?;
        response_tezedge = reqwest::blocking::get(&format!("{}/explorer/block/{}", tezedge_node_indexer_root, n))?;

        let tezedge_json: serde_json::value::Value =
            serde_json::from_str(&response_tezedge.text()?).expect("JSON was not well-formatted");

        let ocaml_json: serde_json::value::Value =
            serde_json::from_str(&response_ocaml.text()?).expect("JSON was not well-formatted");
        
        assert_json_eq!(tezedge_json, ocaml_json);
    }
    println!("Json responses are identical!");
    Ok(())
}