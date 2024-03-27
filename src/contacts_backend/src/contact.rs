use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(CandidType, Deserialize, Serialize, Debug, Clone)]
pub struct Contact {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub phone: String,
}
