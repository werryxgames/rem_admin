use gtk4::glib;
use std::{net::{TcpStream, TcpListener, Shutdown}, io::{Read, Write, ErrorKind}, thread, sync::{Arc, Mutex}};
use gtk4::glib::clone;

use crate::{AUTH_PARTS, VERSION, MIN_SUPPORTED_VERSION, MAX_SUPPORTED_VERSION, ClientCodes, ServerCodes};

static HOST: &str = "0.0.0.0:20900";
// static CONTROL_ENABLED: bool = true;
// static CONTROL_PASSWORD: &str = "RH9AuKR/48MWc0FePT658g==";
static NEXT_INDEX: Mutex<u64> = Mutex::new(1u64);

#[derive(Clone)]
pub struct Request {
    pub id: u64,
}

#[derive(Clone)]
pub struct Client {
    pub id: u128,
    pub index: u64,
    pub version: u64,
    pub request_id: u64,
    pub stream: Arc<Mutex<TcpStream>>,
    pub requests: Arc<Mutex<Vec<Request>>>,
    pub is_authorized: bool,
    pub is_controlled: bool,
}

impl Client {
    pub fn new(stream: TcpStream) -> Client {
        let mut index = NEXT_INDEX.lock().unwrap();
        *index = index.overflowing_add(1).0;
        Client { index: index.overflowing_sub(1).0, stream: Arc::new(Mutex::new(stream)), id: 0, version: u64::MAX, requests: Arc::new(Mutex::new(Vec::new())), request_id: 0, is_authorized: false, is_controlled: true }
    }

    pub fn request(&mut self) {
        self.requests.lock().unwrap().push(Request { id: self.request_id });
        self.request_id += 1;
    }
}

