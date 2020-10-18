#![warn(rust_2018_idioms)]

use std::env;
use std::error::Error;
use std::io::{stdin, Read};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

use tokio::{
    time::{self, Duration},
};

fn get_stdin_data() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    stdin().read_to_end(&mut buf)?;
    Ok(buf)
}

#[derive(Debug)]
enum State {
    Start,
    Registering,
    Registered,
    Connecting_1,
    Connecting_2,
    Connected,
    Waiting_Connection,
}

// #[derive(Debug)]
// struct State_Machine {
//     _inner: State
// }

// impl State_Machine {
//     pub fn new() -> State_Machine {
//         State_Machine { _inner: State::Start }
//     }
// }

// //Start
// impl State_Machine {
//     pub fn start(self) -> State_Machine {
//         State_Machine { _inner: State::Registering }
//     }
// }

// //Start
// impl State_Machine {
//     pub fn change_state(self, new_state:State) -> State_Machine {
//         State_Machine { _inner: new_state }
//     }
// }

// //Registering
// impl State_Machine {
//     pub fn received_register_approved(self) -> State_Machine {
//         State_Machine { _inner: State::Registered }
//     }
// }

// //Registered
// impl State_Machine {
//     pub fn received_connection_request(self) -> State_Machine {
//         State_Machine { _inner: State::Connecting_1 }
//     }
//     pub fn send_connection_request(self) -> State_Machine {
//         State_Machine { _inner: State::Waiting_Connection }
//     }
// }

// //Connecting_1
// impl State_Machine {
//     pub fn time_out(self) -> State_Machine {
//         State_Machine { _inner: State::Registered }
//     }
//     pub fn received_a_connected(self) -> State_Machine {
//         State_Machine { _inner: State::Connecting_2 }
//     }
// }

// //Connecting_2
// impl State_Machine {
//     pub fn received_connected_2(self) -> State_Machine {
//         State_Machine { _inner: State::Connected }
//     }
// }

// fn main() {
//     let state = State::new(); // green
//     let state = state.next(); // yellow
//     let state = state.next(); // red
//     let state = state.next(); // green
//     dbg!(state);
// }

use serde::{Deserialize, Serialize};

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

use std::io;

async fn listen_udp(socket: &UdpSocket, buf: &mut [u8], wait_time_ms: u64) -> io::Result<(bool, usize, SocketAddr)> {
    
    let res = match time::timeout(Duration::from_millis(wait_time_ms), socket.recv_from(buf)).await {
        Ok(test) => {
            let (count, src) = test.unwrap();
            (true, count, src)
        }
        Err(e) => {
            (false, 0 as usize, "0.0.0.0:0".parse().unwrap())
        } 
    };
    Ok(res)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let remote_addr: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".into())
        .parse()?;

    let local_addr: SocketAddr = env::args()
        .nth(2)
        .unwrap_or_else(|| "127.0.0.1:8888".into())
        .parse()?;

    // We use port 0 to let the operating system allocate an available port for us.
    // let local_addr: SocketAddr = if remote_addr.is_ipv4() {
    //     "0.0.0.0:8888"
    // } else {
    //     "[::]:0"
    // }
    // .parse()?;

    let mut state = State::Start;
    let socket = UdpSocket::bind(local_addr).await?;
    const MAX_DATAGRAM_SIZE: usize = 65_507;
    let mut data = vec![0u8; MAX_DATAGRAM_SIZE];

    loop {
        dbg!(&state);
        state = match state {
            
            State::Start => {
                State::Registering
            }

            State::Registering => {
                let encoded: Vec<u8> = bincode::serialize(&Package {
                    package_type: Package_Type::register_request,
                    data: Vec::new(),
                })
                .unwrap();
 
                socket.send_to(&encoded, &remote_addr).await?;
                let res = listen_udp(&socket, &mut data, 100).await?;
                dbg!(&res);
                let mut new_state = State::Registering;
                if res.0 {
                    let a: Package = bincode::deserialize(&data[..res.1]).unwrap();
                    if a.package_type == Package_Type::register_approve {
                        new_state = State::Registered;
                    }
                }

                new_state
            },
            _ => {
                println!("Error, not a valid state");
                state
            }
        };
        // break;
    }

    

    // let data = get_stdin_data()?;
    // loop {
    //     let encoded: Vec<u8> = bincode::serialize(&test).unwrap();

    //     socket.send_to(&encoded, &remote_addr).await?;
    //     let mut data = vec![0u8; MAX_DATAGRAM_SIZE];

    //     let (len, peer) = socket.recv_from(&mut data).await?;

    //     println!(
    //         "Received {} bytes from {}:\n{}",
    //         len,
    //         peer,
    //         String::from_utf8_lossy(&data[..len])
    //     );

    //     let a: Entity = bincode::deserialize(&data).unwrap();

    //     test.x = a.x + 1.0;
    //     test.y = a.y + 1.0;

    //     println!("x: {}, y: {}", a.x, a.y,);
    // }

    Ok(())
}
