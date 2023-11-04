use std::{net::{TcpStream, Shutdown}, io::{Write, Read, ErrorKind, Error}, thread::{sleep, self}, time::Duration, env, process::{Command, Child, exit}, sync::{Mutex, Arc}, fs};
use crate::{AUTH_PARTS, VERSION, MIN_SUPPORTED_VERSION, MAX_SUPPORTED_VERSION, ClientCodes, ServerCodes};
use enigo::{Enigo, MouseControllable, KeyboardControllable};
use gtk::prelude::DialogExt;
use rand::Rng;
use sysinfo::{System, SystemExt, CpuExt};

static HOST: &str = "127.0.0.1:20900";
static CONNECT_INTERVAL: u64 = 5000;
static ARGV_DIALOG: &str = "Zk62lYNU1paEiNxk5DVu";
static ARGV_DIALOG_YESNO: &str = "dxvYc4DVJnBetKI4ImyE";
static REQUESTS: Mutex<Vec<Request>> = Mutex::new(Vec::new());
static REQUEST_ID: Mutex<u64> = Mutex::new(0);

pub struct Request {
    pub process: Option<Arc<Mutex<Child>>>,
    pub id: u64,
}

impl Request {
    pub fn new(process: Option<Arc<Mutex<Child>>>) -> u64 {
        let lock: &mut u64 = &mut REQUEST_ID.lock().unwrap();
        let id = lock.overflowing_add(1);
        *lock = id.0;
        let process_id = id.0.overflowing_sub(1).0;
        REQUESTS.lock().unwrap().push(Request { process: process, id: process_id });
        process_id
    }

    pub fn abort(id: u64) -> Option<bool> {
        let mut remove: Option<usize> = None;
        let mut status: Option<bool> = None;

        {
            let mut requests = REQUESTS.lock().unwrap();

            for request_tuple in requests.iter_mut().enumerate() {
                let request = request_tuple.1;

                if request.id == id && request.process.is_some() {
                    let process_m = request.process.as_ref().unwrap();
                    let mut process = process_m.lock().unwrap();
                    let exited = process.try_wait().unwrap().is_some();
                    process.kill().unwrap();
                    remove = Some(request_tuple.0);
                    status = Some(!exited);
                    break;
                }
            }
        }

        if remove.is_some() {
            REQUESTS.lock().unwrap().remove(remove.unwrap());
        }

        status
    }
}

/// Returns randomly generated unique id for this machine.
/// Can be regenerated or even changed to match specific id.
pub fn get_machine_id() -> u128 {
    let mut path = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    path.push("machine_id.dat");
    let file = fs::read(path.as_path());

    if file.is_err() {
        let rand_num: u128 = rand::thread_rng().gen();
        let _ = fs::write(path, rand_num.to_be_bytes());
        rand_num
    } else {
        let mut bytes = file.unwrap();

        while bytes.len() < 16 {
            bytes.push(0);
        }

        let mut mid_bytes = [0u8; 16];

        for byte in bytes[..16].iter().enumerate() {
            mid_bytes[byte.0] = *byte.1;
        }

        u128::from_be_bytes(mid_bytes)
    }
}

pub fn show_dialog(stream: Arc<Mutex<TcpStream>>, title: String, message: String) {
    let child = Command::new(env::current_exe().unwrap()).args([ARGV_DIALOG, ARGV_DIALOG, ARGV_DIALOG, ARGV_DIALOG, &title, &message, ARGV_DIALOG]).spawn().unwrap();
    let child_m = Arc::new(Mutex::new(child));
    let process_id = Request::new(Some(child_m.clone()));
    thread::spawn(move || {
        let mut code_option;

        loop {
            code_option = child_m.lock().unwrap().try_wait().unwrap();

            if code_option.is_some() {
                break;
            }
        }

        let code = code_option.unwrap().code();

        if code.is_none() || code.unwrap() == 0 {
            let mut buf: Vec<u8> = Vec::new();
            buf.push(ClientCodes::ROK as u8);
            buf.extend(process_id.to_be_bytes());
            stream.lock().unwrap().write(&buf).unwrap();
        } else {
            let mut buf: Vec<u8> = Vec::new();
            buf.push(ClientCodes::RFail as u8);
            buf.extend(process_id.to_be_bytes());
            stream.lock().unwrap().write(&buf).unwrap();
        }
    });
}

