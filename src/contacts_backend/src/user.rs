use candid::CandidType;
use serde::{Deserialize, Serialize};
use super::contact::Contact;

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct User {
    pub username: String,
    pub contacts: Vec<Contact>,
    pub shared_contacts: Vec<u64>, // Contact IDs
}
