extern crate machine_uid;
use std::{net::{TcpStream, Shutdown}, io::{Write, Read, Error, ErrorKind}, thread::sleep, time::Duration};
use crate::{AUTH_PARTS, VERSION, MIN_SUPPORTED_VERSION, MAX_SUPPORTED_VERSION, ClientCodes, ServerCodes};

static HOST: &str = "127.0.0.1:20900";
static CONNECT_INTERVAL: u64 = 5000;

pub fn start_client() {
    let mut stream: TcpStream;
    let mut started: bool = false;

    loop {
        if started {
            sleep(Duration::from_millis(CONNECT_INTERVAL));
        } else {
            started = true;
        }

        match TcpStream::connect(HOST) {
            Ok(server) => {
                stream = server;
                stream.set_nodelay(true).unwrap();
                println!("Connected to server '{}'", HOST);

                let mut msg: Vec<u8> = Vec::new();
                msg.push(ClientCodes::CAuth as u8);
                msg.extend(VERSION.to_be_bytes());
                msg.extend(AUTH_PARTS[0].to_be_bytes());
                stream.write(&msg).unwrap();

                let mut server_code = [0u8; 1];
                stream.read_exact(&mut server_code).unwrap();
                let code: ServerCodes = server_code[0].try_into().unwrap();

                match code {
                    ServerCodes::SEAuthPart => {
                        println!("Auth part mismatch");
                    }
                    ServerCodes::SEAuthVersion => {
                        let mut data1 = [0u8; 8];
                        stream.read_exact(&mut data1).unwrap();
                        let min_version = u64::from_be_bytes(data1);
                        let mut data2 = [0u8; 8];
                        stream.read_exact(&mut data2).unwrap();
                        let max_version = u64::from_be_bytes(data2);
                        println!("Incorrect version {}. Expected from {} to {}", VERSION, min_version, max_version);
                    }
                    ServerCodes::SAuth => {
                        let mut data1 = [0u8; 8];
                        stream.read_exact(&mut data1).unwrap();
                        let version = u64::from_be_bytes(data1);

                        if version < MIN_SUPPORTED_VERSION || version > MAX_SUPPORTED_VERSION {
                            let mut msg: Vec<u8> = Vec::new();
                            msg.push(ClientCodes::CEAuthVersion as u8);
                            msg.extend(MIN_SUPPORTED_VERSION.to_be_bytes());
                            msg.extend(MAX_SUPPORTED_VERSION.to_be_bytes());
                            stream.write(&msg).unwrap();
                        } else {
                            let mut data2 = [0u8; 8];
                            stream.read_exact(&mut data2).unwrap();
                            let auth_part2 = u64::from_be_bytes(data2);

                            if auth_part2 != AUTH_PARTS[1] {
                                let msg = [ClientCodes::CEAuthPart as u8; 1];
                                stream.write(&msg).unwrap();
                            } else {
                                let mut msg: Vec<u8> = Vec::new();
                                msg.push(ClientCodes::CAuthOK as u8);
                                let value: u128 = match machine_uid::get() {
                                    Ok(uid_str) => {
                                        match u128::from_str_radix(&uid_str, 16) {
                                            Ok(uid) => {
                                                uid
                                            }
                                            Err(_) => {
                                                0u128
                                            }
                                        }
                                    },
                                    Err(_) => {
                                        0u128
                                    }
                                };
                                msg.extend(value.to_be_bytes());
                                stream.write(&msg).unwrap();
                                println!("Authorized");
                                continue;
                            }
                        }
                    }
                    _ => {
                        todo!()
                    }
                }
            }
            Err(err) => {
                println!("Connection to server error: {}", err);
                continue;
            }
        }

        stream.flush().unwrap();
        stream.shutdown(Shutdown::Both).unwrap();
    }
}
