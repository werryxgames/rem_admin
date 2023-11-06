use std::{io::{self, Write, ErrorKind, Read}, sync::{Arc, Mutex}};
use rand::{thread_rng, Rng};

use crate::{server::Client, ServerCodes, ClientCodes};

macro_rules! get_client {
    ($clients: expr, $args: expr, $index: expr, $if_b: expr, $else_b: expr) => {
        let mut clients = $clients.lock().unwrap();
        let mut found: bool = false;

        for client in clients.iter_mut() {
            if client.index == $index {
                $if_b(client, $args.to_vec());
                found = true;
                break;
            }
        }

        if !found {
            $else_b();
        }
    };
}

fn parse_quotes(vec: &mut Vec<String>, args: String) -> bool {
    let mut esc: bool = false;
    let mut buf: String = String::new();
    let mut quote: bool = false;

    macro_rules! esc_char {
        ($cchr: expr, $pchr: expr, $chr: expr, $buf: expr, $esc: expr, $else: expr) => {
            if $chr == $cchr {
                $esc = false;
                $buf.push($pchr);
            } else {
                $else
            }
        };
    }

    for chr in args.chars() {
        if esc {
            esc_char!('\\', '\\', chr, buf, esc,
            esc_char!('n', '\n', chr, buf, esc,
            esc_char!('r', '\r', chr, buf, esc,
            esc_char!('t', '\t', chr, buf, esc,
            esc_char!('\"', '\"', chr, buf, esc,
            esc_char!(' ', ' ', chr, buf, esc,
            esc_char!('\t', '\t', chr, buf, esc,
            esc_char!('\n', '\n', chr, buf, esc,
            esc_char!('\r', '\r', chr, buf, esc,
            {
                esc = true;
                buf.push('\\');
                buf.push(chr);
            }
            )))))))));
        } else if chr == '\\' {
            esc = true;
        } else if chr == '\"' {
            quote = !quote;
        } else if [' ', '\t', '\n', '\r'].contains(&chr) {
            if quote {
                buf.push(chr);
            } else if !buf.is_empty() {
                vec.push(buf.clone());
                buf.clear();
            }
        } else {
            buf.push(chr);
        }
    };

    quote
}

pub fn command_test(client: &mut Client, _args: Vec<String>) {
    let mut msg: Vec<u8> = Vec::new();
    msg.push(ServerCodes::MTest as u8);
    let num: u32 = thread_rng().gen();
    msg.extend(num.to_be_bytes());
    client.stream.lock().unwrap().write_all(&msg).unwrap();
    let mut code = [0u8; 1];

    loop {
        match client.stream.lock().unwrap().read_exact(&mut code) {
            Ok(_) => {
                break;
            }
            Err(err) => {
                if err.kind() == ErrorKind::WouldBlock {
                    continue;
                }

                panic!("{}", err);
            }
        }
    }

    if u8::from_be_bytes(code) == ClientCodes::RTestEcho as u8 {
        let mut data = [0u8; 4];

        loop {
            match client.stream.lock().unwrap().read_exact(&mut data) {
                Ok(_) => {
                    break;
                }
                Err(err) => {
                    if err.kind() == ErrorKind::WouldBlock {
                        continue;
                    }

                    panic!("{}", err);
                }
            }
        }

        let recv_num = u32::from_be_bytes(data);

        if recv_num == num {
            println!("Test passed ({} == {})", recv_num, num);
        } else {
            println!("Test failed ({} != {})", recv_num, num);
        }
    } else {
        println!("Invalid response");
    }
}

pub fn command_text(client: &mut Client, args: Vec<String>) {
    let mut msg: Vec<u8> = Vec::new();
    msg.push(ServerCodes::MGui as u8);
    msg.extend((args[0].len() as u32).to_be_bytes());
    msg.extend(args[0].as_bytes());
    msg.extend((args[1].len() as u32).to_be_bytes());
    msg.extend(args[1].as_bytes());
    client.request();
    client.stream.lock().unwrap().write_all(&msg).unwrap();
}

pub fn command_confirm(client: &mut Client, args: Vec<String>) {
    let mut msg: Vec<u8> = Vec::new();
    msg.push(ServerCodes::MGuiYesNo as u8);
    msg.extend((args[0].len() as u32).to_be_bytes());
    msg.extend(args[0].as_bytes());
    msg.extend((args[1].len() as u32).to_be_bytes());
    msg.extend(args[1].as_bytes());
    client.request();
    client.stream.lock().unwrap().write_all(&msg).unwrap();
}

