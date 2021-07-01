// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::process::Command;

use failure::bail;
use itertools::Itertools;

use crate::configuration::RpcPerformanceTestEnv;
use crate::types::{Branch, BranchType, WrkResult};

type WrkResultMap = HashMap<Branch, WrkResult>;

fn run_wrk(branch: &Branch, rpc: &str, duration: &u64) -> Result<WrkResult, failure::Error> {
    let url = format!("{}{}", branch.url, rpc);
    println!("URL: {}", url);

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

    let out = String::from_utf8(output.stdout)?;
    println!("output: ======\n{}", out);

    // get the last line of the output, which is a json object
    let json_out = out
        .lines()
        .last()
        .unwrap()
        .to_string();

    let ret: WrkResult = serde_json::from_str(&json_out).expect("JSON was not well-formated");
    Ok(ret)
}

pub(crate) fn test_rpc_performance(env: RpcPerformanceTestEnv) -> Result<(), failure::Error> {
    let RpcPerformanceTestEnv {
        tezedge_new_node,
        tezedge_old_node,
        ocaml_node,
        url_file,
        wrk_test_duration,
        max_latency_threshold,
        throughput_threshold,
        latency_no_fail,
        throughput_no_fail,
    } = env;

    for rpc in super::utils::get_urls(&url_file)? {
        let ocaml = Branch::new(0, ocaml_node.clone(), BranchType::Ocaml);
        let tezedge_new = Branch::new(1, tezedge_new_node.clone(), BranchType::Feature);
        let tezedge_old = tezedge_old_node
            .as_ref()
            .map(|b| Branch::new(2, b.clone(), BranchType::Stable));

        println!("Preheating services for rpc: {}", rpc);
        println!();
        if let Some(tezedge_old) = &tezedge_old {
            let _ = run_wrk(&tezedge_old, &rpc, &1)?;
        }
        let _ = run_wrk(&ocaml, &rpc, &1)?;
        let _ = run_wrk(&tezedge_new, &rpc, &1)?;

        println!("Running wrk for rpc: {}", rpc);
        println!();
        let mut outputs: WrkResultMap = HashMap::new();

        if let Some(tezedge_old) = &tezedge_old {
            std::thread::sleep(std::time::Duration::from_secs(1));

            outputs.insert(
                tezedge_old.clone(),
                run_wrk(&tezedge_old, &rpc, &wrk_test_duration)?,
            );
        }

        std::thread::sleep(std::time::Duration::from_secs(1));

        outputs.insert(ocaml.clone(), run_wrk(&ocaml, &rpc, &wrk_test_duration)?);

        std::thread::sleep(std::time::Duration::from_secs(1));

        outputs.insert(
            tezedge_new.clone(),
            run_wrk(&tezedge_new, &rpc, &wrk_test_duration)?,
        );

        calculate_and_display_statistics(&outputs, max_latency_threshold, throughput_threshold, latency_no_fail, throughput_no_fail);
    }

    Ok(())
}

fn calculate_and_display_statistics(wrk_results: &WrkResultMap, max_latency_threshold: f32, throughput_threshold: f32, latency_no_fail: bool, throughput_no_fail: bool) {
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
    calc_deltas(wrk_results, max_latency_threshold, throughput_threshold, latency_no_fail, throughput_no_fail);
    println!("------------------------------------------------------");
    println!();
}

fn calc_throughput(req: &f32, dur: &f32) -> f32 {
    req / (dur * 0.000001)
}

fn calc_max_latency(lat: &f32) -> f32 {
    lat * 0.001
}

fn calc_deltas(wrk_results: &WrkResultMap, max_latency_threshold: f32, throughput_threshold: f32, latency_no_fail: bool, throughput_no_fail: bool) {
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

        // TODO: remove this after the high variance of the max_latency has been reduced
        const MINIMAL_STABLE_LATENCY_TO_CHECK: f32 = 50.0;
        if new.latency_max() > stable.latency_max()
            && stable.latency_max() > &MINIMAL_STABLE_LATENCY_TO_CHECK
        {
            // fail the test if the 10% performance happened
            if stable.latency_max() * max_latency_threshold
                < new.latency_max() - stable.latency_max()
            {
                if latency_no_fail {
                    println!(
                        "[Max Latency] Perforamnce regression greater than {}%!",
                        max_latency_threshold * 100.0
                    )
                } else {
                    panic!(
                        "[Max Latency] Perforamnce regression greater than {}%!",
                        max_latency_threshold * 100.0
                    )
                }
            }
        }

        if new.requests() < stable.requests() {
            // fail the test if the 10% performance happened
            if stable.requests() * throughput_threshold < stable.requests() - new.requests() {
                if throughput_no_fail {
                    println!(
                        "[Troughput] Performance regression greater than {}%!",
                        throughput_threshold * 100.0
                    )
                } else {
                    panic!(
                        "[Troughput] Performance regression greater than {}%!",
                        throughput_threshold * 100.0
                    )
                }
            }
        }
    }
}
