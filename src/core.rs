use crate::persistence::{Persistence, PersistenceError, PersistenceResult, SqliteFactory};
use crate::tokens::DatabaseAddress;
use actix::prelude::*;
use crossbeam_channel::Sender;
use futures::FutureExt;
use log::debug;
use rusqlite::InterruptHandle;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::hash_map::Entry;
use std::{
    collections::{BTreeMap, HashMap},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

/// `RoutingActor` supervises all the active databases.
pub struct RoutingActor {
    persistence: SqliteFactory,
    actors: HashMap<DatabaseAddress, Addr<CoreActor>>,
}
impl RoutingActor {
    pub fn new(persistence: SqliteFactory) -> RoutingActor {
        RoutingActor {
            persistence,
            actors: HashMap::new(),
        }
    }
}

impl Actor for RoutingActor {
    type Context = Context<Self>;
}

impl Message for DatabaseAddress {
    type Result = PersistenceResult<Addr<CoreActor>>;
}
impl Handler<DatabaseAddress> for RoutingActor {
    type Result = PersistenceResult<Addr<CoreActor>>;

    fn handle(
        &mut self,
        db_addr: DatabaseAddress,
        _ctx: &mut Context<Self>,
    ) -> PersistenceResult<Addr<CoreActor>> {
        let addr = match self.actors.entry(db_addr) {
            Entry::Occupied(occ) => occ.get().clone(),
            Entry::Vacant(vac) => {
                let db = crate::persistence::Timed::new(self.persistence.open(vac.key())?);
                vac.insert(CoreActor::new(db).start()).clone()
            }
        };
        Ok(addr)
    }
}

struct Job<I, O> {
    input: I,
    output: futures::channel::oneshot::Sender<O>,
    generation: usize,
}
/// `CoreActor` manages connections to a given database.
pub struct CoreActor {
    queue: Sender<Job<DataMessage, PersistenceResult<String>>>,
    interrupt_handle: InterruptHandle,
    generation: Arc<AtomicUsize>,
}

const MAILBOX_SIZE: usize = 16;
impl CoreActor {
    pub fn new<P: Persistence + 'static>(mut persistence: P) -> CoreActor {
        let interrupt_handle = (&persistence).get_interrupt_handle();
        let (tx, rx) =
            crossbeam_channel::bounded::<Job<DataMessage, PersistenceResult<String>>>(MAILBOX_SIZE);
        let signal = Arc::new(AtomicUsize::new(0));
        let signal2 = signal.clone();
        std::thread::spawn(move || {
            let signal = signal.clone();
            while let Ok(job) = rx.recv() {
                let r = if signal.load(Ordering::Relaxed) > job.generation {
                    Err(PersistenceError::Interrupted)
                } else {
                    handle_data_request(&mut persistence, job.input)
                };
                let _ = job.output.send(r);
            }
        });
        CoreActor {
            queue: tx,
            interrupt_handle,
            generation: signal2,
        }
    }

    pub fn interrupt(&self) {
        self.generation.fetch_add(1, Ordering::Relaxed);
        self.interrupt_handle.interrupt();
    }
}

impl Actor for CoreActor {
    type Context = Context<Self>;

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.interrupt();
    }
}

/// Message to interact with the data in the database.
#[derive(Debug)]
pub enum EzdbMessage {
    Data(DataMessage),
    Logistics(LogisticsMessage),
}

/// Message to interact with the data in the database.
#[derive(Debug)]
pub enum DataMessage {
    QueryNamed(String, BTreeMap<String, Value>),
    MutateNamed(String, BTreeMap<String, Value>),
    QueryRaw(String),
    MutateRaw(String),
    FetchPolicy,
    SetPolicy(Policy),
}

/// Message to control the logistics of the database.
#[derive(Debug)]
pub enum LogisticsMessage {
    Interrupt,
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

impl Message for EzdbMessage {
    type Result = PersistenceResult<String>;
}

impl Handler<EzdbMessage> for CoreActor {
    type Result = ResponseFuture<PersistenceResult<String>>;

    fn handle(&mut self, msg: EzdbMessage, _: &mut Context<Self>) -> Self::Result {
        match msg {
            EzdbMessage::Logistics(LogisticsMessage::Interrupt) => {
                self.interrupt();
                Box::pin(std::future::ready(Ok("ok".to_owned())))
            }
            EzdbMessage::Data(input) => {
                let (tx, rx) = futures::channel::oneshot::channel();
                let job = Job {
                    input,
                    output: tx,
                    generation: self.generation.load(Ordering::Relaxed),
                };
                match self.queue.try_send(job) {
                    Ok(_) => Box::pin(rx.map(|r| r.unwrap())),
                    Err(_) => Box::pin(std::future::ready(Err(PersistenceError::Busy))),
                }
            }
        }
    }
}

fn handle_data_request<P: Persistence>(
    persistence: &mut P,
    msg: DataMessage,
) -> PersistenceResult<String> {
    debug!("handling {:?}", msg);
    match msg {
        DataMessage::QueryNamed(name, params) => {
            let data = persistence.query_named(name, params)?;
            Ok(serde_json::to_string(&data).expect("serialize"))
        }
        DataMessage::QueryRaw(query) => {
            let data = persistence.query_raw(query)?;
            Ok(serde_json::to_string(&data).expect("serialize"))
        }
        DataMessage::MutateNamed(name, params) => {
            let data = persistence.mutate_named(name, params)?;
            Ok(serde_json::to_string(&data).expect("serialize"))
        }
        DataMessage::MutateRaw(stmt) => {
            let data = persistence.mutate_raw(stmt)?;
            Ok(serde_json::to_string(&data).expect("serialize"))
        }
        DataMessage::FetchPolicy => {
            let data = persistence.fetch_policy()?;
            Ok(serde_json::to_string(&data).expect("serialize"))
        }
        DataMessage::SetPolicy(policy) => {
            let data = persistence.set_policy(policy)?;
            Ok(serde_json::to_string(&data).expect("serialize"))
        }
    }
}

#[cfg(test)]
mod test {
    use super::{CoreActor, DataMessage, EzdbMessage, LogisticsMessage};
    use crate::persistence::{PersistenceError, SqlitePersistence};
    use actix::{Actor, Addr};
    use std::time::Duration;

    #[actix_rt::test]
    async fn expensive_queries_can_be_interrupted() {
        let actor = CoreActor::new(SqlitePersistence::in_memory().unwrap()).start();
        mutate_raw(&actor, "CREATE TABLE foo (x INTEGER)").await;
        mutate_raw(&actor, "INSERT INTO foo (x) VALUES (0)").await;
        for _ in 0..10 {
            mutate_raw(&actor, "INSERT INTO foo (x) SELECT x FROM foo").await;
        }
        // `foo` now has 1024 entries. `foo JOIN foo JOIN foo` has 2^30 entries, which is extremely expensive.
        let m0 = actor.send(EzdbMessage::Data(DataMessage::QueryRaw(
            "SELECT COUNT(1) FROM foo JOIN foo JOIN foo".to_owned(),
        )));
        actix_rt::time::delay_for(Duration::from_millis(10)).await;
        let m1 = actor.send(EzdbMessage::Logistics(LogisticsMessage::Interrupt));

        assert_eq!(m1.await.unwrap().unwrap(), "ok");
        assert_eq!(m0.await.unwrap(), Err(PersistenceError::Interrupted));
    }

    async fn mutate_raw(actor: &Addr<CoreActor>, raw: &str) {
        let req = DataMessage::MutateRaw(raw.to_owned());
        actor.send(EzdbMessage::Data(req)).await.unwrap().unwrap();
    }
}
