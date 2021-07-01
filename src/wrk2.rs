// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::process::Command;

use failure::bail;
use crate::configuration::RpcLatencyTestEnv;
use crate::types::{Branch, BranchType};

type WrkResultMap = HashMap<Branch, ()>;

fn run_wrk(branch: &Branch, rpc: &str, duration: &u64, rate: u64) -> Result<(), failure::Error> {
    let url = format!("{}{}", branch.url, rpc);
    println!("URL: {}", url);

    // local testing
    // let master_url = format!("{}/{}", "http://116.202.128.230:28732", rpc);
    // let modified_url = format!("{}/{}", "http://116.202.128.230:18732", rpc);
    // let ocaml_url = format!("{}/{}", "http://116.202.128.230:10000", rpc);
    let duration_string = &format!("-d{}s", duration.clone());
    let rate_string = &format!("-R{}", rate);

    let mut wrk_args = vec![
        "-t1",
        "-c1",
        duration_string,
        //"-R1000",
        "--timeout",
        "30s",
        rate_string,
        "--latency",
    ];
    println!(
        "Running wrk for {:?} with arguments: {:?}",
        branch.url.domain().unwrap_or(""),
        &wrk_args
    );
    println!();

    wrk_args.push(&url);

    let output = Command::new("wrk2")
        .args(&wrk_args)
        .current_dir("/")
        .output()?;

    if !output.status.success() {
        println!("{:?}", output);
        bail!("Command executed with failing error code");
    }

    let output = String::from_utf8(output.stdout)?;
    println!("=== Output ===\n{}===========", output);

    Ok(())
}

pub(crate) fn test_rpc_performance(env: RpcLatencyTestEnv) -> Result<(), failure::Error> {
    let RpcLatencyTestEnv {
        tezedge_new_node,
        tezedge_old_node,
        ocaml_node,
        url_file,
        wrk_test_duration,
        wrk_request_rate,
    } = env;

    for rpc in super::utils::get_urls(&url_file)? {
        let ocaml = Branch::new(0, ocaml_node.clone(), BranchType::Ocaml);
        let tezedge_new = Branch::new(1, tezedge_new_node.clone(), BranchType::Feature);
        let tezedge_old = tezedge_old_node.as_ref().map(|b| Branch::new(2, b.clone(), BranchType::Stable));

        println!("Running wrk for rpc: {}", rpc);
        println!();
        let mut outputs: WrkResultMap = HashMap::new();


        if let Some(tezedge_old) = &tezedge_old {
            std::thread::sleep(std::time::Duration::from_secs(1));

            outputs.insert(
                tezedge_old.clone(),
                run_wrk(&tezedge_old, &rpc, &wrk_test_duration, wrk_request_rate)?,
            );
        }

        std::thread::sleep(std::time::Duration::from_secs(1));

        outputs.insert(ocaml.clone(), run_wrk(&ocaml, &rpc, &wrk_test_duration, wrk_request_rate)?);

        std::thread::sleep(std::time::Duration::from_secs(1));

        outputs.insert(
            tezedge_new.clone(),
            run_wrk(&tezedge_new, &rpc, &wrk_test_duration, wrk_request_rate)?,
        );
    }

    Ok(())
}
