#[macro_use]
extern crate log;

use std::hash::Hash;
use std::thread;
use std::sync::mpsc::{channel, Sender, SendError, RecvError};
use std::collections::HashMap;

pub enum Error {
    ConnectionBroken,
    NoRegistration,
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

pub trait Registrar<I, T> {
    fn reg_sender(&self, id: I, sender: Sender<T>) -> Result<(), Error>;
    fn unreg_sender(&self, id: I) -> Result<(), Error>;
}

pub trait Finder<I, T> {
    fn find_sender(&self, id: I) -> Result<Sender<T>, Error>;
}

enum Action<I, T> {
    Register(I, Sender<T>),
    Unregister(I),
    Find(I),
}

struct Request<I, T> {
    action: Action<I, T>,
    recipient: Sender<Option<Sender<T>>>,
}

#[derive(Clone)]
pub struct SendBroker<I, T> {
    sender: Sender<Request<I, T>>,
}

impl<I, T> SendBroker<I, T>
    where I: Hash + Eq + Send + 'static, T: Send + 'static
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
                                    map.insert(id, conn)
                                },
                                Action::Unregister(id) => {
                                    map.remove(&id)
                                },
                                Action::Find(id) => {
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
    fn do_action(&self, action: Action<I, T>) -> Result<Option<Sender<T>>, Error> {
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
    fn reg_sender(&self, id: I, sender: Sender<T>) -> Result<(), Error> {
        self.do_action(Action::Register(id, sender)).map(|_| ())
    }

    fn unreg_sender(&self, id: I) -> Result<(), Error> {
        self.do_action(Action::Unregister(id))?.map(|_| ()).ok_or(Error::NoRegistration)
    }
}

impl<I, T> Finder<I, T> for SendBroker<I, T> {
    fn find_sender(&self, id: I) -> Result<Sender<T>, Error> {
        self.do_action(Action::Find(id))?.ok_or(Error::NoRegistration)
    }
}


#[cfg(test)]
mod tests {
    //use super::SendBroker;

    #[test]
    fn test_sendbroker() {
    }
}