pub fn show_dialog_yesno(stream: Arc<Mutex<TcpStream>>, title: String, message: String) {
    let child = Command::new(env::current_exe().unwrap()).args([ARGV_DIALOG_YESNO, ARGV_DIALOG_YESNO, ARGV_DIALOG_YESNO, ARGV_DIALOG_YESNO, &title, &message, ARGV_DIALOG_YESNO]).spawn().unwrap();
    let child_m = Arc::new(Mutex::new(child));
    let process_id = Request::new(Some(child_m.clone()));
    thread::spawn(move || {
        let mut code_option;
        
        loop {
            code_option = child_m.lock().unwrap().try_wait().unwrap();

            if code_option.is_some() {
                break;
            }
        }

        let code = code_option.unwrap().code();

        if code.is_none() {
            let mut buf: Vec<u8> = Vec::new();
            buf.push(ClientCodes::RFail as u8);
            buf.extend(process_id.to_be_bytes());
            stream.lock().unwrap().write(&buf).unwrap();
        } else {
            let ncode = code.unwrap();

            if ncode == 0 {
                let mut buf: Vec<u8> = Vec::new();
                buf.push(ClientCodes::RBool as u8);
                buf.extend(process_id.to_be_bytes());
                buf.push(false as u8);
                stream.lock().unwrap().write(&buf).unwrap();
            } else if ncode == 1 {
                let mut buf: Vec<u8> = Vec::new();
                buf.push(ClientCodes::RBool as u8);
                buf.extend(process_id.to_be_bytes());
                buf.push(true as u8);
                stream.lock().unwrap().write(&buf).unwrap();
            } else {
                let mut buf: Vec<u8> = Vec::new();
                buf.push(ClientCodes::RFail as u8);
                buf.extend(process_id.to_be_bytes());
                stream.lock().unwrap().write(&buf).unwrap();
            }
        }
    });
}

