use std::collections::HashMap;
use std::env;
use std::process::Command;

use failure::bail;
use itertools::Itertools;

use crate::types::{Branch, NodeType, WrkResult};

type WrkResultMap = HashMap<Branch, WrkResult>;

fn run_wrk(branch: Branch, node: &NodeType, rpc: &str, duration: i32) -> Result<WrkResult, failure::Error> {
    let url = format!("{}/{}", node.url, rpc);

    // local testing
    // let master_url = format!("{}/{}", "http://116.202.128.230:28732", rpc);
    // let modified_url = format!("{}/{}", "http://116.202.128.230:18732", rpc);
    // let ocaml_url = format!("{}/{}", "http://116.202.128.230:10000", rpc);
    let duration_string = &format!("-d{}s", duration.clone());

    let mut wrk_args = vec![
        "-t1",
        "-c1",
        duration_string,
        //"-R1000",
        "--timeout",
        "30s",
        "-s",
        "/scripts/as_json.lua",
        "--",
        "out.json"
    ];
    println!("Running wrk for {:?} with arguments: {:?}", branch, &wrk_args);

    wrk_args.insert(5, &url);

    let output = Command::new("wrk").args(&wrk_args)
        .current_dir("/")
        .output()?;

    if !output.status.success() {
        println!("{:?}", output);
        bail!("Command executed with failing error code");
    }

    // get the last line of the output, which is a json object
    let json_out = String::from_utf8(output.stdout)?
        .lines()
        .last()
        .unwrap().to_string();

    let ret: WrkResult = serde_json::from_str(&json_out).expect("JSON was not well-formated");
    Ok(ret)
}

pub(crate) fn test_rpc_performance(block_header: i32, nodes: Vec<NodeType>, duration: i32) -> Result<(), failure::Error> {
    let current_cycle = block_header / 2048;
    let rpcs = vec![
        format!("chains/main/blocks/{}/helpers/baking_rights?all=true&cycle={}", block_header, current_cycle + 1),
        format!("chains/main/blocks/{}/helpers/endorsing_rights?all&cycle={}", block_header, current_cycle + 1),
        format!("chains/main/blocks/{}/context/constants", block_header),
        format!("chains/main/blocks/{}/votes/listings", block_header),
        // format!("chains/main/blocks/{}/votes/proposals", block_header),
        // format!("chains/main/blocks/{}/votes/current_proposal", block_header),
        // format!("chains/main/blocks/{}/votes/ballot_list", block_header),
        // format!("chains/main/blocks/{}/votes/current_quorum", block_header),
        format!("chains/main/blocks/{}", block_header),
        format!("chains/main/blocks/{}/header", block_header),
        format!("chains/main/blocks/{}/context/raw/bytes/cycle", block_header),
        format!("/chains/main/blocks/{}/context/raw/json/cycle/0", block_header),
        // format!("/chains/main/blocks/{}/operations", block_header),
        // format!("/chains/main/blocks/{}/context/delegates/tz1PirboZKFVqkfE45hVLpkpXaZtLk3mqC17", block_header),

        // // first smart contract on the carthagenet (level 734)
        // format!("/chains/main/blocks/{}/context/contracts/KT1T2V8prXxe2uwanMim7TYHsXMmsrygGbxG", block_header),

        // format!("/chains/main/blocks/{}/context/contracts/tz1PirboZKFVqkfE45hVLpkpXaZtLk3mqC17", block_header),

        //"chains/main/blocks/100/context/delegates/tz1PirboZKFVqkfE45hVLpkpXaZtLk3mqC17",
    ];

    let path = env::current_dir()?;
    println!("The current directory is {}", path.display());

    for rpc in rpcs.into_iter() {
        println!("Running wrk for rpc: {}", rpc);
        let mut outputs: WrkResultMap = HashMap::new();

        for (idx, node) in nodes.iter().enumerate() {
            let branch = Branch {
                name: node.name.clone(),
                sort_key: idx,
            };
            outputs.insert(branch.clone(), run_wrk(branch.clone(), &node, &rpc, duration.clone())?);
        }

        calculate_and_display_statistics(&outputs)?;
    }

    Ok(())
}

fn calculate_and_display_statistics(wrk_results: &WrkResultMap) -> Result<(), failure::Error> {
    for (res_key, res_val) in wrk_results {
        println!("{:?} thoughtput: {}req/s", res_key, calc_throughput(res_val.requests(), res_val.duration())?);
        println!("{:?} max latency: {}ms", res_key, calc_max_latency(res_val.latency_max())?);
        println!();
    }
    calc_deltas(wrk_results)?;
    println!("------------------------------------------------------");
    println!();
    Ok(())
}

fn calc_throughput(req: &f32, dur: &f32) -> Result<f32, failure::Error> {
    Ok(req / (dur * 0.000001))
}

fn calc_max_latency(lat: &f32) -> Result<f32, failure::Error> {
    Ok(lat * 0.001)
}

fn calc_deltas(wrk_results: &WrkResultMap) -> Result<(), failure::Error> {
    let keys = wrk_results.keys()
        .into_iter()
        .sorted_by_key(|k| k.sort_key)
        .collect_vec();

    println!("Deltas compared to ocaml node: ");
    for combo in keys.into_iter().combinations(2) {
        let left = combo[0];
        let right = combo[1];

        let left_max_latency = wrk_results.get(left).unwrap().latency_max();
        let right_max_latency = wrk_results.get(right).unwrap().latency_max();

        let delta = (left_max_latency - right_max_latency) * 0.001;

        println!("\t {} - {}: {}ms", left.name, right.name, delta);
    }

    Ok(())
}
