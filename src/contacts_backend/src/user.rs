
use candid::{CandidType, Decode, Deserialize, Encode};
use candid::{CandidType, Decode, Deserialize, Encode, Serialize};
use ic_stable_structures::{
    storable::Bound, Storable,
};
use std::borrow::Cow;
use super::contact::Contact;

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct User {
    pub username: String,
    pub contacts: Vec<Contact>,
    pub shared_contacts: Vec<u64>, // Contact IDs
}

impl Storable for User {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}