pub fn start_client() {
    let argv: Vec<String> = env::args().collect();

    if argv.len() == 8 {
        if argv[3] == ARGV_DIALOG {
            gtk::init().unwrap();
            gtk::MessageDialog::builder()
            .title(argv[5].clone())
            .text(argv[6].clone())
            .buttons(gtk::ButtonsType::Ok)
            .build().run();
        } else if argv[3] == ARGV_DIALOG_YESNO {
            gtk::init().unwrap();
            exit(i32::from(gtk::MessageDialog::builder()
            .title(argv[5].clone())
            .text(argv[6].clone())
            .buttons(gtk::ButtonsType::YesNo)
            .build().run() == gtk::ResponseType::Yes));
        }

        return;
    }

    let stream: Arc<Mutex<TcpStream>>;
    let mut started: bool = false;

    loop {
        if started {
            sleep(Duration::from_millis(CONNECT_INTERVAL));
        } else {
            started = true;
        }

        match TcpStream::connect(HOST) {
            Ok(server) => {
                stream = Arc::new(Mutex::new(server));
                stream.lock().unwrap().set_nodelay(true).unwrap();
                stream.lock().unwrap().set_nonblocking(true).unwrap();
                println!("Connected to server '{}'", HOST);

                let mut msg: Vec<u8> = Vec::new();
                msg.push(ClientCodes::CAuth as u8);
                msg.extend(VERSION.to_be_bytes());
                msg.extend(AUTH_PARTS[0].to_be_bytes());
                stream.lock().unwrap().write(&msg).unwrap();

                let mut server_code = [0u8; 1];

                loop {
                    loop {
                        match stream.lock().unwrap().read_exact(&mut server_code) {
                            Ok(_) => {
                                break;
                            }
                            Err(err) => {
                                if err.kind() == ErrorKind::WouldBlock {
                                    continue;
                                }

                                Err::<(), Error>(err).unwrap();
                            }
                        };
                    }

                    let code: ServerCodes = server_code[0].try_into().unwrap();

                    match code {
                        ServerCodes::SEAuthPart => {
                            println!("Auth part mismatch");
                        }
                        ServerCodes::SEAuthVersion => {
                            let mut data1 = [0u8; 8];
                            stream.lock().unwrap().read_exact(&mut data1).unwrap();
                            let min_version = u64::from_be_bytes(data1);
                            let mut data2 = [0u8; 8];
                            stream.lock().unwrap().read_exact(&mut data2).unwrap();
                            let max_version = u64::from_be_bytes(data2);
                            println!("Incorrect version {}. Expected from {} to {}", VERSION, min_version, max_version);
                        }
                        ServerCodes::SAuth => {
                            let mut data1 = [0u8; 8];
                            stream.lock().unwrap().read_exact(&mut data1).unwrap();
                            let version = u64::from_be_bytes(data1);

                            if version < MIN_SUPPORTED_VERSION || version > MAX_SUPPORTED_VERSION {
                                let mut msg: Vec<u8> = Vec::new();
                                msg.push(ClientCodes::CEAuthVersion as u8);
                                msg.extend(MIN_SUPPORTED_VERSION.to_be_bytes());
                                msg.extend(MAX_SUPPORTED_VERSION.to_be_bytes());
                                stream.lock().unwrap().write(&msg).unwrap();
                            } else {
                                let mut data2 = [0u8; 8];
                                stream.lock().unwrap().read_exact(&mut data2).unwrap();
                                let auth_part2 = u64::from_be_bytes(data2);

                                if auth_part2 != AUTH_PARTS[1] {
                                    let msg = [ClientCodes::CEAuthPart as u8; 1];
                                    stream.lock().unwrap().write(&msg).unwrap();
                                } else {
                                    let mut msg: Vec<u8> = Vec::new();
                                    msg.push(ClientCodes::CAuthOK as u8);
                                    let value: u128 = get_machine_id();
                                    msg.extend(value.to_be_bytes());
                                    stream.lock().unwrap().write(&msg).unwrap();
                                    println!("Authorized");
                                    continue;
                                }
                            }
                        }
                        ServerCodes::MTest => {
                            let mut buf = [0u8; 4];
                            stream.lock().unwrap().read_exact(&mut buf).unwrap();
                            let mut msg: Vec<u8> = Vec::new();
                            msg.push(ClientCodes::RTestEcho as u8);
                            msg.extend(buf);
                            stream.lock().unwrap().write(&msg).unwrap();
                        }
                        ServerCodes::MGui => {
                            let mut title_len = [0u8; 4];
                            stream.lock().unwrap().read_exact(&mut title_len).unwrap();
                            let mut title_bytes: Vec<u8> = vec![0u8; u32::from_be_bytes(title_len) as usize];
                            stream.lock().unwrap().read_exact(&mut title_bytes).unwrap();
                            let mut message_len = [0u8; 4];
                            stream.lock().unwrap().read_exact(&mut message_len).unwrap();
                            let mut message_bytes: Vec<u8> = vec![0u8; u32::from_be_bytes(message_len) as usize];
                            stream.lock().unwrap().read_exact(&mut message_bytes).unwrap();
                            let title = String::from_utf8(title_bytes).unwrap();
                            let message = String::from_utf8(message_bytes).unwrap();
                            show_dialog(stream.clone(), title, message);
                        }
                        ServerCodes::MGuiYesNo => {
                            let mut title_len = [0u8; 4];
                            stream.lock().unwrap().read_exact(&mut title_len).unwrap();
                            let mut title_bytes: Vec<u8> = vec![0u8; u32::from_be_bytes(title_len) as usize];
                            stream.lock().unwrap().read_exact(&mut title_bytes).unwrap();
                            let mut message_len = [0u8; 4];
                            stream.lock().unwrap().read_exact(&mut message_len).unwrap();
                            let mut message_bytes: Vec<u8> = vec![0u8; u32::from_be_bytes(message_len) as usize];
                            stream.lock().unwrap().read_exact(&mut message_bytes).unwrap();
                            let title = String::from_utf8(title_bytes).unwrap();
                            let message = String::from_utf8(message_bytes).unwrap();
                            show_dialog_yesno(stream.clone(), title, message);
                        }
                        ServerCodes::MAbort => {
                            let mut cmd_id_bytes = [0u8; 8];
                            stream.lock().unwrap().read_exact(&mut cmd_id_bytes).unwrap();
                            let cmd_id = u64::from_be_bytes(cmd_id_bytes);
                            println!("Request to abort {}", cmd_id);
                            let result = Request::abort(cmd_id);
                            println!("Done");
                            let mut msg: Vec<u8> = Vec::new();

                            if result.is_none() {
                                msg.push(ClientCodes::RNotAborted as u8);
                                msg.extend(cmd_id_bytes);
                                msg.push(false as u8);
                            } else {
                                if result.unwrap() {
                                    msg.push(ClientCodes::RAborted as u8);
                                    msg.extend(cmd_id_bytes);
                                } else {
                                    msg.push(ClientCodes::RNotAborted as u8);
                                    msg.extend(cmd_id_bytes);
                                    msg.push(true as u8);
                                }
                            }

                            stream.lock().unwrap().write(&msg).unwrap();
                        }
                        ServerCodes::MMoveCursor => {
                            let mut data1 = [0u8; 4];
                            stream.lock().unwrap().read_exact(&mut data1).unwrap();
                            let mut data2 = [0u8; 4];
                            stream.lock().unwrap().read_exact(&mut data2).unwrap();
                            let x = i32::from_be_bytes(data1);
                            let y = i32::from_be_bytes(data2);
                            Enigo::new().mouse_move_to(x, y);
                        }
                        ServerCodes::MMoveCursorRel => {
                            let mut data1 = [0u8; 4];
                            stream.lock().unwrap().read_exact(&mut data1).unwrap();
                            let mut data2 = [0u8; 4];
                            stream.lock().unwrap().read_exact(&mut data2).unwrap();
                            let x = i32::from_be_bytes(data1);
                            let y = i32::from_be_bytes(data2);
                            Enigo::new().mouse_move_relative(x, y);
                        }
                        ServerCodes::MTypeKeyboard => {
                            let mut data1 = [0u8; 4];
                            stream.lock().unwrap().read_exact(&mut data1).unwrap();
                            let mut data2 = vec![0u8; u32::from_be_bytes(data1) as usize];
                            stream.lock().unwrap().read_exact(&mut data2).unwrap();
                            let sequence = String::from_utf8(data2).unwrap();
                            Enigo::new().key_sequence_parse(&sequence);
                        }
                        ServerCodes::MClipboardGet => {
                            let code = Request::new(None);
                            let clipboard = terminal_clipboard::get_string().unwrap();
                            let mut msg: Vec<u8> = Vec::new();
                            msg.push(ClientCodes::ROKText as u8);
                            msg.extend((clipboard.len() as u32).to_be_bytes());
                            msg.extend(clipboard.as_bytes());
                            msg.extend(code.to_be_bytes());
                            stream.lock().unwrap().write(&msg).unwrap();
                        }
                        ServerCodes::MClipboardSet => {
                            let mut data1 = [0u8; 4];
                            stream.lock().unwrap().read_exact(&mut data1).unwrap();
                            let mut data2 = vec![0u8; u32::from_be_bytes(data1) as usize];
                            stream.lock().unwrap().read_exact(&mut data2).unwrap();
                            let clipboard = String::from_utf8(data2).unwrap();
                            println!("Content: {}", clipboard);
                            terminal_clipboard::set_string(clipboard).unwrap();
                        }
                        _ => {
                            todo!()
                        }
                    }
                }
            }
            Err(err) => {
                println!("Connection to server error: {}", err);
                continue;
            }
        };

        stream.lock().unwrap().flush().unwrap();
        stream.lock().unwrap().shutdown(Shutdown::Both).unwrap();
    }
}
