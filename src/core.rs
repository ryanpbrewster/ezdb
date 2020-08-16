use crate::persistence::inmemory::InMemoryPersistence;
use crate::persistence::Persistence;
use actix::prelude::*;
use serde_json::Value;

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
    Get(String),
    Delete(String),
    Put(String, Value),
}

impl Message for RestMessage {
    type Result = Option<String>;
}

impl Handler<RestMessage> for CoreActor {
    type Result = Option<String>;

    fn handle(&mut self, msg: RestMessage, _: &mut Context<Self>) -> Self::Result {
        match msg {
            RestMessage::Get(path) => {
                let data = self.persistence.get(path).expect("read");
                Some(serde_json::to_string(data).expect("serialize"))
            }
            RestMessage::Put(path, data) => {
                self.write(path, data);
                None
            }
            RestMessage::Delete(path) => {
                self.write(path, Value::Null);
                None
            }
        }
    }
}

impl CoreActor {
    fn write(&mut self, path: String, data: Value) {
        self.persistence.put(path, data).expect("put data");
    }
}
