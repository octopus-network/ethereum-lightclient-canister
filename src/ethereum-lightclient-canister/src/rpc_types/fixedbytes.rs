use core::fmt;
use candid::Deserialize;
use serde::{Deserializer, Serialize};

/*#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FixedBytes<const N: usize>( pub [u8; N]);

impl<const N: usize> Default for FixedBytes<N> {
    fn default() -> Self {
        Self([0u8; N])
    }
}


impl<const N: usize> Deserialize for FixedBytes<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        todo!()
    }

    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error> where D: Deserializer<'de> {
        todo!()
    }
}*/