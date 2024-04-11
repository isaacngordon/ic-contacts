use candid::{CandidType, Deserialize, Encode, Decode};
use ic_stable_structures::{
    storable::Bound, Storable,
};
use std::borrow::Cow;
use crate::data::counter::Counter;

pub type ContactID = u64;

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq)]
pub struct Contact {
    id: Option<ContactID>,
    pub name: String,
    pub email: String,
    pub phone: String,
}

impl Contact {
    pub fn new(name: String, email: String, phone: String,  contact_id_counter: Option<&mut Counter>) -> Self {
        match contact_id_counter {
            Some(counter) => {
                let id = Some(counter.increment());
                Self { id, name, email, phone}
            },
            None => Self { id: None, name, email, phone}
        }
    }
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
