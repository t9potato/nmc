#[warn(clippy::pedantic)]

use std::net;
use std::{io::prelude::*, time::Duration, collections::HashMap, net::TcpStream};

/// Runs all given commands
/// 
/// Possible arguments:
///
/// TODO
/// help - Prints help screen
/// discard - removes curently playing song
/// status - Same as no arguments: Prints mpd status screen
/// toggle - Toggles mpd playback
/// volume - changes mpd volume
/// playlist - outputs mpd playlist with index numbers
/// repeat/random/single/consume - toggles mpd state
/// add - adds given files: seperated by comma
///
/// -p/--port - changes mpd port from default 6600
/// -h/--host - changes mpd host from default 127.0.0.1

fn main() -> Result<(), Box<dyn std::error::Error>>{
    // Default host and port
    let mut host = "127.0.0.1".to_string();
    let mut port = "6600".to_string();
    let mut args: Vec<String> = vec![];

    // TODO: Set host and port if ENV variables are set

    // Parse inputs
    std::env::args().skip(1).for_each(|arg| {
        if host == "" {
            host = arg;
            return;
        } else if port == "" {
            port = arg;
            return;
        } else if arg == "-p" || arg == "--port" {
            port = "".to_string();
            return;
        } else if arg == "-h" || arg == "--host" {
            host = "".to_string();
            return;
        }
        args.push(arg);
    });

    // Connect to mpd
    let mut connection = net::TcpStream::connect(
        format!("{}:{}", host, port)
    ).expect("Unable to connect to mpd server");
    // NOTE: Connection buffer reading times out but the contents are still
    //       read?
    connection.set_read_timeout(Some(Duration::from_millis(50)))?;

    type ArgAction = fn(String, &mut TcpStream) -> Result<String, Box<dyn std::error::Error>>;
    let mut arg_function: ArgAction = |_: String, _: &mut TcpStream| {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Option expected handler function none provided"
        )));
    };
    let mut arg_input = false;

    let mut str_buff = String::new();
    // Run user commands
    for arg in args {
        if arg_input {
            arg_function(arg, &mut connection)?;
            continue;
        }
        match arg.as_str() {
            "help" => {
                todo!();
            },
            "toggle" => {
                connection.write(b"pause\n")?;
                let _ = connection.read_to_string(&mut str_buff);
            },
            "discard" => {
                // Query consume state
                connection.write(b"status\n")?;
                let _ = connection.read_to_string(&mut str_buff);
                // Preform regex to get current consume state
                let mut discard_command: &[u8] = &[];
                for line in str_buff.lines() {
                    if !line.starts_with("consume: ") {
                        continue;
                    }
                    if line.contains("1") {
                        discard_command = b"next\n";
                    } else {
                        // If not consuming add toggle before and after next command
                        discard_command =
                            b"command_list_begin\n\
                            consume 1\n\
                            next\n\
                            consume 0\n\
                            command_list_end\n\
                            ";
                    }
                    break;
                }
                print!("{}", str_buff);
                connection.write(discard_command)?;

                let _ = connection.read_to_string(&mut str_buff);
            },
            "status" => {
                // Info about mpd status
                connection.write(b"status\n")?;
                let _ = connection.read_to_string(&mut str_buff);
                let mut items: HashMap<String, String> = HashMap::new();

                // Parse return into key value pairs
                str_buff.lines().for_each(|line| {
                    let (key, value) = line.split_once(':').unzip();
                    if let Some(key_value) = key {
                        items.insert(
                            key_value.to_string(),
                            if let Some(value_value) = value {
                                value_value.trim().to_string()
                            } else {
                                "".to_string()
                            }
                        );
                    }
                });
                // Get info about current song
                connection.write(
                    format!("playlistid {}\n", items["songid"]).as_bytes()
                )?;
                let _ = connection.read_to_string(&mut str_buff);
                
                // Parse return into key value pairs again
                str_buff.lines().for_each(|line| {
                    let (key, value) = line.split_once(':').unzip();
                    if let Some(key_value) = key {
                        items.insert(
                            key_value.to_string(),
                            if let Some(value_value) = value {
                                value_value.trim().to_string()
                            } else {
                                "".to_string()
                            }
                        );
                    }
                });

                println!("{:?}", items);
                // Output status
                // println!(
                //     "{}\n[{}] #{}/{}\n{}",
                //     items["file"],
                //     items["state"],
                // );
            },
            "playlist" => {
                connection.write(b"playlistinfo\n")?;
                let _ = connection.read_to_string(&mut str_buff);
                let mut index = 1;
                str_buff.lines().for_each(|line| {
                    if line.starts_with("file: ") {
                        if let Some (value) = line.split_once(": ") {
                            println!("{}: {}", index, value.1);
                            index += 1;
                        }
                    }
                });
            },
            "repeat" => {
                todo!();
            },
            "random" => {
                todo!();
            },
            "single" => {
                todo!();
            },
            "consume" => {
                todo!();
            },
            "volume" => {
                arg_input = true;
                arg_function = volume;
            },
            "add" => {
                todo!();
            },
            _ => {
                println!("Invalid argument: {}", arg);
            }
        }
        str_buff = "".to_string();
    }
    return Ok(());
}

fn volume(ammount: String, stream: &mut TcpStream) -> Result<String, Box<dyn std::error::Error>> {
    if !(ammount.starts_with("+") || ammount.starts_with("-")) {
        stream.write(format!("setvol {}\n", ammount).as_bytes())?;
        let buff: &mut [u8] = &mut [];
        let _ = stream.read(buff);
    } else {
        let change = ammount.parse::<i32>()?;
        let mut current = "".to_string();
        stream.write(b"getvol\n")?;
        let _ = stream.read_to_string(&mut current);
        let val = current.lines().skip(1).next()
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unexpected data from mpd",
            ))?
            .split_once(": ").ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unexpected data from mpd"
            ))?
            .1.parse::<i32>()?;
        stream.write(format!("setvol {}\n", val + change).as_bytes())?;
        let _ = stream.read_to_string(&mut current);
    }
    Ok("".to_string())
}
