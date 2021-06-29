use std::fmt;

use getset::Getters;
use serde::Deserialize;
use url::Url;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum BranchType {
    Stable,
    Feature,
    Ocaml,
}

impl fmt::Display for BranchType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Branch {
    pub sort_key: usize,
    pub url: Url,
    pub branch_type: BranchType,
}

impl Branch {
    pub fn new(sort_key: usize, url: Url, branch_type: BranchType) -> Self {
        Self {
            sort_key,
            url,
            branch_type,
        }
    }
}

impl fmt::Display for Branch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Deserialize, Debug, Getters, Clone)]
pub struct WrkResult {
    #[get = "pub(crate)"]
    duration: f32,

    #[get = "pub(crate)"]
    requests: f32,

    #[get = "pub(crate)"]
    latency_max: f32,

    #[get = "pub(crate)"]
    latency_min: f32,

    #[get = "pub(crate)"]
    latency_mean: f32,

    #[get = "pub(crate)"]
    latency_stdev: f32,
}
