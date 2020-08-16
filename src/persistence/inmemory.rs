use crate::persistence::{Persistence, PersistenceError, PersistenceResult};
use log::debug;
use rusqlite::{params, Connection, NO_PARAMS};
use serde::Serialize;
use serde_json::Value;

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

#[derive(Serialize)]
struct Person {
    id: i32,
    name: String,
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
        let rows: Vec<Person> = stmt
            .query_map(NO_PARAMS, |row| {
                Ok(Person {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
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
