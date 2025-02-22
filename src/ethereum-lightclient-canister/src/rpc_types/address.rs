use serde::{Deserialize, Serialize};
use tree_hash::fixed_bytes::FixedBytes;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address(pub FixedBytes<20>);