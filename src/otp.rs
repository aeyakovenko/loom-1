//! see test for usage
//! small actor library for named channels inspired by erlang OTP

use std::sync::{Arc, Mutex, RwLock};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{spawn, JoinHandle};
use std::time::Duration;
use std::net::SocketAddr;
use data;
use result::Result;
use result::Error;

pub enum Port {
    Main,
    Reader,
    State,
    Recycle,
    Sender,
}

impl Port {
    fn to_usize(self) -> usize {
        match self {
            Port::Main => 0,
            Port::Reader => 1,
            Port::State => 2,
            Port::Recycle => 3,
            Port::Sender => 4,
        }
    }
}

#[derive(Clone)]
pub enum Data {
    Signal,
    SharedMessages(data::SharedMessages),
    SendMessage(data::Message, SocketAddr),
}

struct Locked {
    ports: Vec<Sender<Data>>,
    readers: Vec<Arc<Mutex<Receiver<Data>>>>,
    threads: Vec<Arc<Option<JoinHandle<Result<()>>>>>,
}

pub struct OTP {
    lock: Arc<RwLock<Locked>>,
    exit: Arc<Mutex<bool>>,
}

pub type Ports = Vec<Sender<Data>>;

impl OTP {
    pub fn new() -> OTP {
        let (s1, r1) = channel();
        let (s2, r2) = channel();
        let (s3, r3) = channel();
        let (s4, r4) = channel();
        let (s5, r5) = channel();
        let locked = Locked {
            ports: [s1, s2, s3, s4, s5].to_vec(),
            readers: [
                Arc::new(Mutex::new(r1)),
                Arc::new(Mutex::new(r2)),
                Arc::new(Mutex::new(r3)),
                Arc::new(Mutex::new(r4)),
                Arc::new(Mutex::new(r5)),
            ].to_vec(),
            threads: [
                Arc::new(None),
                Arc::new(None),
                Arc::new(None),
                Arc::new(None),
                Arc::new(None),
            ].to_vec(),
        };
        let exit = Arc::new(Mutex::new(false));
        OTP {
            lock: Arc::new(RwLock::new(locked)),
            exit: exit,
        }
    }
    pub fn source<F>(&self, port: Port, func: F) -> Result<()>
    where
        F: Send + 'static + Fn(&Ports) -> Result<()>,
    {
        let mut w = self.lock.write().unwrap();
        let pz = port.to_usize();
        if w.threads[pz].is_some() {
            return Err(Error::OTPError);
        }
        let c_ports = w.ports.clone();
        let c_exit = self.exit.clone();
        let j = spawn(move || loop {
            match func(&c_ports) {
                Ok(()) => (),
                e => return e,
            }
            if *c_exit.lock().unwrap() == true {
                return Ok(());
            }
        });
        w.threads[pz] = Arc::new(Some(j));
        return Ok(());
    }
    pub fn listen<F>(&mut self, port: Port, func: F) -> Result<()>
    where
        F: Send + 'static + Fn(&Ports, Data) -> Result<()>,
    {
        let mut w = self.lock.write().unwrap();
        let pz = port.to_usize();
        if w.threads[pz].is_some() {
            return Err(Error::OTPError);
        }
        let recv_lock = w.readers[pz].clone();
        let c_ports = w.ports.clone();
        let c_exit = self.exit.clone();
        let j: JoinHandle<Result<()>> = spawn(move || loop {
            let recv = recv_lock.lock().unwrap();
            let timer = Duration::new(0, 500000);
            match recv.recv_timeout(timer) {
                Ok(val) => func(&c_ports, val).expect("otp listen"),
                _ => (),
            }
            if *c_exit.lock().unwrap() == true {
                return Ok(());
            }
        });
        w.threads[pz] = Arc::new(Some(j));
        return Ok(());
    }
    pub fn send(ports: &Ports, to: Port, m: Data) -> Result<()> {
        ports[to.to_usize()]
            .send(m)
            .or_else(|_| Err(Error::SendError))
    }
    pub fn join(&mut self) -> Result<()> {
        let pz = Port::Main.to_usize();
        let recv = self.lock.write().unwrap().readers[pz].clone();
        recv.lock().unwrap().recv()?;
        self.shutdown()?;
        return Ok(());
    }
    pub fn shutdown(&mut self) -> Result<()> {
        {
            *self.exit.lock().unwrap() = true;
        }
        {
            let r = self.lock.read().unwrap();
            for t in r.threads.iter() {
                match Arc::try_unwrap((*t).clone()) {
                    Ok(Some(j)) => j.join()??,
                    _ => (),
                };
            }
        }
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use otp::OTP;
    use otp::Port::{Main, Reader, State};
    use otp::Data::Signal;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_init() {
        let mut o = OTP::new();
        assert_matches!(o.shutdown(), Ok(()));
    }
    #[test]
    fn test_join() {
        let mut o = OTP::new();
        assert_matches!(
            o.source(Reader, move |ports| OTP::send(ports, Main, Signal)),
            Ok(())
        );
        assert_matches!(o.join(), Ok(()));
    }
    #[test]
    fn test_source() {
        let mut o = OTP::new();
        assert_matches!(
            o.source(Reader, move |ports| OTP::send(ports, Main, Signal)),
            Ok(())
        );
        assert!(o.source(Reader, move |_ports| Ok(())).is_err());
        assert!(o.listen(Reader, move |_ports, _data| Ok(())).is_err());
        assert_matches!(o.join(), Ok(()));
    }
    #[test]
    fn test_listen() {
        let mut o = OTP::new();
        let val = Arc::new(Mutex::new(false));
        assert_matches!(
            o.source(Reader, move |ports| OTP::send(ports, State, Signal)),
            Ok(())
        );
        let c_val = val.clone();
        assert_matches!(
            o.listen(State, move |ports, data| match data {
                Signal => {
                    *c_val.lock().unwrap() = true;
                    OTP::send(ports, Main, Signal)
                }
                _ => Ok(()),
            }),
            Ok(())
        );
        assert_matches!(o.join(), Ok(()));
        assert_eq!(*val.lock().unwrap(), true);
    }

}
