// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

use std::fs::File;
use std::io::{BufRead, BufReader};

fn lines(read: impl std::io::Read) -> std::io::Result<Vec<String>> {
    BufReader::new(read).lines().filter_map(|r| {
        match r {
            Err(_) => Some(r),
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() || line.starts_with("#") {
                    None
                } else {
                    Some(Ok(line.trim_start_matches("/").to_string()))
                }
            }
        }
    }).collect::<Result<_, _>>()
}

pub(crate) fn get_urls(file: &str) -> Result<Vec<String>, failure::Error> {
    if file == "-" {
        Ok(lines(std::io::stdin())?)
    } else {
        Ok(lines(File::open(file)?)?)
    }
}
