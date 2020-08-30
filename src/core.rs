use crate::persistence::{Persistence, PersistenceError, PersistenceResult, SqlitePersistence};
use crate::tokens::DatabaseId;
use actix::prelude::*;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// `CoreActor` manages connections to a given database.
pub struct CoreActor {
    persistence: SqlitePersistence,
}
impl CoreActor {
    pub fn new(persistence: SqlitePersistence) -> CoreActor {
        CoreActor { persistence }
    }
}

impl Actor for CoreActor {
    type Context = Context<Self>;
}

/// Message from a REST request
#[derive(Debug)]
pub enum RestMessage {
    QueryNamed(String, BTreeMap<String, Value>),
    MutateNamed(String, BTreeMap<String, Value>),
    QueryRaw(String),
    MutateRaw(String),
    FetchPolicy,
    SetPolicy(Policy),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Policy {
    pub queries: Vec<QueryPolicy>,
    pub mutations: Vec<MutationPolicy>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryPolicy {
    pub name: String,
    pub raw_sql: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationPolicy {
    pub name: String,
    pub raw_sql: String,
}

impl Message for RestMessage {
    type Result = PersistenceResult<String>;
}

impl Handler<RestMessage> for CoreActor {
    type Result = PersistenceResult<String>;

    fn handle(&mut self, msg: RestMessage, _: &mut Context<Self>) -> Self::Result {
        debug!("handling {:?}", msg);
        match msg {
            RestMessage::QueryNamed(name, params) => {
                let data = self.persistence.query_named(name, params)?;
                Ok(serde_json::to_string(&data).expect("serialize"))
            }
            RestMessage::QueryRaw(query) => {
                let data = self.persistence.query_raw(query)?;
                Ok(serde_json::to_string(&data).expect("serialize"))
            }
            RestMessage::MutateNamed(name, params) => {
                let data = self.persistence.mutate_named(name, params)?;
                Ok(serde_json::to_string(&data).expect("serialize"))
            }
            RestMessage::MutateRaw(stmt) => {
                let data = self.persistence.mutate_raw(stmt)?;
                Ok(serde_json::to_string(&data).expect("serialize"))
            }
            RestMessage::FetchPolicy => {
                let data = self.persistence.fetch_policy()?;
                Ok(serde_json::to_string(&data).expect("serialize"))
            }
            RestMessage::SetPolicy(policy) => {
                let data = self.persistence.set_policy(policy)?;
                Ok(serde_json::to_string(&data).expect("serialize"))
            }
        }
    }
}
