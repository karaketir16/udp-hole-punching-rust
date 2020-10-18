//! A UDP client that just sends everything it gets via `stdio` in a single datagram, and then
//! waits for a reply.
//!
//! For the reasons of simplicity data from `stdio` is read until `EOF` in a blocking manner.
//!
//! You can test this out by running an echo server:
//!
//! ```
//!     $ cargo run --example echo-udp -- 127.0.0.1:8080
//! ```
//!
//! and running the client in another terminal:
//!
//! ```
//!     $ cargo run --example udp-client
//! ```
//!
//! You can optionally provide any custom endpoint address for the client:
//!
//! ```
//!     $ cargo run --example udp-client -- 127.0.0.1:8080
//! ```
//!
//! Don't forget to pass `EOF` to the standard input of the client!
//!
//! Please mind that since the UDP protocol doesn't have any capabilities to detect a broken
//! connection the server needs to be run first, otherwise the client will block forever.

#![warn(rust_2018_idioms)]

use std::env;
use std::error::Error;
use std::io::{stdin, Read};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

fn get_stdin_data() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    stdin().read_to_end(&mut buf)?;
    Ok(buf)
}


use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Entity {
    x: f32,
    y: f32,
}


#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum Package_Type {
    connection_request,
    register_request,
    register_approve,
    connected_1,
    connected_2,
}

#[derive(Serialize, Deserialize, Debug)]
struct Package {
    package_type: Package_Type,
    data: Vec<u8>,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let remote_addr: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8888".into())
        .parse()?;

    let local_addr: SocketAddr = env::args()
        .nth(2)
        .unwrap_or_else(|| "127.0.0.1:8080".into())
        .parse()?;

    // We use port 0 to let the operating system allocate an available port for us.
    // let local_addr: SocketAddr = if remote_addr.is_ipv4() {
    //     "0.0.0.0:8888"
    // } else {
    //     "[::]:0"
    // }
    // .parse()?;

    let mut test: Package = Package {
        package_type: Package_Type::register_approve,
        data: Vec::new(),
    };

    let socket = UdpSocket::bind(local_addr).await?;
    const MAX_DATAGRAM_SIZE: usize = 65_507;

    // let data = get_stdin_data()?;
    loop{
        let encoded: Vec<u8> = bincode::serialize(&test).unwrap();

        socket.send_to(&encoded, &remote_addr).await?;
        let mut data = vec![0u8; MAX_DATAGRAM_SIZE];
        
        let (len, peer) = socket.recv_from(&mut data).await?;
    
        println!(
            "Received {} bytes from {}:\n{}",
            len, peer,
            String::from_utf8_lossy(&data[..len])
        );
    
        let a:Package = bincode::deserialize(&data).unwrap();
    
        dbg!(a);
    }    

    Ok(())
}