fn handle_packet(clients: Arc<Mutex<Vec<Client>>>, index: u64, code: ClientCodes) -> bool {
    let mut stream_option: Option<Arc<Mutex<TcpStream>>> = None;

    for client in clients.lock().unwrap().iter() {
        if client.index == index {
            stream_option = Some(client.stream.clone());
            break;
        }
    }

    if let Some(stream_m) = stream_option {
        println!("Handling packet: {:?}", code);
        let mut stream = stream_m.lock().unwrap();
        match code {
            ClientCodes::CAuth => {
                let mut data1 = [0u8; 8];
                stream.read_exact(&mut data1).unwrap();
                let version = u64::from_be_bytes(data1);

                if version < MIN_SUPPORTED_VERSION || version > MAX_SUPPORTED_VERSION {
                    let mut msg: Vec<u8> = Vec::new();
                    msg.push(ServerCodes::SEAuthVersion as u8);
                    msg.extend(MIN_SUPPORTED_VERSION.to_be_bytes());
                    msg.extend(MAX_SUPPORTED_VERSION.to_be_bytes());
                    stream.write_all(&msg).unwrap();
                    return true;
                }

                let mut data2 = [0u8; 8];
                stream.read_exact(&mut data2).unwrap();
                let auth_part1 = u64::from_be_bytes(data2);

                if auth_part1 != AUTH_PARTS[0] {
                    let msg = [ServerCodes::SEAuthPart as u8; 1];
                    stream.write_all(&msg).unwrap();
                    return true;
                }

                let mut msg: Vec<u8> = Vec::new();
                msg.push(ServerCodes::SAuth as u8);
                msg.extend(VERSION.to_be_bytes());
                msg.extend(AUTH_PARTS[1].to_be_bytes());
                stream.write_all(&msg).unwrap();
                stream.flush().unwrap();

                for client in clients.lock().unwrap().iter_mut() {
                    if client.index == index {
                        client.version = version;
                        client.is_authorized = true;
                        break;
                    }
                }
            }
            ClientCodes::CEAuthPart => {
                println!("Invalid authorization part");
            }
            ClientCodes::CEAuthVersion => {
                let mut data1 = [0u8; 8];
                stream.read_exact(&mut data1).unwrap();
                let min_version = u64::from_be_bytes(data1);
                let mut data2 = [0u8; 8];
                stream.read_exact(&mut data2).unwrap();
                let max_version = u64::from_be_bytes(data2);
                println!("Unsupported server version: should be between {} and {}", min_version, max_version);
            }
            ClientCodes::CAuthOK => {
                let mut data = [0u8; 16];
                stream.read_exact(&mut data).unwrap();
                let uid = u128::from_be_bytes(data);

                for client in clients.lock().unwrap().iter_mut() {
                    if client.index == index {
                        client.id = uid;
                        break;
                    }
                }

                println!("Client authorized: {}", uid);
            }
            ClientCodes::RTestEcho => {
                let mut data = [0u8; 4];
                stream.read_exact(&mut data).unwrap();
                let number = u32::from_be_bytes(data);
                println!("Unhandled test response: {}", number);
            }
            ClientCodes::ROk => {
                let mut data = [0u8; 8];
                stream.read_exact(&mut data).unwrap();
                let packet_id = u64::from_be_bytes(data);
                println!("Packet with id {} returned successful result", packet_id);
            }
            ClientCodes::RFail => {
                let mut data = [0u8; 8];
                stream.read_exact(&mut data).unwrap();
                let packet_id = u64::from_be_bytes(data);
                println!("Packet with id {} returned non-successful result", packet_id);
            }
            ClientCodes::RFailText => {
                let mut data1 = [0u8; 4];
                stream.read_exact(&mut data1).unwrap();
                let mut data2 = vec![0u8; u32::from_be_bytes(data1) as usize];
                stream.read_exact(&mut data2).unwrap();
                let mut data3 = [0u8; 8];
                stream.read_exact(&mut data3).unwrap();
                let packet_id = u64::from_be_bytes(data3);
                println!("Packet with id {} failed with \"{}\"", packet_id, String::from_utf8(data2).unwrap())
            }
            ClientCodes::ROkText => {
                let mut data1 = [0u8; 4];
                stream.read_exact(&mut data1).unwrap();
                let mut data2 = vec![0u8; u32::from_be_bytes(data1) as usize];
                stream.read_exact(&mut data2).unwrap();
                let mut data3 = [0u8; 8];
                stream.read_exact(&mut data3).unwrap();
                let packet_id = u64::from_be_bytes(data3);
                println!("Packet with id {} returned \"{}\"", packet_id, String::from_utf8(data2).unwrap())
            }
            ClientCodes::RAborted => {
                let mut data = [0u8; 8];
                stream.read_exact(&mut data).unwrap();
                let cmd_id = u64::from_be_bytes(data);
                println!("Unhandled abort response: {}", cmd_id);
            }
            ClientCodes::RBool => {
                let mut data = [0u8; 8];
                stream.read_exact(&mut data).unwrap();
                let packet_id = u64::from_be_bytes(data);
                let mut data2 = [0u8; 1];
                stream.read_exact(&mut data2).unwrap();
                let result = u8::from_be_bytes(data2) != 0;

                if result {
                    println!("Packet with id {} returned true", packet_id);
                } else {
                    println!("Packet with id {} returned false", packet_id);
                }
            }
            ClientCodes::RNotAborted => {
                let mut data = [0u8; 8];
                stream.read_exact(&mut data).unwrap();
                let cmd_id = u64::from_be_bytes(data);
                let mut data2 = [0u8; 1];
                stream.read_exact(&mut data2).unwrap();
                let executed = u8::from_be_bytes(data2) != 0;
                println!("Unhandled abort fail: {}, {}", cmd_id, executed);
            }
            ClientCodes::RInt => {
                let mut data = [0u8; 8];
                stream.read_exact(&mut data).unwrap();
                let cmd_id = u64::from_be_bytes(data);
                let mut data2 = [0u8; 4];
                stream.read_exact(&mut data2).unwrap();
                let int = u32::from_be_bytes(data2);
                println!("Packet with id {} returned {}", cmd_id, int);
            }
            ClientCodes::RBytes => {
                let mut data = [0u8; 8];
                stream.read_exact(&mut data).unwrap();
                let cmd_id = u64::from_be_bytes(data);
                let mut data2 = [0u8; 4];
                stream.read_exact(&mut data2).unwrap();
                let vec_len = u32::from_be_bytes(data2);
                let mut data3 = vec![0u8; vec_len as usize];
                stream.read_exact(&mut data3).unwrap();
                println!("Packet with id {} returned unhandled bytes", cmd_id);
            }
            ClientCodes::RStdOutErr => {
                let mut data = [0u8; 8];
                stream.read_exact(&mut data).unwrap();
                let cmd_id = u64::from_be_bytes(data);
                let mut data2 = [0u8; 4];
                stream.read_exact(&mut data2).unwrap();
                let vec_len = u32::from_be_bytes(data2);
                let mut stdout = vec![0u8; vec_len as usize];
                stream.read_exact(&mut stdout).unwrap();
                let mut data4 = [0u8; 4];
                stream.read_exact(&mut data4).unwrap();
                let vec_len2 = u32::from_be_bytes(data4);
                let mut stderr = vec![0u8; vec_len2 as usize];
                stream.read_exact(&mut stderr).unwrap();
                println!("Packet with id {} returned result", cmd_id);
                println!("Stdout: {}", String::from_utf8_lossy(&stdout));
                println!("Stderr: {}", String::from_utf8_lossy(&stderr));
            }
            ClientCodes::RStdOutErrFail => {
                let mut data = [0u8; 8];
                stream.read_exact(&mut data).unwrap();
                let cmd_id = u64::from_be_bytes(data);
                let mut data2 = [0u8; 4];
                stream.read_exact(&mut data2).unwrap();
                let vec_len = u32::from_be_bytes(data2);
                let mut stdout = vec![0u8; vec_len as usize];
                stream.read_exact(&mut stdout).unwrap();
                let mut data4 = [0u8; 4];
                stream.read_exact(&mut data4).unwrap();
                let vec_len2 = u32::from_be_bytes(data4);
                let mut stderr = vec![0u8; vec_len2 as usize];
                stream.read_exact(&mut stderr).unwrap();
                let mut data6 = [0u8; 4];
                stream.read_exact(&mut data6).unwrap();
                let code = u32::from_be_bytes(data6);
                println!("Packet with id {} failed with code {} and result", cmd_id, code);
                println!("Stdout: {}", String::from_utf8_lossy(&stdout));
                println!("Stderr: {}", String::from_utf8_lossy(&stderr));
            }
            _ => {
                todo!()
            }
        }

        return false;
    }

    true
}