pub fn command_abort(client: &mut Client, args: Vec<String>) {
    let mut msg: Vec<u8> = Vec::new();
    msg.push(ServerCodes::MAbort as u8);
    let arg_result: Result<u64, _> = args[0].parse();
    let mut arg_error = false;

    if arg_result.is_err() {
        arg_error = true;
    } else {
        let arg = arg_result.unwrap();
        msg.extend(arg.to_be_bytes());
        client.stream.lock().unwrap().write_all(&msg).unwrap();

        let mut code = [0u8; 1];

        loop {
            match client.stream.lock().unwrap().read_exact(&mut code) {
                Ok(_) => {
                    break;
                }
                Err(err) => {
                    if err.kind() == ErrorKind::WouldBlock {
                        continue;
                    }

                    panic!("{}", err);
                }
            }
        }

        let num_code = u8::from_be_bytes(code);

        if num_code == ClientCodes::RAborted as u8 {
            let mut data = [0u8; 8];

            loop {
                match client.stream.lock().unwrap().read_exact(&mut data) {
                    Ok(_) => {
                        break;
                    }
                    Err(err) => {
                        if err.kind() == ErrorKind::WouldBlock {
                            continue;
                        }

                        panic!("{}", err);
                    }
                }
            }

            let code = u64::from_be_bytes(data);

            if code == arg {
                println!("Aborted");
            } else {
                println!("Wrong process aborted");
            }
        } else if num_code == ClientCodes::RNotAborted as u8 {
            let mut data = [0u8; 8];

            loop {
                match client.stream.lock().unwrap().read_exact(&mut data) {
                    Ok(_) => {
                        break;
                    }
                    Err(err) => {
                        if err.kind() == ErrorKind::WouldBlock {
                            continue;
                        }

                        panic!("{}", err);
                    }
                }
            }

            let code = u64::from_be_bytes(data);
            let mut data2 = [0u8; 1];

            loop {
                match client.stream.lock().unwrap().read_exact(&mut data2) {
                    Ok(_) => {
                        break;
                    }
                    Err(err) => {
                        if err.kind() == ErrorKind::WouldBlock {
                            continue;
                        }

                        panic!("{}", err);
                    }
                }
            }

            let reason: bool = u8::from_be_bytes(data2) != 0;

            if code == arg {
                if reason {
                    println!("Already executed");
                } else {
                    println!("Process never spawned");
                }
            } else {
                println!("Wrong process not aborted");
            }
        }
    }

    if arg_error {
        println!("First argument must be an unsigned 64-bit integer");
    }
}

pub fn command_moveto(client: &mut Client, args: Vec<String>) {
    let x_result: Result<i32, _> = args[0].parse();

    if x_result.is_ok() {
        let y_result: Result<i32, _> = args[1].parse();

        if y_result.is_ok() {
            let x = x_result.unwrap();
            let y = y_result.unwrap();
            let mut msg: Vec<u8> = Vec::new();
            msg.push(ServerCodes::MMoveCursor as u8);
            msg.extend(x.to_be_bytes());
            msg.extend(y.to_be_bytes());
            client.stream.lock().unwrap().write_all(&msg).unwrap();
        } else {
            println!("Second argument must be a signed 32-bit integer");
        }
    } else {
        println!("First argument must be a signed 32-bit integer");
    }
}

pub fn command_moveby(client: &mut Client, args: Vec<String>) {
    let x_result: Result<i32, _> = args[0].parse();

    if x_result.is_ok() {
        let y_result: Result<i32, _> = args[1].parse();

        if y_result.is_ok() {
            let x = x_result.unwrap();
            let y = y_result.unwrap();
            let mut msg: Vec<u8> = Vec::new();
            msg.push(ServerCodes::MMoveCursorRel as u8);
            msg.extend(x.to_be_bytes());
            msg.extend(y.to_be_bytes());
            client.stream.lock().unwrap().write_all(&msg).unwrap();
        } else {
            println!("Second argument must be a signed 32-bit integer");
        }
    } else {
        println!("First argument must be a signed 32-bit integer");
    }
}

pub fn command_type(client: &mut Client, args: Vec<String>) {
    let mut msg: Vec<u8> = Vec::new();
    msg.push(ServerCodes::MTypeKeyboard as u8);
    msg.extend((args[0].len() as u32).to_be_bytes());
    msg.extend(args[0].as_bytes());
    client.stream.lock().unwrap().write_all(&msg).unwrap();
}

pub fn command_clipget(client: &mut Client, _args: Vec<String>) {
    let msg: Vec<u8> = vec![ServerCodes::MClipboardGet as u8];
    client.request();
    client.stream.lock().unwrap().write_all(&msg).unwrap();
}

pub fn command_clipset(client: &mut Client, args: Vec<String>) {
    let mut msg: Vec<u8> = Vec::new();
    msg.push(ServerCodes::MClipboardSet as u8);
    msg.extend((args[0].len() as u32).to_be_bytes());
    msg.extend(args[0].as_bytes());
    client.stream.lock().unwrap().write_all(&msg).unwrap();
}

pub fn command_prompt(client: &mut Client, args: Vec<String>) {
    let mut msg: Vec<u8> = Vec::new();
    msg.push(ServerCodes::MGuiInput as u8);
    msg.extend((args[0].len() as u32).to_be_bytes());
    msg.extend(args[0].as_bytes());
    msg.extend((args[1].len() as u32).to_be_bytes());
    msg.extend(args[1].as_bytes());
    client.request();
    client.stream.lock().unwrap().write_all(&msg).unwrap();
}

