use candid::{CandidType, Deserialize};

#[derive(CandidType, Deserialize, Debug)]
pub enum BasicResponse {
    Success(String),
    Unauthorized,
    Forbidden,
    Conflict(String)
}