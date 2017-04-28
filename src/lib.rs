#[macro_use] extern crate log;
#[macro_use] extern crate error_chain;

use std::fmt;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::collections::HashMap;

error_chain! {
    errors {
        NotFound
    }
}

pub type CallResult<T> = Result<Option<Sender<T>>>;

pub trait Registrar<I, T> {
    fn reg_sender(&self, id: I, sender: Sender<T>) -> CallResult<T>;
    fn unreg_sender(&self, id: I) -> CallResult<T>;
}

pub trait Finder<I, T> {
    fn find_sender(&self, id: I) -> CallResult<T>;
}

pub struct SendBroker<I, T> {
    map: Arc<Mutex<HashMap<I, Sender<T>>>>,
}

impl<I, T> SendBroker<I, T>
    where I: Hash + Eq
{
    pub fn new() -> Self {
        SendBroker {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<I, T> Clone for SendBroker<I, T> {
    fn clone(&self) -> Self {
        SendBroker {
            map: self.map.clone(),
        }
    }
}

impl<I, T> Registrar<I, T> for SendBroker<I, T>
    where I: fmt::Debug + Hash + Eq
{
    fn reg_sender(&self, id: I, sender: Sender<T>) -> CallResult<T> {
        trace!("Register in broker id: {:?}", id);
        let mut map = self.map.lock().unwrap();
        match (*map).insert(id, sender) {
            None => Ok(None),
            some => Ok(some),
        }
    }

    fn unreg_sender(&self, id: I) -> CallResult<T> {
        trace!("Unregister in broker id: {:?}", id);
        let mut map = self.map.lock().unwrap();
        match (*map).remove(&id) {
            None => Err(ErrorKind::NotFound.into()),
            some => Ok(some),
        }
    }
}

impl<I, T> Finder<I, T> for SendBroker<I, T>
    where I: fmt::Debug + Hash + Eq
{
    fn find_sender(&self, id: I) -> CallResult<T> {
        trace!("Finding in broker id: {:?}", id);
        let map = self.map.lock().unwrap();
        match (*map).get(&id).cloned() {
            None => Err(ErrorKind::NotFound.into()),
            some => Ok(some),
        }
    }
}

