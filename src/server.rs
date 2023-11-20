use std::net::TcpListener;
use std::fmt::Write;
mod net;
use crate::net::PORT;

fn main() -> std::io::Result<()> {
    let mut sockaddr = String::new();
    let _ = write!(sockaddr, "0.0.0.0:{}", PORT);
    let listener = TcpListener::bind(sockaddr).expect("Could not bind to address");

    for _stream in listener.incoming() {

    }
    Ok(())
}
