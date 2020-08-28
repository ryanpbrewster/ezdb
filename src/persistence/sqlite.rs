use crate::core::{MutationPolicy, Policy, QueryPolicy};
use crate::persistence::{Persistence, PersistenceResult};
use log::debug;
use rusqlite::types::{FromSql, FromSqlError, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{Connection, Transaction, NO_PARAMS};
use serde::ser::Serializer;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

pub struct SqlitePersistence {
    conn: Connection,
}
impl SqlitePersistence {
    pub fn in_memory() -> PersistenceResult<SqlitePersistence> {
        let conn = Connection::open_in_memory().unwrap();
        initialize_metadata(&conn)?;
        Ok(SqlitePersistence { conn })
    }
    pub fn from_file(path: &Path) -> PersistenceResult<SqlitePersistence> {
        let conn = Connection::open(path)?;
        initialize_metadata(&conn)?;
        Ok(SqlitePersistence { conn })
    }
}
fn initialize_metadata(conn: &Connection) -> PersistenceResult<()> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS __ezdb_metadata__ (
            type TEXT NOT NULL,
            name TEXT NOT NULL,
            raw_sql TEXT NOT NULL,
            PRIMARY KEY (type, name)
        )
    "#,
        NO_PARAMS,
    )?;
    Ok(())
}

impl Persistence for SqlitePersistence {
    fn query_named(
        &self,
        name: String,
        params: BTreeMap<String, Value>,
    ) -> PersistenceResult<Value> {
        debug!("running named query: {}", name);
        let txn = self.conn.unchecked_transaction()?;
        let query: String = txn.query_row(
            "SELECT raw_sql FROM __ezdb_metadata__ WHERE type = 'query' AND name = ?",
            &[&name],
            |row| row.get(0),
        )?;
        let params: Vec<(String, MyValue)> =
            params.into_iter().map(|(k, v)| (k, v.into())).collect();
        let params: Vec<(&str, &dyn ToSql)> = params
            .iter()
            .map(|(k, v)| (k.as_ref(), v as &dyn ToSql))
            .collect();
        let mut stmt = self.conn.prepare(&query)?;
        let rows: Vec<BTreeMap<String, MyValue>> = stmt
            .query_map_named(params.as_slice(), |row| {
                let values: BTreeMap<String, MyValue> = (0..row.column_count())
                    .map(|i| (row.column_name(i).unwrap().to_owned(), row.get_unwrap(i)))
                    .collect();
                Ok(values)
            })?
            .collect::<Result<Vec<_>, _>>()?;
        txn.commit()?;
        Ok(serde_json::to_value(&rows).unwrap())
    }
    fn mutate_named(&self, name: String, params: BTreeMap<String, Value>) -> PersistenceResult<()> {
        debug!("performing named mutation: {}", name);
        let txn = self.conn.unchecked_transaction()?;
        let mutation: String = txn.query_row(
            "SELECT raw_sql FROM __ezdb_metadata__ WHERE type = 'mutation' AND name = ?",
            &[&name],
            |row| row.get(0),
        )?;
        let params: Vec<(String, MyValue)> =
            params.into_iter().map(|(k, v)| (k, v.into())).collect();
        let params: Vec<(&str, &dyn ToSql)> = params
            .iter()
            .map(|(k, v)| (k.as_ref(), v as &dyn ToSql))
            .collect();
        let mut stmt = self.conn.prepare(&mutation)?;
        stmt.execute_named(params.as_slice())?;
        txn.commit()?;
        Ok(())
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
    fn fetch_policy(&self) -> PersistenceResult<Policy> {
        debug!("fetching policy");
        let mut queries = self
            .conn
            .prepare("SELECT name, raw_sql FROM __ezdb_metadata__ WHERE type = 'query'")?;
        let queries: Vec<QueryPolicy> = queries
            .query_map(NO_PARAMS, |row| {
                let name: String = row.get(0)?;
                let raw_sql: String = row.get(1)?;
                Ok(QueryPolicy { name, raw_sql })
            })?
            .collect::<Result<_, _>>()?;
        let mut mutations = self
            .conn
            .prepare("SELECT name, raw_sql FROM __ezdb_metadata__ WHERE type = 'mutation'")?;
        let mutations: Vec<MutationPolicy> = mutations
            .query_map(NO_PARAMS, |row| {
                let name: String = row.get(0)?;
                let raw_sql: String = row.get(1)?;
                Ok(MutationPolicy { name, raw_sql })
            })?
            .collect::<Result<_, _>>()?;
        Ok(Policy { queries, mutations })
    }
    fn set_policy(&self, policy: Policy) -> PersistenceResult<()> {
        debug!("updating policy to: {:?}", policy);
        let mut txn = self.conn.unchecked_transaction()?;
        txn.execute("DELETE FROM __ezdb_metadata__", NO_PARAMS)?;
        populate_policy(&mut txn, policy)?;
        txn.commit()?;
        Ok(())
    }
}

fn populate_policy(txn: &mut Transaction, policy: Policy) -> PersistenceResult<()> {
    let mut stmt =
        txn.prepare("INSERT INTO __ezdb_metadata__ (type, name, raw_sql) VALUES (?, ?, ?)")?;
    for p in policy.queries {
        stmt.execute(&["query", &p.name, &p.raw_sql])?;
    }
    for p in policy.mutations {
        stmt.execute(&["mutation", &p.name, &p.raw_sql])?;
    }
    Ok(())
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

impl ToSql for MyValue {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(match self {
            MyValue::Null => ToSqlOutput::Borrowed(ValueRef::Null),
            MyValue::Integer(i) => ToSqlOutput::Borrowed(ValueRef::Integer(*i)),
            MyValue::Float(i) => ToSqlOutput::Borrowed(ValueRef::Real(*i)),
            MyValue::Text(i) => ToSqlOutput::Borrowed(ValueRef::Text(i.as_bytes())),
            MyValue::Bytes(ref i) => ToSqlOutput::Borrowed(ValueRef::Blob(i)),
        })
    }
}

impl From<Value> for MyValue {
    fn from(v: Value) -> MyValue {
        match v {
            Value::Null => MyValue::Null,
            Value::Bool(b) => MyValue::Integer(if b { 1 } else { 0 }),
            Value::Number(i) => i
                .as_i64()
                .map(|i| MyValue::Integer(i))
                .unwrap_or_else(|| MyValue::Float(i.as_f64().unwrap())),
            Value::String(i) => MyValue::Text(i),
            Value::Array(_) => panic!(),
            Value::Object(_) => panic!(),
        }
    }
}
