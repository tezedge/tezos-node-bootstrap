use std::fmt;

use getset::Getters;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct NodeType {
    pub name: String,
    pub url: String,
    pub branch_type: BranchType,
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

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
    pub name: String,
    pub branch_type: BranchType,
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