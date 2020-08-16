use crate::persistence::{Persistence, PersistenceError, PersistenceResult};
use log::debug;
use rusqlite::types::{FromSql, FromSqlError, ValueRef};
use rusqlite::{params, Connection, NO_PARAMS};
use serde::ser::Serializer;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

pub struct InMemoryPersistence {
    conn: Connection,
}
impl Default for InMemoryPersistence {
    fn default() -> InMemoryPersistence {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            r#"
            CREATE TABLE person (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )
        "#,
            NO_PARAMS,
        )
        .unwrap();
        conn.execute(
            r#"
            INSERT INTO person (name) VALUES (?1), (?2), (?3)
        "#,
            params!["Alice", "Bob", "Carol"],
        )
        .unwrap();
        InMemoryPersistence { conn }
    }
}

impl Persistence for InMemoryPersistence {
    fn query_named(&self, name: String) -> PersistenceResult<Value> {
        debug!("running named query: {}", name);
        Err(PersistenceError::Unknown("unimplemented".to_owned()))
    }
    fn mutate_named(&self, name: String) -> PersistenceResult<()> {
        debug!("performing named mutation: {}", name);
        Err(PersistenceError::Unknown("unimplemented".to_owned()))
    }

    fn query_raw(&self, query: String) -> PersistenceResult<Value> {
        debug!("running query {}", query);
        let mut stmt = self.conn.prepare(&query)?;
        let rows: Vec<BTreeMap<String, MyValue>> = stmt
            .query_map(NO_PARAMS, |row| {
                let values: BTreeMap<String, MyValue> = (0..row.column_count())
                    .map(|i| (row.column_name(i).unwrap().to_owned(), row.get_unwrap(i)))
                    .collect();
                Ok(values)
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(serde_json::to_value(&rows).unwrap())
    }
    fn mutate_raw(&self, stmt: String) -> PersistenceResult<()> {
        debug!("running mutation {}", stmt);
        self.conn.execute(&stmt, NO_PARAMS)?;
        Ok(())
    }
}

// TODO(rpb): try to optimize this so that it's serialized directly from the ValueRef.
// Right now we're cloning the data just so that we can serialize it after the query completes.
enum MyValue {
    Null,
    Integer(i64),
    Float(f64),
    Text(String),
    Bytes(Vec<u8>),
}
impl Serialize for MyValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            MyValue::Null => serializer.serialize_none(),
            MyValue::Integer(i) => serializer.serialize_i64(*i),
            MyValue::Float(i) => serializer.serialize_f64(*i),
            MyValue::Text(i) => serializer.serialize_str(i),
            MyValue::Bytes(i) => serializer.serialize_bytes(i),
        }
    }
}

impl FromSql for MyValue {
    fn column_result(value: ValueRef) -> Result<MyValue, FromSqlError> {
        Ok(match value {
            ValueRef::Null => MyValue::Null,
            ValueRef::Integer(i) => MyValue::Integer(i),
            ValueRef::Real(i) => MyValue::Float(i),
            ValueRef::Text(i) => MyValue::Text(String::from_utf8(i.to_vec()).unwrap()),
            ValueRef::Blob(i) => MyValue::Bytes(i.to_vec()),
        })
    }
}
