// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

use std::thread;
use std::time::Duration;

use assert_json_diff::assert_json_eq;
use url::Url;

use crate::configuration::IndexerTestEnv;

fn get_indexer_data(
    to_block_header: i32,
    node: Url,
    indexer: Url,
) -> Result<reqwest::blocking::Response, failure::Error> {
    loop {
        let response =
            reqwest::blocking::get(&format!("{}/explorer/block/{}", indexer, to_block_header));

        match response {
            Ok(res) => {
                if !res.status().is_success() {
                    println!("[{}] Indexer still indexing. Sleeping for 10s", node);
                    thread::sleep(Duration::from_secs(10));
                    continue;
                } else {
                    return Ok(res);
                }
            }
            Err(_e) => {
                println!("[{:?} or {}] Service not started yet", node, indexer);
                thread::sleep(Duration::from_secs(10));
                continue;
            }
        }
    }
}

pub(crate) fn test_indexer(env: IndexerTestEnv) -> Result<(), failure::Error> {
    let IndexerTestEnv {
        tezedge_node,
        tezedge_indexer,
        ocaml_node,
        ocaml_indexer,
        level,
    } = env;

    // wait for the indexer to be fully indexed to the chosen point
    get_indexer_data(level, tezedge_node, tezedge_indexer.clone())?;
    get_indexer_data(level, ocaml_node, ocaml_indexer.clone())?;

    for n in 0..level {
        println!("Checking and comparing indexed block {}", n);
        let response_node1 =
            reqwest::blocking::get(&format!("{}/explorer/block/{}", tezedge_indexer, n))?;
        let response_node2 =
            reqwest::blocking::get(&format!("{}/explorer/block/{}", ocaml_indexer, n))?;

        let tezedge_json: serde_json::value::Value =
            serde_json::from_str(&response_node1.text()?).expect("JSON was not well-formatted");

        let ocaml_json: serde_json::value::Value =
            serde_json::from_str(&response_node2.text()?).expect("JSON was not well-formatted");

        assert_json_eq!(tezedge_json, ocaml_json);
    }
    println!("Json responses are identical!");
    Ok(())
}
