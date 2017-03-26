#[macro_use]
extern crate log;

use std::fmt;
use std::error;
use std::result;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, SendError, RecvError};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {
    ConnectionBroken,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ConnectionBroken => "broker connection broken",
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


impl<T> From<SendError<T>> for Error {
    fn from(_: SendError<T>) -> Self {
        Error::ConnectionBroken
    }
}

impl From<RecvError> for Error {
    fn from(_: RecvError) -> Self {
        Error::ConnectionBroken
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

/*
enum Action<I, T> {
    Register(I, Sender<T>),
    Unregister(I),
    Find(I),
}

struct Request<I, T> {
    action: Action<I, T>,
    recipient: Sender<Option<Sender<T>>>,
}
*/

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
            None => Err(Error::ConnectionBroken),
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
            None => Err(Error::ConnectionBroken),
            some => Ok(some),
        }
    }
}

/*

pub struct SendBroker<I, T> {
    sender: Sender<Request<I, T>>,
}

impl<I, T> Clone for SendBroker<I, T> {
    fn clone(&self) -> Self {
        SendBroker {
            sender: self.sender.clone(),
        }
    }
}

impl<I, T> SendBroker<I, T>
    where I: fmt::Debug + Hash + Eq + Send + 'static, T: Send + 'static
{
    pub fn new() -> Self {
        let (tx, rx) = channel();
        thread::spawn(move || {
            let mut map: HashMap<I, Sender<T>> = HashMap::new();
            loop {
                match rx.recv() {
                    Ok(Request { action, recipient }) => {
                        let opt = {
                            match action {
                                Action::Register(id, conn) => {
                                    trace!("Register in broker id: {:?}", id);
                                    map.insert(id, conn)
                                },
                                Action::Unregister(id) => {
                                    trace!("Unregister in broker id: {:?}", id);
                                    map.remove(&id)
                                },
                                Action::Find(id) => {
                                    trace!("Finding in broker id: {:?}", id);
                                    map.get(&id).cloned()
                                },
                            }
                        };
                        if let Err(_) = recipient.send(opt) {
                            warn!("Can't send response to recipient");
                        }
                    },
                    Err(_) => {
                        error!("Receiver of send broker corrupted");
                        break;
                    },
                }
            }
        });
        SendBroker {
            sender: tx,
        }
    }
}

impl<I, T> SendBroker<I, T> {
    fn do_action(&self, action: Action<I, T>) -> Result<T> {
        let (tx, rx) = channel();
        let request = Request {
            action: action,
            recipient: tx,
        };
        self.sender.send(request)?;
        Ok(rx.recv()?)
    }
}

impl<I, T> Registrar<I, T> for SendBroker<I, T> {
    fn reg_sender(&self, id: I, sender: Sender<T>) -> Result<T> {
        self.do_action(Action::Register(id, sender))
    }

    fn unreg_sender(&self, id: I) -> Result<T> {
        self.do_action(Action::Unregister(id))
    }
}

impl<I, T> Finder<I, T> for SendBroker<I, T> {
    fn find_sender(&self, id: I) -> Result<T> {
        self.do_action(Action::Find(id))
    }
}


#[cfg(test)]
mod tests {
    //use super::SendBroker;

    #[test]
    fn test_sendbroker() {
    }
}
*/
