extern crate machine_uid;
use core::time::Duration;
use std::{net::{TcpStream, Shutdown}, io::{Write, Read, Error, ErrorKind}, thread::sleep};

static HOST: &str = "127.0.0.1:20900";
// Two secure-random generated 8-byte unsigned integers
// Should equals to client parts
static AUTH_PARTS: [u64; 2] = [0xf61388842cb1b921, 0x9a0c109ca878b305];
static CONNECT_INTERVAL: u64 = 5000;
static VERSION: u64 = 0;
static MIN_SUPPORTED_VERSION: u64 = 0;
static MAX_SUPPORTED_VERSION: u64 = 0;

#[repr(u8)]
enum ClientCodes {
    CAuth = 0x00,
    CEAuthPart = 0x01,
    CEAuthVersion = 0x02,
    CAuthOK = 0x03,
    RTestEcho = 0x04,
    ROK = 0x05,
    RFail = 0x06,
    RFailText = 0x07,
    ROKText = 0x08,
    RAborted = 0x09,
    RBool = 0x0A,
    CControl = 0x70,
    CControlAll = 0x71,
    CControlList = 0x72,
    CControlOne = 0x73,
    CControlMany = 0x74,
}

#[repr(u8)]
enum ServerCodes {
    SAuth = 0x00,
    SEAuthPart = 0x01,
    SEAuthVersion = 0x02,
    MTest = 0x03,
    MGui = 0x04,
    MAbort = 0x05,
    MGuiYesNo = 0x06,
    MMoveCursor = 0x07,
    MMoveCursorRel = 0x08,
    MTypeKeyboard = 0x09,
    MClipboardGet = 0x0A,
    MClipboardSet = 0x0B,
    SControlOK = 0x70,
    SEControlOff = 0x71,
    SEControlPass = 0x72,
    SControlPacket = 0x73,
}

impl TryFrom<u8> for ServerCodes {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::SAuth),
            0x01 => Ok(Self::SEAuthPart),
            0x02 => Ok(Self::SEAuthVersion),
            0x03 => Ok(Self::MTest),
            0x04 => Ok(Self::MGui),
            0x05 => Ok(Self::MAbort),
            0x06 => Ok(Self::MGuiYesNo),
            0x07 => Ok(Self::MMoveCursor),
            0x08 => Ok(Self::MMoveCursorRel),
            0x09 => Ok(Self::MTypeKeyboard),
            0x0A => Ok(Self::MClipboardGet),
            0x0B => Ok(Self::MClipboardSet),
            0x70 => Ok(Self::SControlOK),
            0x71 => Ok(Self::SEControlOff),
            0x72 => Ok(Self::SEControlPass),
            0x73 => Ok(Self::SControlPacket),
            _ => Err(Error::new(ErrorKind::InvalidData, "Code not in enum")),
        }
    }
}

fn main() {
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
