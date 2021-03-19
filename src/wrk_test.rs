use std::collections::HashMap;
use std::process::Command;

use failure::bail;
use itertools::Itertools;

use crate::configuration::PerformanceTestEnv;
use crate::types::{Branch, BranchType, WrkResult};

type WrkResultMap = HashMap<Branch, WrkResult>;

fn run_wrk(branch: &Branch, rpc: &str, duration: &u64) -> Result<WrkResult, failure::Error> {
    let url = format!("{}/{}", branch.url, rpc);

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
        "out.json",
    ];
    println!(
        "Running wrk for {:?} with arguments: {:?}",
        branch.url.domain().unwrap_or(""),
        &wrk_args
    );
    println!();

    wrk_args.insert(5, &url);

    let output = Command::new("wrk")
        .args(&wrk_args)
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
        .unwrap()
        .to_string();

    let ret: WrkResult = serde_json::from_str(&json_out).expect("JSON was not well-formated");
    Ok(ret)
}

pub(crate) fn test_rpc_performance(env: PerformanceTestEnv) -> Result<(), failure::Error> {
    let PerformanceTestEnv {
        tezedge_new_node,
        tezedge_old_node,
        ocaml_node,
        wrk_test_duration,
        level,
        max_latency_threshold,
        throughput_threshold,
    } = env;

    let current_cycle = level / 2048;
    let rpcs = vec![
        format!(
            "chains/main/blocks/{}/helpers/baking_rights?all=true&cycle={}",
            level,
            current_cycle + 1
        ),
        format!(
            "chains/main/blocks/{}/helpers/endorsing_rights?all&cycle={}",
            level,
            current_cycle + 1
        ),
        format!("chains/main/blocks/{}/helpers/baking_rights", level),
        format!("chains/main/blocks/{}/helpers/endorsing_rights", level),
        format!("chains/main/blocks/{}/context/constants", level),
        format!("chains/main/blocks/{}/votes/listings", level),
        // format!("chains/main/blocks/{}/votes/proposals", level),
        // format!("chains/main/blocks/{}/votes/current_proposal", level),
        // format!("chains/main/blocks/{}/votes/ballot_list", level),
        // format!("chains/main/blocks/{}/votes/current_quorum", level),
        format!("chains/main/blocks/{}", level),
        format!("chains/main/blocks/{}/header", level),
        format!("chains/main/blocks/{}/context/raw/bytes/cycle", level),
        format!("/chains/main/blocks/{}/context/raw/json/cycle/0", level),
        // format!("/chains/main/blocks/{}/operations", level),
        // format!("/chains/main/blocks/{}/context/delegates/tz1PirboZKFVqkfE45hVLpkpXaZtLk3mqC17", level),

        // // first smart contract on the carthagenet (level 734)
        // format!("/chains/main/blocks/{}/context/contracts/KT1T2V8prXxe2uwanMim7TYHsXMmsrygGbxG", level),

        // format!("/chains/main/blocks/{}/context/contracts/tz1PirboZKFVqkfE45hVLpkpXaZtLk3mqC17", level),

        //"chains/main/blocks/100/context/delegates/tz1PirboZKFVqkfE45hVLpkpXaZtLk3mqC17",
    ];

    for rpc in rpcs.into_iter() {
        println!("Running wrk for rpc: {}", rpc);
        println!();
        let mut outputs: WrkResultMap = HashMap::new();

        let ocaml = Branch::new(0, ocaml_node.clone(), BranchType::Ocaml);
        let tezedge_new = Branch::new(1, tezedge_new_node.clone(), BranchType::Feature);
        let tezedge_old = Branch::new(2, tezedge_old_node.clone(), BranchType::Stable);

        outputs.insert(ocaml.clone(), run_wrk(&ocaml, &rpc, &wrk_test_duration)?);
        outputs.insert(
            tezedge_new.clone(),
            run_wrk(&tezedge_new, &rpc, &wrk_test_duration)?,
        );
        outputs.insert(
            tezedge_old.clone(),
            run_wrk(&tezedge_old, &rpc, &wrk_test_duration)?,
        );

        calculate_and_display_statistics(&outputs, max_latency_threshold, throughput_threshold);
    }

    Ok(())
}

fn calculate_and_display_statistics(
    wrk_results: &WrkResultMap,
    max_latency_threshold: f32,
    throughput_threshold: f32,
) {
    for (res_key, res_val) in wrk_results {
        println!(
            "{:?} thoughtput: {}req/s",
            res_key.url.domain().unwrap_or(""),
            calc_throughput(res_val.requests(), res_val.duration())
        );
        println!(
            "{:?} max latency: {}ms",
            res_key.url.domain().unwrap_or(""),
            calc_max_latency(res_val.latency_max())
        );
        println!();
    }
    calc_deltas(wrk_results, max_latency_threshold, throughput_threshold);
    println!("------------------------------------------------------");
    println!();
}

fn calc_throughput(req: &f32, dur: &f32) -> f32 {
    req / (dur * 0.000001)
}

fn calc_max_latency(lat: &f32) -> f32 {
    lat * 0.001
}

fn calc_deltas(wrk_results: &WrkResultMap, max_latency_threshold: f32, throughput_threshold: f32) {
    let keys = wrk_results
        .keys()
        .into_iter()
        .sorted_by_key(|k| k.sort_key)
        .collect_vec();

    println!("Deltas compared to ocaml node: ");
    for combo in keys.clone().into_iter().combinations(2) {
        let left = combo[0];
        let right = combo[1];

        let left_result = wrk_results.get(left).unwrap();
        let right_result = wrk_results.get(right).unwrap();

        let left_node = left.url.domain().unwrap_or("");
        let right_node = right.url.domain().unwrap_or("");

        let left_max_latency = left_result.latency_max();
        let right_max_latency = right_result.latency_max();

        let left_max_throughput = calc_throughput(left_result.requests(), left_result.duration());
        let right_max_throughput =
            calc_throughput(right_result.requests(), right_result.duration());

        let delta_latency = (left_max_latency - right_max_latency) * 0.001;
        let delta_throughput = left_max_throughput - right_max_throughput;

        println!("\t {} - {}: {}ms", left_node, right_node, delta_latency);
        println!(
            "\t {} - {}: {}req/s",
            left_node, right_node, delta_throughput
        );
        println!();
    }

    // only compare, when stable is present
    if let Some(stable_key) = keys
        .clone()
        .into_iter()
        .filter(|key| key.branch_type == BranchType::Stable)
        .last()
    {
        let new_key = keys
            .into_iter()
            .filter(|key| key.branch_type == BranchType::Feature)
            .last()
            .unwrap();

        let stable = wrk_results.get(&stable_key).unwrap();
        let new = wrk_results.get(&new_key).unwrap();

        if new.latency_max() > stable.latency_max() {
            // fail the test if the 10% performance happened
            if stable.latency_max() * max_latency_threshold
                < new.latency_max() - stable.latency_max()
            {
                panic!("[Max Latency] Perforamnce regression greater than {}%!", max_latency_threshold)
            }
        }

        if new.requests() < stable.requests() {
            // fail the test if the 10% performance happened
            if stable.requests() * throughput_threshold < stable.requests() - new.requests() {
                panic!("[Troughput] Perforamnce regression greater than {}%!", throughput_threshold)
            }
        }
    }
}
