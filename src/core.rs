use crate::persistence::inmemory::InMemoryPersistence;
use crate::persistence::{Persistence, PersistenceResult};
use actix::prelude::*;

/// `CoreActor` manages connections to a given database.
#[derive(Default)]
pub struct CoreActor {
    persistence: InMemoryPersistence,
}

impl Actor for CoreActor {
    type Context = Context<Self>;
}

/// Message from a REST request
#[derive(Debug)]
pub enum RestMessage {
    QueryNamed(String),
    MutateNamed(String),
    QueryRaw(String),
    MutateRaw(String),
}

impl Message for RestMessage {
    type Result = PersistenceResult<String>;
}

impl Handler<RestMessage> for CoreActor {
    type Result = PersistenceResult<String>;

    fn handle(&mut self, msg: RestMessage, _: &mut Context<Self>) -> Self::Result {
        match msg {
            RestMessage::QueryNamed(name) => {
                let data = self.persistence.query_named(name)?;
                Ok(serde_json::to_string(&data).expect("serialize"))
            }
            RestMessage::QueryRaw(query) => {
                let data = self.persistence.query_raw(query)?;
                Ok(serde_json::to_string(&data).expect("serialize"))
            }
            RestMessage::MutateNamed(name) => {
                let data = self.persistence.mutate_named(name)?;
                Ok(serde_json::to_string(&data).expect("serialize"))
            }
            RestMessage::MutateRaw(stmt) => {
                let data = self.persistence.mutate_raw(stmt)?;
                Ok(serde_json::to_string(&data).expect("serialize"))
            }
        }
    }
}
