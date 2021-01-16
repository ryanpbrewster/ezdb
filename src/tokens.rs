use serde::{Deserialize, Deserializer};
use std::{fmt, str::FromStr};

#[derive(PartialEq, Eq, Hash)]
pub struct ProjectId(String);
impl FromStr for ProjectId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if is_valid_token(s) {
            Ok(ProjectId(s.to_owned()))
        } else {
            Err(format!("invalid project id: {}", s))
        }
    }
}
impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl<'de> Deserialize<'de> for ProjectId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct DatabaseId(String);
impl FromStr for DatabaseId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if is_valid_token(s) {
            Ok(DatabaseId(s.to_owned()))
        } else {
            Err(format!("invalid database id: {}", s))
        }
    }
}
impl fmt::Display for DatabaseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl<'de> Deserialize<'de> for DatabaseId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
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

#[derive(Hash, PartialEq, Eq)]
pub struct DatabaseAddress {
    pub project_id: ProjectId,
    pub database_id: DatabaseId,
}
impl DatabaseAddress {
    pub fn filename(&self) -> String {
        format!("{}-{}.sqlite", self.project_id, self.database_id)
    }
}
