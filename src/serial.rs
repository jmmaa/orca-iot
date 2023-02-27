use serialport;
use serialport::Result;

use reqwest;

use std::env::consts::OS;
use std::str;
use std::string::FromUtf8Error;
use std::thread::sleep;
use std::time::Duration;

use crate::parser::parse;

pub fn open_port(
    env: &str,
    baud_rate: u32,
    timeout: u64,
) -> Result<Box<dyn serialport::SerialPort>> {
    let path = match env {
        "windows" => "COM3",
        "linux" => "/dev/ttyACM0",
        _ => {
            eprintln!("Failed to recognize OS");
            std::process::exit(1);
        }
    };

    return serialport::new(path, baud_rate)
        .timeout(Duration::from_millis(timeout))
        .open();
}

pub fn read_bytes(
    port: &mut Box<dyn serialport::SerialPort>,
    buf: &mut Vec<u8>,
) -> Result<Vec<u8>> {
    return match port.read(buf) {
        Ok(_) => Ok(buf.to_owned()),
        Err(e) => Err(serialport::Error::from(e)),
    };
}

pub fn filter_null_bytes(byte_arr: Vec<u8>) -> Vec<u8> {
    return byte_arr
        .into_iter()
        .filter(|byte| byte.to_owned() != 0)
        .collect();
}

pub fn split_byte_array(buf: &[u8], t: char) -> (Vec<&u8>, Vec<&u8>) {
    let term: u8 = t as u8;

    let mut terminated = false;
    let mut resolving: Vec<&u8> = Vec::new();
    let mut remaining: Vec<&u8> = Vec::new();

    for b in buf.iter() {
        let _byte = b;

        if !terminated {
            if *_byte != term {
                resolving.push(_byte);
            } else {
                remaining.push(_byte);

                terminated = true;
            }
        } else {
            remaining.push(_byte);
        }
    }

    return (resolving, remaining);
}

pub fn resolve(bytes: Vec<u8>) -> std::result::Result<String, FromUtf8Error> {
    let filtered = filter_null_bytes(bytes.to_owned());
    return String::from_utf8(filtered);
}

pub fn listen(port: &mut Box<dyn serialport::SerialPort>) {
    let mut bytes_arr: Vec<u8> = Vec::new();

    let mut buf = [0; 32];

    let client = reqwest::blocking::Client::new();

    loop {
        match port.read(&mut buf) {
            Ok(num_bytes) => {
                if num_bytes > 0 {
                    let bytes = &buf[0..num_bytes];

                    let (resolving, remaining) = split_byte_array(bytes, '\n');

                    bytes_arr.extend(resolving);

                    if remaining.len() > 0 {
                        match resolve(bytes_arr) {
                            Ok(string) => match parse(&string) {
                                Ok(parsed) => {
                                    // SERVER COMM LOGIC HERE

                                    let data = &parsed.to_hashmap();
                                    let req = client
                                        .post("https://web-production-e3f6.up.railway.app/post")
                                        .json(data)
                                        .send();

                                    match req {
                                        Ok(res) => println!("success POST: {:?}", res),
                                        Err(e) => eprintln!("error POST: {e}"),
                                    }

                                    println!("{:?}", parsed.to_hashmap());
                                }
                                Err(e) => eprintln!("failed to parse data: {}", e),
                            },
                            Err(e) => {
                                eprintln!("failed to convert data to string: {}", e)
                            }
                        }

                        bytes_arr = Vec::new();
                        bytes_arr.extend(remaining)
                    }
                }
            }
            Err(e) => {
                eprintln!("failed to read: {}", e);
                break;
            }
        };
    }
}

pub fn init(baud_rate: u32, timeout: u64) {
    loop {
        match open_port(OS, baud_rate, timeout) {
            Ok(mut port) => {
                println!("connected!");
                listen(&mut port);
            }
            Err(e) => {
                eprintln!("failed to open port: {:?}", e.description);
                eprintln!("retrying...");
            }
        }
        sleep(Duration::from_secs(1));
    }
}
