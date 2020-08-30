use crate::core::Policy;
use serde_json::Value;
use std::collections::BTreeMap;

pub type PersistenceResult<T> = ::std::result::Result<T, PersistenceError>;

#[derive(Debug)]
pub enum PersistenceError {
    Unknown(String),
    NoSuchQuery(String),
}

impl From<rusqlite::Error> for PersistenceError {
    fn from(err: rusqlite::Error) -> PersistenceError {
        PersistenceError::Unknown(format!("{:?}", err))
    }
}

impl From<actix::MailboxError> for PersistenceError {
    fn from(err: actix::MailboxError) -> PersistenceError {
        PersistenceError::Unknown(format!("{:?}", err))
    }
}

pub trait Persistence {
    fn query_named(
        &self,
        name: String,
        params: BTreeMap<String, Value>,
    ) -> PersistenceResult<Value>;
    fn mutate_named(&self, name: String, params: BTreeMap<String, Value>) -> PersistenceResult<()>;
    fn query_raw(&self, query: String) -> PersistenceResult<Value>;
    fn mutate_raw(&self, stmt: String) -> PersistenceResult<()>;
    fn fetch_policy(&self) -> PersistenceResult<Policy>;
    fn set_policy(&self, policy: Policy) -> PersistenceResult<()>;
}

mod sqlite;

pub use sqlite::SqliteFactory;
pub use sqlite::SqlitePersistence;
