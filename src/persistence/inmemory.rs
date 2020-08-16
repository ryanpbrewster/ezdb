use crate::persistence::{Persistence, PersistenceResult};
use log::debug;
use rusqlite::{params, Connection, NO_PARAMS};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

pub struct InMemoryPersistence {
    full_data: BTreeMap<String, Value>,
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
        InMemoryPersistence {
            full_data: BTreeMap::new(),
            conn,
        }
    }
}

#[derive(Serialize)]
struct Person {
    id: i32,
    name: String,
}

impl Persistence for InMemoryPersistence {
    fn get(&self, path: String) -> PersistenceResult<Value> {
        debug!("running query {}", path);
        let mut stmt = self.conn.prepare("SELECT id, name FROM person")?;
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

    fn put(&mut self, path: String, value: Value) -> PersistenceResult<()> {
        debug!("writing @ {:?} == {}", path, value);
        self.full_data.insert(path, value);
        Ok(())
    }
}
