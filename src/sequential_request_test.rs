use std::time::{Instant, Duration};

use crate::types::NodeType;


pub(crate) fn test_sequential_requests(
    number_of_cycles: i32,
    nodes: &Vec<NodeType>) {

    for node in nodes {
        let start = Instant::now();

        for cycle in 1..number_of_cycles {
            // get the first 3 cycles form the first block
            let url = if cycle < 4 {
                format!("{}/chains/main/blocks/1/helpers/baking_rights?cycle={}&all=true", node.url, cycle)
            } else {
                // allways get the rights from the first block possible
                let block_level = (cycle * 2048) - (3 * 2048) + 1;
                format!("{}/chains/main/blocks/{}/helpers/baking_rights?cycle={}&all=true", node.url, block_level, cycle)
            };
            let req_start = Instant::now();
            let response = reqwest::blocking::get(&url).unwrap();
            let req_elapsed = req_start.elapsed();

            if !response.status().is_success() {
                panic!("Request {} failed!", url)
            }
            println!("[{}] Requested {} -> {}s", node.name, url, extract_secs(req_elapsed));
        }
        let elapsed = start.elapsed();
        println!("[{}] Duration in seconds: {}s", node.name, extract_secs(elapsed));
        println!("--------------------------------------------------------------------")
    }
}

fn extract_secs(dur: Duration) -> f64 {
    (dur.as_secs() as f64) + (dur.subsec_nanos() as f64 / 1000_000_000.0)
}