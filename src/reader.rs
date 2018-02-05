use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::net::{SocketAddr, Ipv4Addr, IpAddr};
use result::{Result, from_option};
use mio;
use data;
use net;

struct MessageData {
    msgs: Vec<data::Message>,
    data: Vec<(usize, SocketAddr),
}

type Messages = Arc<MessageData>;

struct Data {
    pending: VecDeque<Messages>,
    gc: Vec<Messages>,
}
struct Reader {
    lock: Mutex<Data>,
    port: u16,
}
impl Reader {
    pub fn new(port: u16) -> Reader {
        let d = Data { gc: Vec::new(),
                       pending: VecDeque::new() };

        return Reader{lock: Mutex::new(d), port: port};
    }
    pub fn next(&self) -> Result<Messages> {
        let mut d = self.lock.lock().expect("lock");
        let o = d.pending.pop_front();
        return from_option(o);
    }
    pub fn recycle(&self, m: Messages) {
        let mut d = self.lock.lock().expect("lock");
        d.gc.push(m);
    }
    pub fn run(&self) -> Result<()> {
        let ipv4 = Ipv4Addr::new(0, 0, 0, 0);
        let addr = SocketAddr::new(IpAddr::V4(ipv4), self.port);
        const READABLE: mio::Token = mio::Token(0);
        let poll = mio::Poll::new()?;
        let srv = mio::net::UdpSocket::bind(&addr)?;
        poll.register(&srv, READABLE, mio::Ready::readable(),
                       mio::PollOpt::edge())?;
        let mut events = mio::Events::with_capacity(8);
        
        loop {
            poll.poll(&mut events, None)?;
            let mut m =  self.allocate();
            let num = net::read_from(&srv, &mut m.msgs, &mut m.data)?;
            let total = m.data.sum(|v| v.0);
            m.msgs.resize(total);
            m.data.resize(num);
            self.enqueue(m);
            self.notify();
        }
    }
    fn notify(&self) {
        //TODO(anatoly), hard code other threads to notify
    }
    fn allocate(&self) -> Messages {
        let mut s = self.lock.lock().expect("lock");
        return match s.gc.pop() {
                Some(v) => {
                    v.2 = 0;
                    v
                },
                _ => {
                    let mut m = Vec::new();
                    m.resize(1024, data::Message::default());
                    let mut d = Vec::new();
                    let df = (0, "0.0.0.0:0000".parse().expect("parse"));
                    d.resize(1024, df);
                    Arc::new(MessageData{msgs:m, data:d})
                }
        }
    }
    fn enqueue(&self, m: Messages) {
        let mut s = self.lock.lock().expect("lock");
        s.pending.push_back(m);
    }
}
