use std::borrow::Cow;
use candid::{CandidType, Deserialize};
use ic_stable_structures::Storable;
use ic_stable_structures::storable::Bound;
use serde::Serialize;

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct HeaderInfo {
    pub receipt_root: String,
}

impl Storable for HeaderInfo {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let pts = ciborium::de::from_reader(bytes.as_ref())
            .expect("failed to decode pending ticket status");
        pts
    }
    const BOUND: Bound = Bound::Unbounded;
}