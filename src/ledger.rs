#![allow(mutable_transmutes)]
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use result::Result;
use mio;
use mio::net::UdpSocket;
use std::mem::transmute;
use std::mem::size_of;
use std::slice::from_raw_parts;
use data;

pub struct Ledger {
    file: File,
}

//TODO(aeyakovenko): config file somewhere
const LEDGER: &str = "./loom.ledger";

impl Ledger {
    pub fn new() -> Result<Ledger> {
        let file = File::open(LEDGER)?;
        let l = Ledger { file: file };
        return Ok(l);
    }
    pub fn run() -> Result<()> {
        let poll = mio::Poll::new()?;
        const READABLE: mio::Token = mio::Token(0);
        let srv = net::ledger_server();
        poll.register(&srv, READABLE, mio::Ready::readable(),
                       mio::PollOpt::edge())?;
        let mut m = Vec::new();
        m.resize(1024, data::Message::default());
        let mut events = mio::Events::with_capacity(8);
        loop {
            poll.poll(&mut events, None)?;
            for event in events.iter() {
                match event.token() {
                    READABLE => {
                        let from = net::read_from(&srv, &mut m, &mut num,
                                                  &mut from)?;
                        self.execute(srv, from, &m[0 .. num]);
                    }
                    _ => (),
                }
            }
        }
        return Ok(());
    }
    pub fn execute(&self,  sock: &UdpSocket, from: &SocketAddr, msgs: &[data::Message]) -> Result<()> {
        for m in msgs.iter() {
            self.exec(sock, from, &m)?;
        }
        return Ok(());
    }
    pub fn append(&mut self, msgs: &[data::Message]) -> Result<()> {
        //TODO(aeyakovenko): the fastest way to do this:
        // have the msgs memory be mmaped
        // then `splice` the mmap fd into the file fd
        let p = &msgs[0] as *const data::Message;
        let sz = size_of::<data::Message>();
        let bz = msgs.len() * sz;
        let buf = unsafe { transmute(from_raw_parts(p as *const u8, bz)) };
        self.file.write_all(buf)?;
        return Ok(());
    }
    fn load(msgs: &mut [data::Message], start: u64) -> Result<()> {
        //TODO(aeyakovenko): the fastest way to do this:
        // have the msgs memory be mmaped
        // then `splice` from mmap fd, or to the socket directly
        let mut file = File::open(LEDGER)?;
        let p = &mut msgs[0] as *mut data::Message;
        let sz = size_of::<data::Message>();
        file.seek(SeekFrom::Start(sz as u64 * start))?;
        let bz = msgs.len() * sz;
        let buf = unsafe { transmute(from_raw_parts(p as *mut u8, bz)) };
        file.read(buf)?;
        return Ok(());
    }
    fn get_ledger(&self, sock: &UdpSocket, from: &SocketAddr, get: &data::GetLedger) -> Result<()> {
        let mut mem = Vec::new();
        mem.resize(get.num as usize, data::Message::default());
        Self::load(&mut mem, get.start)?;
        let p = &mem[0] as *const data::Message;
        let sz = size_of::<data::Message>();
        let bz = mem.len() * sz;
        let buf = unsafe { transmute(from_raw_parts(p as *const u8, bz)) };
        sock.send_to(buf, &from)?;
        return Ok(());
    }
    fn exec(&self, sock: &UdpSocket, from: &SocketAddr, m: &data::Message) -> Result<()> {
        match m.pld.kind {
            data::Kind::GetLedger => {
                let get = unsafe { &m.pld.data.get };
                self.get_ledger(sock, get)?;
            }
            _ => return Ok(()),
        };
        return Ok(());
    }

}
