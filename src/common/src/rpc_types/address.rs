use serde::{Deserialize, Serialize};
use tree_hash::fixed_bytes::FixedBytes;
use tree_hash::{Hash256, PackedEncoding, TreeHash, TreeHashType};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address(pub FixedBytes<20>);


impl TreeHash for Address {
    fn tree_hash_type() -> TreeHashType {
        TreeHashType::Vector
    }

    fn tree_hash_packed_encoding(&self) -> PackedEncoding {
        let mut result = [0; 32];
        result[0..20].copy_from_slice(self.0.as_slice());
        PackedEncoding::from_slice(&result)
    }

    fn tree_hash_packing_factor() -> usize {
        1
    }

    fn tree_hash_root(&self) -> Hash256 {
        let mut result = [0; 32];
        result[0..20].copy_from_slice(self.0.as_slice());
        Hash256::from_slice(&result)
    }
}