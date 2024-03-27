use candid::CandidType;
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
pub struct NewUser {
    pub username: String,
}
