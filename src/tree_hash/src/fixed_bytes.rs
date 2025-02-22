use std::convert::TryFrom;
use std::panic;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;
use derive_more::Index;

#[derive(Debug, Copy, Index, Clone, PartialEq)]
pub struct FixedBytes<const N: usize>( pub [u8; N]);

impl<const N: usize> Default for FixedBytes<N> {
    fn default() -> Self {
        Self([0u8; N])
    }
}

pub type B256 = FixedBytes<32>;


impl<'de, const N: usize> Deserialize<'de> for FixedBytes<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let val: String = serde::Deserialize::deserialize(deserializer)?;
        let h = hex::decode(val.trim_start_matches("0x")).map_err(D::Error::custom)?;
        if h.len() != N {
            return Err("length error".to_string()).map_err(D::Error::custom);
        }
        let mut v: [u8;N] = [0u8;N];
        v.copy_from_slice(h.as_slice());
        Ok(FixedBytes(v))
    }
}


impl<const N: usize> Serialize for FixedBytes<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let x = format!("0x{}", hex::encode(self.0.as_slice()));
        serializer.serialize_str(&x)
    }
}

impl<const N: usize> FixedBytes<N> {
    pub const ZERO: Self = Self([0u8;N]);

    #[inline]
    pub fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }

    #[inline]
    pub const fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn from_slice(src: &[u8]) -> Self {
        match Self::try_from(src) {
            Ok(x) => x,
            Err(_) => {
                panic!("xxx");
            }
        }
    }
}


impl<const N: usize> TryFrom<&[u8]> for FixedBytes<N> {
    type Error = core::array::TryFromSliceError;

    #[inline]
    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        <&Self>::try_from(slice).copied()
    }
}

/// Tries to create a `FixedBytes<N>` by copying from a mutable slice `&mut
/// [u8]`. Succeeds if `slice.len() == N`.
impl<const N: usize> TryFrom<&mut [u8]> for FixedBytes<N> {
    type Error = core::array::TryFromSliceError;

    #[inline]
    fn try_from(slice: &mut [u8]) -> Result<Self, Self::Error> {
        Self::try_from(&*slice)
    }
}

/// Tries to create a ref `FixedBytes<N>` by copying from a slice `&[u8]`.
/// Succeeds if `slice.len() == N`.
impl<'a, const N: usize> TryFrom<&'a [u8]> for &'a FixedBytes<N> {
    type Error = core::array::TryFromSliceError;

    #[inline]
    fn try_from(slice: &'a [u8]) -> Result<&'a FixedBytes<N>, Self::Error> {
        // SAFETY: `FixedBytes<N>` is `repr(transparent)` for `[u8; N]`
        <&[u8; N]>::try_from(slice).map(|array_ref| unsafe { core::mem::transmute(array_ref) })
    }
}

/// Tries to create a ref `FixedBytes<N>` by copying from a mutable slice `&mut
/// [u8]`. Succeeds if `slice.len() == N`.
impl<'a, const N: usize> TryFrom<&'a mut [u8]> for &'a mut FixedBytes<N> {
    type Error = core::array::TryFromSliceError;

    #[inline]
    fn try_from(slice: &'a mut [u8]) -> Result<&'a mut FixedBytes<N>, Self::Error> {
        // SAFETY: `FixedBytes<N>` is `repr(transparent)` for `[u8; N]`
        <&mut [u8; N]>::try_from(slice).map(|array_ref| unsafe { core::mem::transmute(array_ref) })
    }
}
