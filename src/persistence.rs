use serde_json::Value;

pub type PersistenceResult<T> = ::std::result::Result<T, PersistenceError>;

#[derive(Debug)]
pub enum PersistenceError {
    Unknown(String),
}

impl From<rusqlite::Error> for PersistenceError {
    fn from(err: rusqlite::Error) -> PersistenceError {
        PersistenceError::Unknown(format!("{:?}", err))
    }
}

pub trait Persistence {
    fn query_named(&self, name: String) -> PersistenceResult<Value>;
    fn mutate_named(&self, name: String) -> PersistenceResult<()>;
    fn query_raw(&self, query: String) -> PersistenceResult<Value>;
    fn mutate_raw(&self, stmt: String) -> PersistenceResult<()>;
}

mod sqlite;

pub use sqlite::SqlitePersistence;
