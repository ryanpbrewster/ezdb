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
    fn get(&self, path: String) -> PersistenceResult<Value>;
    fn put(&mut self, path: String, value: Value) -> PersistenceResult<()>;
}

pub mod inmemory;
