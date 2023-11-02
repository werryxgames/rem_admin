use std::{net::{TcpStream, TcpListener, Shutdown}, io::{Read, Write, ErrorKind}, thread, sync::{Arc, Mutex}};

use crate::{AUTH_PARTS, VERSION, MIN_SUPPORTED_VERSION, MAX_SUPPORTED_VERSION, ClientCodes, ServerCodes};

static HOST: &str = "0.0.0.0:20900";
static CONTROL_ENABLED: bool = true;
static CONTROL_PASSWORD: &str = "RH9AuKR/48MWc0FePT658g==";
static CLIENTS: Mutex<Vec<Arc<Mutex<Client>>>> = Mutex::new(Vec::new());

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

    pub fn request(&mut self, req: Request) {
        self.requests.push(req);
        self.request_id += 1;
    }
}

fn handle_packet(client: Arc<Mutex<Client>>, code: ClientCodes) -> bool {
    match code {
        ClientCodes::CAuth => {
            let mut data1 = [0u8; 8];
            client.lock().unwrap().stream.read(&mut data1).unwrap();
            let version = u64::from_be_bytes(data1);

            if version < MIN_SUPPORTED_VERSION || version > MAX_SUPPORTED_VERSION {
                let mut msg: Vec<u8> = Vec::new();
                msg.push(ServerCodes::SEAuthVersion as u8);
                msg.extend(MIN_SUPPORTED_VERSION.to_be_bytes());
                msg.extend(MAX_SUPPORTED_VERSION.to_be_bytes());
                client.lock().unwrap().stream.write(&msg).unwrap();
                return true;
            }

            let mut data2 = [0u8; 8];
            client.lock().unwrap().stream.read(&mut data2).unwrap();
            let auth_part1 = u64::from_be_bytes(data2);

            if auth_part1 != AUTH_PARTS[0] {
                let msg = [ServerCodes::SEAuthPart as u8; 1];
                client.lock().unwrap().stream.write(&msg).unwrap();
                return true;
            }

            let mut msg: Vec<u8> = Vec::new();
            msg.push(ServerCodes::SAuth as u8);
            msg.extend(VERSION.to_be_bytes());
            msg.extend(AUTH_PARTS[1].to_be_bytes());
            client.lock().unwrap().stream.write(&msg).unwrap();
            client.lock().unwrap().stream.flush().unwrap();
            client.lock().unwrap().version = version;
            client.lock().unwrap().is_authorized = true;
        },
        ClientCodes::CAuthOK => {
            let mut data = [0u8; 16];
            client.lock().unwrap().stream.read(&mut data).unwrap();
            let uid = u128::from_be_bytes(data);
            client.lock().unwrap().id = uid;
            println!("Client authorized: {}", uid);
        }
        _ => {
            todo!()
        }
    }

    false
}

fn handle_client(client: Arc<Mutex<Client>>) {
    let mut code = [0u8; 1];

    loop {
        let result = client.lock().unwrap().stream.read(&mut code);
        match result {
            Ok(_) => {
                if code[0] >= 0x80 {
                    println!("Invalid code from client (>= 0x80)");
                    continue;
                }

                match ClientCodes::try_from(code[0]) {
                    Ok(code) => {
                        if handle_packet(client.clone(), code) {
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
                if err.kind() == ErrorKind::WouldBlock {
                    continue;
                }

                println!("Error: {}", err);
                return;
            }
        }
    }

    client.lock().unwrap().stream.flush().unwrap();
    client.lock().unwrap().stream.shutdown(Shutdown::Both).unwrap();
}

#[cfg(feature = "controller-cli")]
pub fn controller_start(clients: &Mutex<Vec<Arc<Mutex<Client>>>>) {
    use crate::controller_cli::controller_cli_start;
    controller_cli_start(clients);
}

#[cfg(not(feature = "controller-cli"))]
pub fn controller_start(_clients: Mutex<Vec<Mutex<Client>>>) {
    loop {}
}

pub fn start_server() {
    thread::spawn(|| {
        let listener = TcpListener::bind(HOST).unwrap();

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    stream.set_nodelay(true).unwrap();
                    stream.set_nonblocking(true).unwrap();
                    println!("New client connected: {}", stream.peer_addr().unwrap());
                    thread::spawn(move || {
                        let client = Arc::new(Mutex::new(Client::new(stream)));
                        CLIENTS.lock().unwrap().push(client.clone());
                        handle_client(client);
                    });
                }
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        }
    });
    println!("Server started on '{}'", HOST);
    controller_start(&CLIENTS);
}
