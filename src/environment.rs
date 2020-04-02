use std::env;

pub fn to_block_header() -> i32 {
    env::var("TO_BLOCK_HEADER")
        .unwrap_or_else(|_| panic!("TO_BLOCK_HEADER env variable is missing, check rpc/README.md"))
        .parse()
        .unwrap_or_else(|_| panic!("TO_BLOCK_HEADER env variable can not be parsed as a number, check rpc/README.md"))
}

pub fn ocaml_node_rpc_context_root() -> String {
    env::var("OCAML_NODE_RPC_CONTEXT_ROOT")
        .unwrap_or("http://ocaml-node-run:8732".to_string())
}

pub fn tezedge_node_rpc_context_root() -> String {
    env::var("TEZEDGE_NODE_RPC_CONTEXT_ROOT")
        .unwrap_or("http://tezedge-node-run:18732".to_string())
}

pub fn tezedge_node_master_rpc_context_root() -> String {
    env::var("TEZEDGE_NODE_MASTER_RPC_CONTEXT_ROOT")
        .unwrap_or("http://tezedge-master-node-run:28732".to_string())
}

pub fn ocaml_node_indexer_root() -> String {
    env::var("OCAML_NODE_INDEXER_ROOT")
        .unwrap_or("http://tz-indexer-ocaml:8002".to_string())
} 

pub fn tezedge_node_indexer_root() -> String {
    env::var("TEZEDGE_NODE_INDEXER_ROOT")
        .unwrap_or("http://tz-indexer-ocaml:8002".to_string())
}
