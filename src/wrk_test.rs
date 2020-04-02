use failure::bail;
use std::env;
use std::process::Command;
use std::collections::HashMap;

use crate::types::{Branch, WrkResult};

type WrkResultMap = HashMap<Branch, WrkResult>;

fn run_wrk(branch: Branch, rpc: &str) -> Result<WrkResult, failure::Error> {
    let output;

    let modified_url = format!("{}/{}", "http://tezedge-node-run:18732", rpc);
    let master_url = format!("{}/{}", "http://tezedge-master-node-run:28732", rpc);
    let ocaml_url = format!("{}/{}", "http://ocaml-node-run:8732", rpc);


    // local testing
    // let master_url = format!("{}/{}", "http://116.202.128.230:28732", rpc);
    // let modified_url = format!("{}/{}", "http://116.202.128.230:18732", rpc);
    // let ocaml_url = format!("{}/{}", "http://116.202.128.230:10000", rpc);

    let mut wrk_args = vec![
        "-t1", 
        "-c1",
        "-d30s",
        //"-R1000",
        "--timeout",
        "30s",
        "-s",
        "/scripts/as_json.lua",
        "--",
        "out.json"
    ];
    println!("Running wrk for {:?} with arguments: {:?}", branch, &wrk_args);

    match branch {
        Branch::Master => {
            wrk_args.insert(5, &master_url);

            output = Command::new("wrk").args(&wrk_args)
            .current_dir("/")
            .output()?;
        }
        Branch::Modified => {
            wrk_args.insert(5, &modified_url);

            output = Command::new("wrk").args(&wrk_args)
            .current_dir("/")
            .output()?;
        }
        Branch::Ocaml => {
            wrk_args.insert(5, &ocaml_url);

            output = Command::new("wrk").args(&wrk_args)
            .current_dir("/")
            .output()?;
        }
    }

    
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
        "chains/main/blocks/100/helpers/baking_rights?all&cycle=1",
        "chains/main/blocks/100/helpers/endorsing_rights?all&cycle=1",
        "chains/main/blocks/100/context/constants",
        "chains/main/blocks/100/votes/listings",
        "chains/main/blocks/100",
        "chains/main/blocks/100/header",
        "chains/main/blocks/100/context/raw/bytes/cycle",
        //"chains/main/blocks/100/context/delegates/tz1PirboZKFVqkfE45hVLpkpXaZtLk3mqC17",
    ];

    let path = env::current_dir()?;
    println!("The current directory is {}", path.display());

    for rpc in rpcs.into_iter() {
        println!("Running wrk for rpc: {}", rpc);
        let mut outputs: WrkResultMap = HashMap::new();

        outputs.insert(Branch::Master, run_wrk(Branch::Master, &rpc)?);
        outputs.insert(Branch::Modified, run_wrk(Branch::Modified, &rpc)?);
        outputs.insert(Branch::Ocaml, run_wrk(Branch::Ocaml, &rpc)?);

        calculate_and_display_statistics(&outputs)?;
    }

    Ok(())
}

fn calculate_and_display_statistics(wrk_results: &WrkResultMap) -> Result<(), failure::Error>{

    for (res_key, res_val) in wrk_results {
         println!("{:?} thoughtput: {}req/s", res_key, calc_throughput(res_val.requests(), res_val.duration())?);
         println!("{:?} max latency: {}ms", res_key, calc_max_latency(res_val.latency_max())?);
         println!("");
    }
    calc_deltas(wrk_results)?;
    println!("------------------------------------------------------");
    println!("");
    Ok(())
}

fn calc_throughput(req: &f32, dur: &f32) -> Result<f32, failure::Error> {
    Ok(req / (dur * 0.000001))
}

fn calc_max_latency(lat: &f32) -> Result<f32, failure::Error> {
    Ok(lat * 0.001)
}

fn calc_deltas(wrk_results: &WrkResultMap) -> Result<(), failure::Error> {
    let ocaml_max_latency = wrk_results.get(&Branch::Ocaml).unwrap().latency_max();
    let master_max_latency = wrk_results.get(&Branch::Master).unwrap().latency_max();
    let modified_max_latency = wrk_results.get(&Branch::Modified).unwrap().latency_max();

    let delta_master = (master_max_latency - ocaml_max_latency) * 0.001;
    let delta_modified = (modified_max_latency - ocaml_max_latency) * 0.001;

    println!("Deltas compared to ocaml node: ");
    println!("\t Master: {}ms", delta_master);
    println!("\t Modified: {}ms", delta_modified);

    Ok(())
}
