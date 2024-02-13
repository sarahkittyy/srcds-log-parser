use srcds_log_parser::{LogMessage, LogParseError};

use std::{env, net::UdpSocket};

fn main() {
    let port: u16 = env::args()
        .next()
        .and_then(|a| a.parse::<u16>().ok())
        .unwrap_or(9999);

    let sock = UdpSocket::bind(("0.0.0.0", port)).expect("Could not bind to port");
    println!("Listening on port {}", port);

    let mut buf = [0u8; 1024];
    loop {
        let (len, from) = sock.recv_from(&mut buf).unwrap();
        let message = match LogMessage::from_bytes(&buf[..len]) {
            Ok(m) => m,
            Err(e) => {
                println!("Could not parse packet from {from:?} with len {len}: {e:?}");
                continue;
            }
        };
        println!("{:?}", message)
    }
}
