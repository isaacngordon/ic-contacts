use candid::{CandidType, Deserialize};

#[derive(CandidType, Deserialize)]
pub struct NewUser {
    pub username: String,
}
