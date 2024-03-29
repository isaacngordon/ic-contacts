use candid::CandidType;
use serde::{Deserialize, Serialize, Encode, Decode};
use ic_stable_structures::{
    storable::Bound, Storable,
};
use std::borrow::Cow;

#[derive(CandidType, Deserialize, Serialize, Debug, Clone)]
pub struct Contact {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub phone: String,
}

impl Storable for Contact {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}
