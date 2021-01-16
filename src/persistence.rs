use crate::core::Policy;
use rusqlite::InterruptHandle;
use serde_json::Value;
use std::collections::BTreeMap;

pub type PersistenceResult<T> = ::std::result::Result<T, PersistenceError>;

#[derive(Debug, Eq, PartialEq)]
pub enum PersistenceError {
    Unknown(String),
    NoSuchQuery(String),
    Busy,
    Interrupted,
}

impl From<rusqlite::Error> for PersistenceError {
    fn from(err: rusqlite::Error) -> PersistenceError {
        if let rusqlite::Error::SqliteFailure(e, _) = err {
            if e.code == rusqlite::ErrorCode::OperationInterrupted {
                return PersistenceError::Interrupted;
            }
        }
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
    fn get_interrupt_handle(&self) -> InterruptHandle;
}

mod sqlite;

pub use sqlite::SqliteFactory;
pub use sqlite::SqlitePersistence;