pub fn controller_cli_start(clients: Arc<Mutex<Vec<Client>>>) {
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    let mut selected_client: u64 = 0;

    loop {
        let mut buf: String = String::new();
        stdout.write_all(b"> ").unwrap();
        stdout.flush().unwrap();
        stdin.read_line(&mut buf).unwrap();
        let mut comargs: Vec<String> = Vec::new();
        
        if parse_quotes(&mut comargs, buf) {
            println!("Invalid syntax");
            continue;
        }

        if comargs.is_empty() {
            continue;
        }

        let com = comargs[0].as_str();
        let args = &comargs[1..];

        match com {
            "quit" | "exit" | "q" => {
                return;
            }
            "echo" => {
                println!("{}", args.join(" "));
            }
            "args" => {
                for arg in args.iter().enumerate() {
                    println!("Arg {}: '{}'", arg.0, arg.1);
                }
            }
            "list" => {
                let mut ids: Vec<String> = Vec::new();
                let clients2 = clients.clone();

                for client in clients2.lock().unwrap().iter() {
                    ids.push(format!("Client({}, {})", client.index, client.id));
                }

                println!("Connected clients: [{}]", ids.join(", "));
            }
            "select" => {
                if args.is_empty() {
                    println!("Selected client: {}", selected_client);
                } else {
                    match args[0].parse() {
                        Ok(value) => {
                            selected_client = value;
                        }
                        Err(err) => { 
                            println!("Error: {}", err);
                        }
                    };
                }
            }
            "test" => {
                if selected_client == 0 {
                    println!("Select client with command `select <id>`");
                    continue;
                }

                get_client!(clients, args, selected_client, command_test, {
                    println!("Selected client not found")
                });
            }
            "text" => {
                if args.len() != 2 {
                    println!("This command takes exactly 2 arguments, {} given", args.len());
                    continue;
                }

                if selected_client == 0 {
                    println!("Select client with command `select <id>`");
                    continue;
                }

                get_client!(clients, args, selected_client, command_text, {
                    println!("Selected client not found")
                });
            }
            "confirm" => {
                if args.len() != 2 {
                    println!("This command takes exactly 2 arguments, {} given", args.len());
                    continue;
                }

                if selected_client == 0 {
                    println!("Select client with command `select <id>`");
                    continue;
                }

                get_client!(clients, args, selected_client, command_confirm, {
                    println!("Selected client not found")
                });
            }
            "abort" => {
                if args.len() != 1 {
                    println!("This command takes exactly 1 argument, {} given", args.len());
                    continue;
                }

                if selected_client == 0 {
                    println!("Select client with command `select <id>`");
                    continue;
                }

                get_client!(clients, args, selected_client, command_abort, {
                    println!("Selected client not found")
                });
            }
            "moveto" => {
                if args.len() != 2 {
                    println!("This command takes exactly 2 arguments, {} given", args.len());
                    continue;
                }

                if selected_client == 0 {
                    println!("Select client with command `select <id>`");
                    continue;
                }

                get_client!(clients, args, selected_client, command_moveto, {
                    println!("Selected client not found")
                });
            }
            "moveby" => {
                if args.len() != 2 {
                    println!("This command takes exactly 2 arguments, {} given", args.len());
                    continue;
                }

                if selected_client == 0 {
                    println!("Select client with command `select <id>`");
                    continue;
                }

                get_client!(clients, args, selected_client, command_moveby, {
                    println!("Selected client not found")
                });
            }
            "type" => {
                if args.len() != 1 {
                    println!("This command takes exactly 1 argument, {} given", args.len());
                    continue;
                }

                if selected_client == 0 {
                    println!("Select client with command `select <id>`");
                    continue;
                }

                get_client!(clients, args, selected_client, command_type, {
                    println!("Selected client not found")
                });
            }
            "clipget" => {
                if !args.is_empty() {
                    println!("This command takes no arguments, {} given", args.len());
                    continue;
                }

                if selected_client == 0 {
                    println!("Select client with command `select <id>`");
                    continue;
                }

                get_client!(clients, args, selected_client, command_clipget, {
                    println!("Selected client not found")
                });
            }
            "clipset" => {
                if args.len() != 1 {
                    println!("This command takes exactly 1 argument, {} given", args.len());
                    continue;
                }

                if selected_client == 0 {
                    println!("Select client with command `select <id>`");
                    continue;
                }

                get_client!(clients, args, selected_client, command_clipset, {
                    println!("Selected client not found")
                });
            }
            "prompt" => {
                if args.len() != 2 {
                    println!("This command takes exactly 2 arguments, {} given", args.len());
                    continue;
                }

                if selected_client == 0 {
                    println!("Select client with command `select <id>`");
                    continue;
                }

                get_client!(clients, args, selected_client, command_prompt, {
                    println!("Selected client not found")
                });
            }
            _ => {
                println!("Unknown command");
            }
        }
    }
}
