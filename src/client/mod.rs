use std::{net::TcpStream, io::{Write, Read, ErrorKind, self}, thread::{sleep, self}, time::Duration, env, process::{Command, Child, exit, Stdio}, sync::{Mutex, Arc}, fs};
use crate::{AUTH_PARTS, VERSION, MIN_SUPPORTED_VERSION, MAX_SUPPORTED_VERSION, ClientCodes, ServerCodes, command::parse_quotes};
use enigo::{Enigo, KeyboardControllable, MouseControllable};
use glib::clone;
use gtk4::{prelude::*, glib::{self, random_int}};
use jpeg_encoder::{Encoder, ColorType};
use rand::Rng;

static HOST: &str = "127.0.0.1:20900";
static CONNECT_INTERVAL: u64 = 5000;
static ARGV_DIALOG: &str = "Zk62lYNU1paEiNxk5DVu";
static ARGV_DIALOG_YESNO: &str = "dxvYc4DVJnBetKI4ImyE";
static ARGV_DIALOG_INPUT: &str = "7M52jCHyOtwbH4MQa4vA";
static REQUESTS: Mutex<Vec<Request>> = Mutex::new(Vec::new());
static REQUEST_ID: Mutex<u64> = Mutex::new(1);

pub struct Request {
    pub process: Option<Arc<Mutex<Child>>>,
    pub id: u64,
}

impl Request {
    pub fn add(process: Option<Arc<Mutex<Child>>>) -> u64 {
        let lock: &mut u64 = &mut REQUEST_ID.lock().unwrap();
        let id = lock.overflowing_add(1);
        *lock = id.0;
        let process_id = id.0.overflowing_sub(1).0;
        REQUESTS.lock().unwrap().push(Request { process, id: process_id });
        process_id
    }

