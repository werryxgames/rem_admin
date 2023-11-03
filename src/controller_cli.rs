use std::{io::{self, Write, Read, ErrorKind, Error}, sync::{Arc, Mutex}};
use rand::{thread_rng, Rng};

use crate::{server::Client, ServerCodes};

macro_rules! get_client {
    ($clients: expr, $id: expr, $if_b: expr, $else_b: expr) => {
        let clients = $clients.lock().unwrap();
        let mut found: bool = false;

        for client in clients.iter() {
            if client.lock().unwrap().id == $id {
                $if_b(&mut client.lock().unwrap());
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
        } else {
            if chr == '\\' {
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
        }
    };

    quote
}

pub fn controller_cli_start(clients: &Mutex<Vec<Arc<Mutex<Client>>>>) {
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    let mut selected_client: u128 = 0;

    loop {
        let mut buf: String = String::new();
        stdout.write(b"> ").unwrap();
        stdout.flush().unwrap();
        stdin.read_line(&mut buf).unwrap();
        let mut comargs: Vec<String> = Vec::new();
        
        if parse_quotes(&mut comargs, buf) {
            println!("Invalid syntax");
            continue;
        }

        if comargs.len() == 0 {
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
                let mut ids: Vec<u128> = Vec::new();

                for client in clients.lock().unwrap().iter() {
                    ids.push(client.lock().unwrap().id);
                }

                println!("Connected clients: {:?}", ids);
            }
            "select" => {
                if args.len() == 0 {
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

                get_client!(clients, selected_client, |client: &mut Client| {
                    let mut msg: Vec<u8> = Vec::new();
                    msg.push(ServerCodes::MTest as u8);
                    let num: u32 = thread_rng().gen();
                    msg.extend(num.to_be_bytes());
                    client.stream.write(&msg).unwrap();
                    let mut data = [0u8; 4];

                    loop {
                        match client.stream.read_exact(&mut data) {
                            Ok(_) => {
                                break;
                            }
                            Err(err) => {
                                if err.kind() == ErrorKind::WouldBlock {
                                    continue;
                                }

                                Err::<(), Error>(err).unwrap();
                            }
                        }
                    }

                    let recv_num = u32::from_be_bytes(data);

                    if recv_num == num {
                        println!("Test passed ({} == {})", recv_num, num);
                    } else {
                        println!("Test failed ({} != {})", recv_num, num);
                    }
                }, {
                    println!("Selected client not found")
                });
            }
            _ => {
                println!("Unknown command");
            }
        }
    }
}
