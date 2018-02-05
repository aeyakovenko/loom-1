use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::net::SocketAddr;
use mio;

type Messages = Arc<(Vec<data::Message>, Vec<(usize, SocketAddr)>, usize)>

struct Data {
    pending: VecDeque<Messages>,
    gc: Vec<Messages>,
}
struct Reader {
    lock: Mutex<Data>,
    port: u16,
    poll: mio::Poll,
    sock: mio::net::UdpSocket,
}
impl Reader {
    pub fn new(port: u16) -> Result<Reader> {
        let d = Data { gc: Vec::new(),
                       pending: Vec::new() };

        return Reader{lock: Mutex::new(d), poll: poll, sock: sock};
    }
    pub fn next(&self) -> Result<Messages> {
        let d = self.lock.lock()
        d.pending.pop_front()
    }
    pub fn recycle(&self, m: Messages) -> Result<Messages> {
        let d = self.lock.lock()
        d.gc.push(m)
    }
    pub fn run(&self) {
        let mut num = 0usize;
        let ipv4 = Ipv4Addr::new(0, 0, 0, 0);
        let addr = SocketAddr::new(IpAddr::V4(ipv4), self.port);
        const READABLE: mio::Token = mio::Token(0);
        let poll = mio::Poll::new()?;
        let srv = mio::net::UdpSocket::bind(&addr)?;
        poll.register(&srv, READABLE, mio::Ready::readable(),
                       mio::PollOpt::edge())?;
        let mut events = mio::Events::with_capacity(8);
        
        loop {
            self.poll.poll(&mut events, None)?;
            let mut m =  self.allocate();
            m.2 = net::read_from(&srv, &mut m.0, &mut m.1);
            self.enqueue(m);
            self.notify();
        }
    }
    fn notify(&self) {
        //TODO(anatoly), hard code other threads to notify
    }
    fn allocate(&self) -> Messages {
        let d = self.lock.lock();
        return match d.gc.pop() {
                Some(v) => {
                    v.2 = 0;
                    v;
                },
                _ => {
                    let mut m = Vec::new();
                    m.resize(1024, data::Message::default());
                    let mut d = Vec::new();
                    d.resize(1024, Default::default());
                    Arc::new((v, d, 0))
                }
        }
    }
    fn enqueue(&self, m: Messages) {
        let d = self.lock.lock();
        d.pending.push_back(m);
        return match d.gc.pop() {
                Some(v) => v,
                _ => {
                    let mut v = Vec::new();
                    v.resize(1024, data::Message::default());
                    Arc::new(v)
                }
        }
    }
}