    pub fn abort(id: u64) -> Option<bool> {
        let mut remove: Option<usize> = None;
        let mut status: Option<bool> = None;
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

        if let Some(rem) = remove {
            requests.remove(rem);
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

pub fn show_dialog(stream_m: Arc<Mutex<TcpStream>>, title: String, message: String) {
    let child = Command::new(env::current_exe().unwrap()).args([ARGV_DIALOG, ARGV_DIALOG, ARGV_DIALOG, ARGV_DIALOG, &title, &message, ARGV_DIALOG]).spawn().unwrap();
    let child_m = Arc::new(Mutex::new(child));
    let process_id = Request::add(Some(child_m.clone()));
    thread::spawn(move || {
        let mut code_option;

        loop {
            code_option = child_m.lock().unwrap().try_wait().unwrap();

            if code_option.is_some() {
                break;
            }
        }

        let code = code_option.unwrap().code();
        let mut stream = stream_m.lock().unwrap();

        if code.is_none() || code.unwrap() == 0 {
            let mut buf: Vec<u8> = Vec::new();
            buf.push(ClientCodes::ROk as u8);
            buf.extend(process_id.to_be_bytes());
            stream.write_all(&buf).unwrap();
        } else {
            let mut buf: Vec<u8> = Vec::new();
            buf.push(ClientCodes::RFail as u8);
            buf.extend(process_id.to_be_bytes());
            stream.write_all(&buf).unwrap();
        }
    });
}

pub fn show_dialog_yesno(stream_m: Arc<Mutex<TcpStream>>, title: String, message: String) {
    let child = Command::new(env::current_exe().unwrap()).args([ARGV_DIALOG_YESNO, ARGV_DIALOG_YESNO, ARGV_DIALOG_YESNO, ARGV_DIALOG_YESNO, &title, &message, ARGV_DIALOG_YESNO]).spawn().unwrap();
    let child_m = Arc::new(Mutex::new(child));
    let process_id = Request::add(Some(child_m.clone()));
    thread::spawn(move || {
        let mut code_option;
        
        loop {
            code_option = child_m.lock().unwrap().try_wait().unwrap();

            if code_option.is_some() {
                break;
            }
        }

        let code = code_option.unwrap().code();
        let mut stream = stream_m.lock().unwrap();

        if code.is_none() {
            let mut buf: Vec<u8> = Vec::new();
            buf.push(ClientCodes::RFail as u8);
            buf.extend(process_id.to_be_bytes());
            stream.write_all(&buf).unwrap();
        } else {
            let ncode = code.unwrap();

            if ncode == 0 {
                let mut buf: Vec<u8> = Vec::new();
                buf.push(ClientCodes::RBool as u8);
                buf.extend(process_id.to_be_bytes());
                buf.push(false as u8);
                stream.write_all(&buf).unwrap();
            } else if ncode == 1 {
                let mut buf: Vec<u8> = Vec::new();
                buf.push(ClientCodes::RBool as u8);
                buf.extend(process_id.to_be_bytes());
                buf.push(true as u8);
                stream.write_all(&buf).unwrap();
            } else {
                let mut buf: Vec<u8> = Vec::new();
                buf.push(ClientCodes::RFail as u8);
                buf.extend(process_id.to_be_bytes());
                stream.write_all(&buf).unwrap();
            }
        }
    });
}

pub fn show_dialog_input(stream_m: Arc<Mutex<TcpStream>>, title: String, message: String) {
    let child = Command::new(env::current_exe().unwrap())
    .args([ARGV_DIALOG_INPUT, ARGV_DIALOG_INPUT, ARGV_DIALOG_INPUT, ARGV_DIALOG_INPUT, &title, &message, ARGV_DIALOG_INPUT])
    .stdout(Stdio::piped())
    .spawn().unwrap();
    let child_m = Arc::new(Mutex::new(child));
    let process_id = Request::add(Some(child_m.clone()));
    thread::spawn(move || {
        let mut code_option;

        loop {
            code_option = child_m.lock().unwrap().try_wait().unwrap();

            if code_option.is_some() {
                break;
            }
        }

        let code = code_option.unwrap().code();
        let mut stream = stream_m.lock().unwrap();

        if code.is_none() {
            let mut buf: Vec<u8> = Vec::new();
            buf.push(ClientCodes::RFail as u8);
            buf.extend(process_id.to_be_bytes());
            stream.write_all(&buf).unwrap();
            println!("No code");
        } else {
            let ncode = code.unwrap();

            if ncode == 0 {
                let mut stdout: Vec<u8> = Vec::new();
                child_m.lock().unwrap().stdout.as_mut().unwrap().read_to_end(&mut stdout).unwrap();
                let mut stderr: Vec<u8> = Vec::new();
                child_m.lock().unwrap().stderr.as_mut().unwrap().read_to_end(&mut stderr).unwrap();

                let mut buf: Vec<u8> = Vec::new();
                buf.push(ClientCodes::RStdOutErr as u8);
                buf.extend(process_id.to_be_bytes());
                buf.extend((stdout.len() as u32).to_be_bytes());
                buf.extend(stdout);
                buf.extend((stderr.len() as u32).to_be_bytes());
                buf.extend(stderr);
                stream.write_all(&buf).unwrap();
                return;
            }

            let mut stdout: Vec<u8> = Vec::new();
            child_m.lock().unwrap().stdout.as_mut().unwrap().read_to_end(&mut stdout).unwrap();
            let mut stderr: Vec<u8> = Vec::new();
            child_m.lock().unwrap().stderr.as_mut().unwrap().read_to_end(&mut stderr).unwrap();

            let mut buf: Vec<u8> = Vec::new();
            buf.push(ClientCodes::RStdOutErrFail as u8);
            buf.extend(process_id.to_be_bytes());
            buf.extend((stdout.len() as u32).to_be_bytes());
            buf.extend(stdout);
            buf.extend((stderr.len() as u32).to_be_bytes());
            buf.extend(stderr);
            buf.extend(ncode.to_be_bytes());
            stream.write_all(&buf).unwrap();
        }
    });
}

pub fn execute_command(stream_m: Arc<Mutex<TcpStream>>, cmd: String) {
    let mut comargs: Vec<String> = Vec::new();

    if parse_quotes(&mut comargs, cmd.clone()) {
        // Send R_FAIL_TEXT
        return;
    }

    if comargs.is_empty() {
        Request::add(None);
        // Send R_OK
        return;
    }

    let child = Command::new(comargs[0].clone())
    .args(&comargs[1..])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn().unwrap();
    let child_m = Arc::new(Mutex::new(child));
    let process_id = Request::add(Some(child_m.clone()));
    thread::spawn(move || {
        let mut code_option;

        loop {
            code_option = child_m.lock().unwrap().try_wait().unwrap();

            if code_option.is_some() {
                break;
            }
        }

        let code = code_option.unwrap().code();
        let mut stream = stream_m.lock().unwrap();

        if let Some(ncode) = code {
            if ncode == 0 {
                let mut child = child_m.lock().unwrap();
                let mut stdout: Vec<u8> = Vec::new();

                if let Some(proc_stdout) = child.stdout.as_mut() {
                    proc_stdout.read_to_end(&mut stdout).unwrap();
                }

                let mut stderr: Vec<u8> = Vec::new();

                if let Some(proc_stderr) = child.stderr.as_mut() {
                    proc_stderr.read_to_end(&mut stderr).unwrap();
                }

                let mut buf: Vec<u8> = Vec::new();
                buf.push(ClientCodes::RStdOutErr as u8);
                buf.extend(process_id.to_be_bytes());
                buf.extend((stdout.len() as u32).to_be_bytes());
                buf.extend(stdout);
                buf.extend((stderr.len() as u32).to_be_bytes());
                buf.extend(stderr);
                stream.write_all(&buf).unwrap();
                return;
            }

            let mut child = child_m.lock().unwrap();
            let mut stdout: Vec<u8> = Vec::new();

            if let Some(proc_stdout) = child.stdout.as_mut() {
                proc_stdout.read_to_end(&mut stdout).unwrap();
            }

            let mut stderr: Vec<u8> = Vec::new();

            if let Some(proc_stderr) = child.stderr.as_mut() {
                proc_stderr.read_to_end(&mut stderr).unwrap();
            }

            let mut buf: Vec<u8> = Vec::new();
            buf.push(ClientCodes::RStdOutErrFail as u8);
            buf.extend(process_id.to_be_bytes());
            buf.extend((stdout.len() as u32).to_be_bytes());
            buf.extend(stdout);
            buf.extend((stderr.len() as u32).to_be_bytes());
            buf.extend(stderr);
            buf.extend(ncode.to_be_bytes());
            stream.write_all(&buf).unwrap();
        } else {
            let mut buf: Vec<u8> = vec![ClientCodes::RFail as u8];
            buf.extend(process_id.to_be_bytes());
            stream.write_all(&buf).unwrap();
        }
    });
}

pub fn start_client() {
    let argv: Vec<String> = env::args().collect();

    if argv.len() == 8 {
        if argv[3] == ARGV_DIALOG {
            let app = gtk4::Application::builder()
            .application_id("com.werryxgames.rem_admin.alert".to_owned() + &random_int().to_string())
            .build();
            let win_title = argv[5].clone();
            let text = argv[6].clone();
            app.connect_activate(move |app: &gtk4::Application| {
                let win = gtk4::ApplicationWindow::new(app);
                win.set_title(Some(&win_title));
                win.set_resizable(false);
                let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
                vbox.set_margin_start(8);
                vbox.set_margin_end(8);
                vbox.set_margin_top(8);
                vbox.set_margin_bottom(8);
                vbox.set_spacing(8);
                win.set_child(Some(&vbox));
                let label = gtk4::Label::builder()
                .label(&text)
                .wrap(true)
                .wrap_mode(gtk4::pango::WrapMode::WordChar)
                .width_request(150)
                .max_width_chars(50)
                .build();
                vbox.append(&label);
                let btn = gtk4::Button::with_label("OK");
                btn.connect_clicked(clone!(@weak win => move |_| {
                    win.close();
                }));
                vbox.append(&btn);
                win.present();
            });
            let args: [&str; 0] = [];
            app.run_with_args(&args);
            return;
        } else if argv[3] == ARGV_DIALOG_YESNO {
            static EXIT_CODE: Mutex<i32> = Mutex::new(2);
            // gtk4::init().unwrap();
            // exit(i32::from(gtk4::MessageDialog::builder()
            // .title(argv[5].clone())
            // .text(argv[6].clone())
            // .buttons(gtk4::ButtonsType::YesNo)
            // .build().run() == gtk4::ResponseType::Yes));
            let app = gtk4::Application::builder()
            .application_id("com.werryxgames.rem_admin.confirm".to_owned() + &random_int().to_string())
            .build();
            let win_title = argv[5].clone();
            let text = argv[6].clone();
            app.connect_activate(move |app: &gtk4::Application| {
                let win = gtk4::ApplicationWindow::new(app);
                win.set_title(Some(&win_title));
                win.set_resizable(false);
                let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
                vbox.set_margin_start(8);
                vbox.set_margin_end(8);
                vbox.set_margin_top(8);
                vbox.set_margin_bottom(8);
                vbox.set_spacing(8);
                win.set_child(Some(&vbox));
                let label = gtk4::Label::builder()
                .label(&text)
                .wrap(true)
                .wrap_mode(gtk4::pango::WrapMode::WordChar)
                .width_request(150)
                .max_width_chars(50)
                .build();
                vbox.append(&label);
                let btn1 = gtk4::Button::with_label("OK");
                btn1.connect_clicked(clone!(@weak win => move |_| {
                    *EXIT_CODE.lock().unwrap() = 1;
                    win.close();
                }));
                vbox.append(&btn1);
                let btn2 = gtk4::Button::with_label("Cancel");
                btn2.connect_clicked(clone!(@weak win => move |_| {
                    *EXIT_CODE.lock().unwrap() = 0;
                    win.close();
                }));
                vbox.append(&btn2);
                win.present();
            });
            let args: [&str; 0] = [];
            app.run_with_args(&args);
            let exit_code = *EXIT_CODE.lock().unwrap();
            exit(exit_code);
        } else if argv[3] == ARGV_DIALOG_INPUT {
            static EXIT_CODE: Mutex<i32> = Mutex::new(1);
            let app = gtk4::Application::builder()
            .application_id("com.werryxgames.rem_admin.prompt".to_owned() + &random_int().to_string())
            .build();
            let win_title = argv[5].clone();
            let text = argv[6].clone();
            app.connect_activate(move |app: &gtk4::Application| {
                let win = gtk4::ApplicationWindow::new(app);
                win.set_title(Some(&win_title));
                win.set_resizable(false);
                let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
                vbox.set_margin_start(8);
                vbox.set_margin_end(8);
                vbox.set_margin_top(8);
                vbox.set_margin_bottom(8);
                vbox.set_spacing(8);
                win.set_child(Some(&vbox));
                let label = gtk4::Label::builder()
                .label(&text)
                .wrap(true)
                .wrap_mode(gtk4::pango::WrapMode::WordChar)
                .width_request(150)
                .max_width_chars(50)
                .build();
                vbox.append(&label);
                let entry = gtk4::Entry::new();
                vbox.append(&entry);
                let btn = gtk4::Button::with_label("OK");
                btn.connect_clicked(clone!(@weak win, @weak entry => move |_| {
                    io::stdout().write_all("~".as_bytes()).unwrap();
                    io::stdout().write_all(entry.text().as_bytes()).unwrap();
                    io::stdout().flush().unwrap();
                    *EXIT_CODE.lock().unwrap() = 0;
                    win.close();
                }));
                entry.connect_activate(clone!(@weak btn => move |_| {
                    btn.activate();
                }));
                vbox.append(&btn);
                win.present();
            });
            let args: [&str; 0] = [];
            app.run_with_args(&args);
            let exit_code = *EXIT_CODE.lock().unwrap();
            exit(exit_code);
        }
    }

    let stream_m: Arc<Mutex<TcpStream>>;
    let mut started: bool = false;

    loop {
        if started {
            sleep(Duration::from_millis(CONNECT_INTERVAL));
        } else {
            started = true;
        }

        match TcpStream::connect(HOST) {
            Ok(server) => {
                server.set_nodelay(true).unwrap();
                server.set_nonblocking(true).unwrap();
                stream_m = Arc::new(Mutex::new(server));

                {
                    let mut stream = stream_m.lock().unwrap();
                    stream.set_nodelay(true).unwrap();
                    stream.set_nonblocking(true).unwrap();
                    println!("Connected to server '{}'", HOST);

                    let mut msg: Vec<u8> = Vec::new();
                    msg.push(ClientCodes::CAuth as u8);
                    msg.extend(VERSION.to_be_bytes());
                    msg.extend(AUTH_PARTS[0].to_be_bytes());
                    stream.write_all(&msg).unwrap();
                }

                let mut enigo = Enigo::new();

                let mut server_code = [0u8; 1];

                loop {
                    loop {
                        let mut stream = stream_m.lock().unwrap();

                        match stream.read_exact(&mut server_code) {
                            Ok(_) => {
                                break;
                            }
                            Err(err) => {
                                if err.kind() == ErrorKind::WouldBlock {
                                    continue;
                                }

                                panic!("{}", err);
                            }
                        };
                    }
    
                    let mut stream = stream_m.lock().unwrap();
                    let code: ServerCodes = server_code[0].try_into().unwrap();
                    let mut screen = scrap::Capturer::new(scrap::Display::primary().unwrap()).unwrap();

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
                                stream.write_all(&msg).unwrap();
                            } else {
                                let mut data2 = [0u8; 8];
                                stream.read_exact(&mut data2).unwrap();
                                let auth_part2 = u64::from_be_bytes(data2);

                                if auth_part2 != AUTH_PARTS[1] {
                                    let msg = [ClientCodes::CEAuthPart as u8; 1];
                                    stream.write_all(&msg).unwrap();
                                } else {
                                    let mut msg: Vec<u8> = Vec::new();
                                    msg.push(ClientCodes::CAuthOK as u8);
                                    let value: u128 = get_machine_id();
                                    msg.extend(value.to_be_bytes());
                                    stream.write_all(&msg).unwrap();
                                    println!("Authorized");
                                    continue;
                                }
                            }
                        }
                        ServerCodes::MTest => {
                            let mut buf = [0u8; 4];
                            stream.read_exact(&mut buf).unwrap();
                            let mut msg: Vec<u8> = Vec::new();
                            msg.push(ClientCodes::RTestEcho as u8);
                            msg.extend(buf);
                            stream.write_all(&msg).unwrap();
                        }
                        ServerCodes::MGui => {
                            let mut title_len = [0u8; 4];
                            stream.read_exact(&mut title_len).unwrap();
                            let mut title_bytes: Vec<u8> = vec![0u8; u32::from_be_bytes(title_len) as usize];
                            stream.read_exact(&mut title_bytes).unwrap();
                            let mut message_len = [0u8; 4];
                            stream.read_exact(&mut message_len).unwrap();
                            let mut message_bytes: Vec<u8> = vec![0u8; u32::from_be_bytes(message_len) as usize];
                            stream.read_exact(&mut message_bytes).unwrap();
                            let title = String::from_utf8(title_bytes).unwrap();
                            let message = String::from_utf8(message_bytes).unwrap();
                            show_dialog(stream_m.clone(), title, message);
                        }
                        ServerCodes::MGuiYesNo => {
                            let mut title_len = [0u8; 4];
                            stream.read_exact(&mut title_len).unwrap();
                            let mut title_bytes: Vec<u8> = vec![0u8; u32::from_be_bytes(title_len) as usize];
                            stream.read_exact(&mut title_bytes).unwrap();
                            let mut message_len = [0u8; 4];
                            stream.read_exact(&mut message_len).unwrap();
                            let mut message_bytes: Vec<u8> = vec![0u8; u32::from_be_bytes(message_len) as usize];
                            stream.read_exact(&mut message_bytes).unwrap();
                            let title = String::from_utf8(title_bytes).unwrap();
                            let message = String::from_utf8(message_bytes).unwrap();
                            show_dialog_yesno(stream_m.clone(), title, message);
                        }
                        ServerCodes::MAbort => {
                            let mut cmd_id_bytes = [0u8; 8];
                            stream.read_exact(&mut cmd_id_bytes).unwrap();
                            let cmd_id = u64::from_be_bytes(cmd_id_bytes);
                            println!("Request to abort {}", cmd_id);
                            let result = Request::abort(cmd_id);
                            println!("Done");
                            let mut msg: Vec<u8> = Vec::new();

                            if result.is_none() {
                                msg.push(ClientCodes::RNotAborted as u8);
                                msg.extend(cmd_id_bytes);
                                msg.push(false as u8);
                            } else if result.unwrap() {
                                    msg.push(ClientCodes::RAborted as u8);
                                    msg.extend(cmd_id_bytes);
                            } else {
                                msg.push(ClientCodes::RNotAborted as u8);
                                msg.extend(cmd_id_bytes);
                                msg.push(true as u8);
                            }

                            stream.write_all(&msg).unwrap();
                        }
                        ServerCodes::MMoveCursor => {
                            let mut data1 = [0u8; 4];
                            stream.read_exact(&mut data1).unwrap();
                            let mut data2 = [0u8; 4];
                            stream.read_exact(&mut data2).unwrap();
                            let x = i32::from_be_bytes(data1);
                            let y = i32::from_be_bytes(data2);
                            enigo.mouse_move_to(x, y);
                        }
                        ServerCodes::MMoveCursorRel => {
                            let mut data1 = [0u8; 4];
                            stream.read_exact(&mut data1).unwrap();
                            let mut data2 = [0u8; 4];
                            stream.read_exact(&mut data2).unwrap();
                            let x = i32::from_be_bytes(data1);
                            let y = i32::from_be_bytes(data2);
                            enigo.mouse_move_relative(x, y);
                        }
                        ServerCodes::MTypeKeyboard => {
                            let mut data1 = [0u8; 4];
                            stream.read_exact(&mut data1).unwrap();
                            let mut data2 = vec![0u8; u32::from_be_bytes(data1) as usize];
                            stream.read_exact(&mut data2).unwrap();
                            let sequence = String::from_utf8(data2).unwrap();
                            enigo.key_sequence_parse(&sequence);
                        }
                        ServerCodes::MClipboardGet => {
                            let code = Request::add(None);
                            let clipboard = terminal_clipboard::get_string().unwrap();
                            let mut msg: Vec<u8> = Vec::new();
                            msg.push(ClientCodes::ROkText as u8);
                            msg.extend((clipboard.len() as u32).to_be_bytes());
                            msg.extend(clipboard.as_bytes());
                            msg.extend(code.to_be_bytes());
                            stream.write_all(&msg).unwrap();
                        }
                        ServerCodes::MClipboardSet => {
                            let mut data1 = [0u8; 4];
                            stream.read_exact(&mut data1).unwrap();
                            let mut data2 = vec![0u8; u32::from_be_bytes(data1) as usize];
                            stream.read_exact(&mut data2).unwrap();
                            let clipboard = String::from_utf8(data2).unwrap();
                            println!("Content: {}", clipboard);
                            terminal_clipboard::set_string(clipboard).unwrap();
                        }
                        ServerCodes::MGuiInput => {
                            let mut title_len = [0u8; 4];
                            stream.read_exact(&mut title_len).unwrap();
                            let mut title_bytes: Vec<u8> = vec![0u8; u32::from_be_bytes(title_len) as usize];
                            stream.read_exact(&mut title_bytes).unwrap();
                            let mut message_len = [0u8; 4];
                            stream.read_exact(&mut message_len).unwrap();
                            let mut message_bytes: Vec<u8> = vec![0u8; u32::from_be_bytes(message_len) as usize];
                            stream.read_exact(&mut message_bytes).unwrap();
                            let title = String::from_utf8(title_bytes).unwrap();
                            let message = String::from_utf8(message_bytes).unwrap();
                            show_dialog_input(stream_m.clone(), title, message);
                        }
                        ServerCodes::MShellCommand => {
                            let mut cmd_len = [0u8; 4];
                            stream.read_exact(&mut cmd_len).unwrap();
                            let mut cmd_bytes: Vec<u8> = vec![0u8; u32::from_be_bytes(cmd_len) as usize];
                            stream.read_exact(&mut cmd_bytes).unwrap();
                            let cmd = String::from_utf8(cmd_bytes).unwrap();
                            execute_command(stream_m.clone(), cmd);
                        }
                        ServerCodes::MGetScreen => {
                            let width = screen.width() as u32;
                            let height = screen.height() as u32;
                            let mut frame: Vec<u8> = vec![];

                            for pixel in screen.frame().unwrap().chunks_exact(4) {
                                frame.push(pixel[2]);
                                frame.push(pixel[1]);
                                frame.push(pixel[0]);
                            }

                            let mut vec: Vec<u8> = Vec::new();
                            let encoder = Encoder::new(&mut vec, 100);
                            encoder.encode(&frame, width as u16, height as u16, ColorType::Rgb).unwrap();
                            let mut msg: Vec<u8> = vec![ClientCodes::RBytes as u8];
                            msg.extend(0u64.to_be_bytes());
                            let len = vec.len();
                            msg.extend((len as u32).to_be_bytes());
                            msg.extend(vec);
                            stream.write_all(&msg).unwrap();
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
    }
}
