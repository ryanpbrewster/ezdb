use crate::persistence::{PersistenceError, PersistenceResult};
use serde::Deserialize;

#[derive(PartialEq, Eq, Deserialize)]
pub struct ProjectId(String);
impl ProjectId {
    pub fn new(raw: String) -> PersistenceResult<ProjectId> {
        if is_valid_token(&raw) {
            Ok(ProjectId(raw))
        } else {
            Err(PersistenceError::Unknown(format!(
                "invalid project id: {}",
                raw
            )))
        }
    }
}

#[derive(PartialEq, Eq, Deserialize)]
pub struct DatabaseId(String);
impl DatabaseId {
    pub fn new(raw: String) -> PersistenceResult<DatabaseId> {
        if is_valid_token(&raw) {
            Ok(DatabaseId(raw))
        } else {
            Err(PersistenceError::Unknown(format!(
                "invalid database id: {}",
                raw
            )))
        }
    }
}

const MAX_TOKEN_SIZE: usize = 32;
// Tokens are ASCII strings with length 1..32 inclusive, where the first character
// is alphabetic and the rest of the characters are alphanumeric.
// I think the regex is approximately /[A-Za-z][A-Za-z0-9]{0,31}/
fn is_valid_token(raw: &str) -> bool {
    !raw.is_empty()
        && raw.len() <= MAX_TOKEN_SIZE
        && raw
            .bytes()
            .enumerate()
            .all(|(idx, b)| b.is_ascii_alphabetic() || (idx > 0 && b.is_ascii_digit()))
}
