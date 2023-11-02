use std::{net::{TcpStream, TcpListener, Shutdown}, io::{Read, Error, ErrorKind, Write}, thread};

static HOST: &str = "0.0.0.0:20900";
// Two secure-random generated 8-byte unsigned integers
// Should equals to client parts
static AUTH_PARTS: [u64; 2] = [0xf61388842cb1b921, 0x9a0c109ca878b305];
static CONTROL_ENABLED: bool = true;
static CONTROL_PASSWORD: &str = "RH9AuKR/48MWc0FePT658g==";
static VERSION: u64 = 0;
static MIN_SUPPORTED_VERSION: u64 = 0;
static MAX_SUPPORTED_VERSION: u64 = 0;

#[repr(u8)]
#[derive(PartialEq, Eq)]
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

impl TryFrom<u8> for ClientCodes {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::CAuth),
            0x01 => Ok(Self::CEAuthPart),
            0x02 => Ok(Self::CEAuthVersion),
            0x03 => Ok(Self::CAuthOK),
            0x04 => Ok(Self::RTestEcho),
            0x05 => Ok(Self::ROK),
            0x06 => Ok(Self::RFail),
            0x07 => Ok(Self::RFailText),
            0x08 => Ok(Self::ROKText),
            0x09 => Ok(Self::RAborted),
            0x0A => Ok(Self::RBool),
            0x70 => Ok(Self::CControl),
            0x71 => Ok(Self::CControlAll),
            0x72 => Ok(Self::CControlList),
            0x73 => Ok(Self::CControlOne),
            0x74 => Ok(Self::CControlMany),
            _ => Err(Error::new(ErrorKind::InvalidData, "Code not in enum")),
        }
    }
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

fn main() {
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
