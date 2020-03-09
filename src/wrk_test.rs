use failure::bail;
use std::env;
use std::process::Command;

use crate::types::{Branch, WrkResult};


fn run_wrk(branch: Branch, rpc: &str) -> Result<WrkResult, failure::Error> {
    let output;

    let master_url = format!("{}/{}", "http://tezedge-node-run:18732", rpc);
    let modified_url = format!("{}/{}", "http://tezedge-master-node-run:18732", rpc);

    // local testing
    // let master_url = format!("{}/{}", "http://116.202.128.230:18732", rpc);
    // let modified_url = format!("{}/{}", "http://116.202.128.230:18732", rpc);

    let mut wrk_args = vec![
        "-t1", 
        "-c1",
        "-d30s",
        "-R1000",
        "-s",
        "/scripts/as_json.lua",
        "--",
        "out.json"
    ];
    match branch {
        Branch::Master => {
            // output = Command::new("wrk").args(&)
            // .current_dir("/")
            // .output()?;
            wrk_args.insert(4, &master_url);

            output = Command::new("wrk").args(&wrk_args)
            .current_dir("/")
            .output()?;
        }
        Branch::Modified => {
            // output = Command::new("wrk").args(&)
            // .current_dir("/")
            // .output()?;
            wrk_args.insert(4, &modified_url);

            output = Command::new("wrk").args(&wrk_args)
            .current_dir("/")
            .output()?;
        }
    }

    println!("Running wrk with arguments: {:?}", &wrk_args);
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

pub(crate) fn test_rpc_performance() -> Result<(), failure::Error> {
    let rpcs = vec![
        "chains/main/blocks/100/helpers/baking_rights",
        "chains/main/blocks/100/helpers/endorsing_rights",
        "chains/main/blocks/100/context/constants",
        // "chains/main/blocks/100/votes/listings",
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

        // calculate req/s with total number of requests(N) and the duration of the test (us) 
        let master_throughput = output_master.requests() / (output_master.duration() * 0.000001);
        let modified_throughput = output_modified.requests() / (output_modified.duration() * 0.000001);

        println!("Master throughput: {}req/s", master_throughput); 
        println!("Modified throughput: {}req/s", modified_throughput); 

        println!("Master max latency: {}us", output_master.latency_max());
        println!("Modified max latency: {}us", output_modified.latency_max());

        let tolerance = output_master.latency_max() * 0.1;
        println!("Tolerance (10%): {}us", tolerance);

        let delta = output_modified.latency_max() - output_master.latency_max();
        println!("Delta: {}us", delta);

        // TODO: fail the test if the d
        if delta < tolerance {
            println!("OK");
        } else {
            println!("FAIL");
        }
        println!("------------------------------------------------------");
        println!("");
    }

    Ok(())
}