fn handle_client(clients: Arc<Mutex<Vec<Client>>>, index: u64) {
    let mut code = [0u8; 1];
    let mut stream_option: Option<Arc<Mutex<TcpStream>>> = None;

    for client in clients.lock().unwrap().iter() {
        if client.index == index {
            stream_option = Some(client.stream.clone());
            break;
        }
    }

    if let Some(stream_m) = stream_option {
        loop {
            let result = stream_m.lock().unwrap().read_exact(&mut code);
            match result {
                Ok(_) => {
                    if code[0] >= 0x80 {
                        println!("Invalid code from client (>= 0x80)");
                        continue;
                    }

                    match ClientCodes::try_from(code[0]) {
                        Ok(code) => {
                            if handle_packet(clients.clone(), index, code) {
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

        let mut stream = stream_m.lock().unwrap();
        stream.flush().unwrap();
        stream.shutdown(Shutdown::Both).unwrap();
    }
}

#[cfg(feature = "controller-cli")]
#[cfg(feature = "controller-gui")]
pub fn controller_start(clients: Arc<Mutex<Vec<Client>>>) {
    use crate::controller_gui::controller_gui_start;
    controller_gui_start(clients);
}

#[cfg(feature = "controller-cli")]
#[cfg(not(feature = "controller-gui"))]
pub fn controller_start(clients: Arc<Mutex<Vec<Client>>>) {
    use crate::controller_cli::controller_cli_start;
    controller_cli_start(clients);
}

#[cfg(not(feature = "controller-cli"))]
#[cfg(feature = "controller-gui")]
pub fn controller_start(clients: Arc<Mutex<Vec<Client>>>) {
    use crate::controller_gui::controller_gui_start;
    controller_gui_start(clients);
}

#[cfg(not(feature = "controller-cli"))]
#[cfg(not(feature = "controller-gui"))]
pub fn controller_start(_clients: Arc<Mutex<Vec<Client>>>) {
    loop {}
}

pub fn start_server() {
    let clients_m: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));
    thread::spawn(clone!(@weak clients_m => move || {
        let listener = TcpListener::bind(HOST).unwrap();

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    stream.set_nodelay(true).unwrap();
                    stream.set_nonblocking(true).unwrap();
                    println!("New client connected: {}", stream.peer_addr().unwrap());
                    thread::spawn(clone!(@weak clients_m => move || {
                        let client = Client::new(stream);
                        let index = client.index;
                        {
                            clients_m.lock().unwrap().push(client);
                        }
                        handle_client(clients_m, index);
                    }));
                }
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        }
    }));
    println!("Server started on '{}'", HOST);
    controller_start(clients_m);
}
