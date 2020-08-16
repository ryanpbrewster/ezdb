use crate::persistence::{Persistence, PersistenceResult};
use log::debug;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct InMemoryPersistence {
    full_data: BTreeMap<String, Value>,
}

impl Persistence for InMemoryPersistence {
    fn get(&self, path: String) -> PersistenceResult<&Value> {
        Ok(&self
            .full_data
            .get(&path)
            .unwrap_or(&serde_json::Value::Null))
    }

    fn put(&mut self, path: String, value: Value) -> PersistenceResult<()> {
        debug!("writing @ {:?} == {}", path, value);
        self.full_data.insert(path, value);
        Ok(())
    }
}
