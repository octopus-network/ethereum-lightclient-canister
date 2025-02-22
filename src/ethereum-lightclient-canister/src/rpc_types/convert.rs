use std::cmp::PartialEq;
use tree_hash::fixed_bytes::B256;
use crate::rpc_types::lightclient_header::{Beacon, ExecutionPayloadHeader, LightClientHeader};


pub fn default_header_to_none(value: LightClientHeader) -> Option<LightClientHeader> {
    if value.beacon == Beacon::default() && value.execution == ExecutionPayloadHeader::default() {
         None
    }else {
        Some(value)
    }
}

pub fn default_branch_to_none(value: &[B256]) -> Option<Vec<B256>> {
    for elem in value {
        if !elem.is_zero() {
            return Some(value.to_vec())
        }
    }
    None
}

fn default_to_none<T: Default + PartialEq>(value: T) -> Option<T> {
    if value == T::default() {
        None
    } else {
        Some(value)
    }
}

pub fn hex_to_u64(str: &str) -> u64 {
    u64::from_str_radix(str.trim_start_matches("0x"),16).unwrap()
}
