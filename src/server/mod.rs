use std::{net::{TcpStream, TcpListener, Shutdown}, io::{Read, Error, ErrorKind, Write}, thread};
use crate::{AUTH_PARTS, VERSION, MIN_SUPPORTED_VERSION, MAX_SUPPORTED_VERSION, ClientCodes, ServerCodes};

static HOST: &str = "0.0.0.0:20900";
static CONTROL_ENABLED: bool = true;
static CONTROL_PASSWORD: &str = "RH9AuKR/48MWc0FePT658g==";

pub struct Request {
    pub id: u64,
}

pub struct Client {
    pub stream: TcpStream,
    pub id: u128,
    pub version: u64,
    pub requests: Vec<Request>,
    pub request_id: u64,
    pub is_authorized: bool,
    pub is_controlled: bool,
}

impl Client {
    pub fn new(stream: TcpStream) -> Client {
        Client { stream, id: u128::MAX, version: u64::MAX, requests: Vec::new(), request_id: 0, is_authorized: false, is_controlled: true }
    }
}

fn handle_packet(client: &mut Client, code: ClientCodes) -> bool {
    match code {
        ClientCodes::CAuth => {
            let mut data1 = [0u8; 8];
            client.stream.read(&mut data1).unwrap();
            let version = u64::from_be_bytes(data1);

            if version < MIN_SUPPORTED_VERSION || version > MAX_SUPPORTED_VERSION {
                let mut msg: Vec<u8> = Vec::new();
                msg.push(ServerCodes::SEAuthVersion as u8);
                msg.extend(MIN_SUPPORTED_VERSION.to_be_bytes());
                msg.extend(MAX_SUPPORTED_VERSION.to_be_bytes());
                client.stream.write(&msg).unwrap();
                return true;
            }

            let mut data2 = [0u8; 8];
            client.stream.read(&mut data2).unwrap();
            let auth_part1 = u64::from_be_bytes(data2);

            if auth_part1 != AUTH_PARTS[0] {
                let msg = [ServerCodes::SEAuthPart as u8; 1];
                client.stream.write(&msg).unwrap();
                return true;
            }

            let mut msg: Vec<u8> = Vec::new();
            msg.push(ServerCodes::SAuth as u8);
            msg.extend(VERSION.to_be_bytes());
            msg.extend(AUTH_PARTS[1].to_be_bytes());
            client.stream.write(&msg).unwrap();
            client.is_authorized = true;
        },
        ClientCodes::CAuthOK => {
            let mut data = [0u8; 16];
            client.stream.read(&mut data).unwrap();
            let uid = u128::from_be_bytes(data);
            println!("Client authorized: {}", uid);
        }
        _ => {
            todo!()
        }
    }

    false
}

fn handle_client(client: &mut Client) {
    let mut code = [0u8; 1];

    loop {
        match client.stream.read(&mut code) {
            Ok(_) => {
                if code[0] >= 0x80 {
                    println!("Invalid code from client (>= 0x80)");
                    continue;
                }

                match ClientCodes::try_from(code[0]) {
                    Ok(code) => {
                        if handle_packet(client, code) {
                            break;
                        }
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                        continue;
                    }
                }
            },
            Err(err) => {
                println!("Error: {}", err);
                return;
            }
        }
    }

    client.stream.flush().unwrap();
    client.stream.shutdown(Shutdown::Both).unwrap();
}

pub fn start_server() {
    let listener = TcpListener::bind(HOST).unwrap();
    println!("Server started on '{}'", HOST);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                stream.set_nodelay(true).unwrap();
                stream.set_nonblocking(false).unwrap();
                println!("New client connected: {}", stream.peer_addr().unwrap());
                thread::spawn(|| {
                    handle_client(&mut Client::new(stream));
                });
            }
            Err(err) => {
                println!("Error: {}", err);
            }
        }
    }
}
