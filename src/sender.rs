use result::Result;
use std::net::UdpSocket;
use net;
use otp::Data;

pub struct Sender {
    s: UdpSocket,
}
impl Sender {
    pub fn new(sock: UdpSocket) -> Sender {
        Sender { s: sock }
    }

    pub fn run(&self, d: Data) -> Result<()> {
        match d {
            Data::SendMessage(m, a) => {
                let msgs = [m];
                let mut num = 0;
                while num < 1 {
                    net::send_to(&self.s, &msgs, &mut num, a)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
}
