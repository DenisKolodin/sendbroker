#[macro_use]
extern crate log;

use std::fmt;
use std::error;
use std::result;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {
    NotFound,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::NotFound => "not found",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        f.write_str(self.description())
    }
}

pub type Result<T> = result::Result<Option<Sender<T>>, Error>;

pub trait Registrar<I, T> {
    fn reg_sender(&self, id: I, sender: Sender<T>) -> Result<T>;
    fn unreg_sender(&self, id: I) -> Result<T>;
}

pub trait Finder<I, T> {
    fn find_sender(&self, id: I) -> Result<T>;
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
    fn reg_sender(&self, id: I, sender: Sender<T>) -> Result<T> {
        trace!("Register in broker id: {:?}", id);
        let mut map = self.map.lock().unwrap();
        match (*map).insert(id, sender) {
            None => Ok(None),
            some => Ok(some),
        }
    }

    fn unreg_sender(&self, id: I) -> Result<T> {
        trace!("Unregister in broker id: {:?}", id);
        let mut map = self.map.lock().unwrap();
        match (*map).remove(&id) {
            None => Err(Error::NotFound),
            some => Ok(some),
        }
    }
}

impl<I, T> Finder<I, T> for SendBroker<I, T>
    where I: fmt::Debug + Hash + Eq
{
    fn find_sender(&self, id: I) -> Result<T> {
        trace!("Finding in broker id: {:?}", id);
        let map = self.map.lock().unwrap();
        match (*map).get(&id).cloned() {
            None => Err(Error::NotFound),
            some => Ok(some),
        }
    }
}

