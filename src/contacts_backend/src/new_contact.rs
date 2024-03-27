use candid::CandidType;
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
pub struct NewContact {
    pub name: String,
    pub email: String,
    pub phone: String,
}
