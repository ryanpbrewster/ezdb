use crate::{
    core::Policy,
    persistence::{Persistence, PersistenceResult},
};
use log::trace;
use rusqlite::InterruptHandle;
use serde_json::Value;
use std::collections::BTreeMap;

pub struct Timed<P: Persistence>(P);
impl<P: Persistence> Timed<P> {
    pub fn new(inner: P) -> Timed<P> {
        Timed(inner)
    }
}

macro_rules! timed {
    ($v:expr) => {{
        let start = std::time::Instant::now();
        let ans = $v;
        trace!(
            "[{}:{}] {}us",
            file!(),
            line!(),
            start.elapsed().as_micros()
        );
        ans
    }};
}

impl<P: Persistence> Persistence for Timed<P> {
    #[track_caller]
    fn query_named(
        &self,
        name: String,
        params: BTreeMap<String, Value>,
    ) -> PersistenceResult<Value> {
        timed!(self.0.query_named(name, params))
    }
    fn mutate_named(&self, name: String, params: BTreeMap<String, Value>) -> PersistenceResult<()> {
        timed!(self.0.mutate_named(name, params))
    }
    fn query_raw(&self, query: String) -> PersistenceResult<Value> {
        timed!(self.0.query_raw(query))
    }
    fn mutate_raw(&self, stmt: String) -> PersistenceResult<()> {
        timed!(self.0.mutate_raw(stmt))
    }
    fn fetch_policy(&self) -> PersistenceResult<Policy> {
        timed!(self.0.fetch_policy())
    }
    fn set_policy(&self, policy: Policy) -> PersistenceResult<()> {
        timed!(self.0.set_policy(policy))
    }
    fn get_interrupt_handle(&self) -> InterruptHandle {
        timed!(self.0.get_interrupt_handle())
    }
}
