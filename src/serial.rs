use csv::Writer;
use serialport;

use csv;

use serde;

use std::env::consts::OS;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::str;
use std::thread::sleep;
use std::time::Duration;

use crate::parser::parse;

pub fn open_port(
    env: &str,
    baud_rate: u32,
    timeout: u64,
) -> serialport::Result<Box<dyn serialport::SerialPort>> {
    let path = match env {
        "windows" => "COM3",
        "linux" => "/dev/ttyACM0",
        _ => {
            eprintln!("Failed to recognize OS");
            std::process::exit(1);
        }
    };

    serialport::new(path, baud_rate)
        .timeout(Duration::from_millis(timeout))
        .open()
}

pub fn split_byte_array(buf: &[u8], t: char) -> (&[u8], &[u8]) {
    let term: u8 = t as u8;

    // check for any markers
    for (i, b) in buf.iter().enumerate() {
        if *b == term {
            return (&buf[..i], &buf[i..]);
        }
    }

    return (buf, &[]); // return the same
}

#[derive(serde::Serialize)]
pub struct Reading {
    temperature: f32,
    pressure: f32,
    windspeed: f32,
    waterlevel: f32,
    humidity: f32,
}

fn create_csv_writer(p: &str) -> Result<Writer<File>, Box<dyn Error>> {
    match fs::metadata(p) {
        Ok(metadata) => {
            if metadata.is_file() {
                println!("found existing {p}");

                let f = fs::OpenOptions::new().append(true).open(p);
                match f {
                    Ok(file) => {
                        return Ok(csv::WriterBuilder::new()
                            .has_headers(false)
                            .from_writer(file))
                    }
                    Err(e) => return Err(Box::new(e)),
                }
            } else {
                println!("found {p} but not a valid comma separated file!");
                match csv::Writer::from_path(p) {
                    Ok(wtr) => return Ok(wtr),
                    Err(e) => return Err(Box::new(e)),
                };
            }
        }
        Err(_) => match csv::Writer::from_path(p) {
            Ok(wtr) => return Ok(wtr),
            Err(e) => return Err(Box::new(e)),
        },
    }
}

pub fn listen(port: &mut Box<dyn serialport::SerialPort>) -> Result<(), Box<dyn Error>> {
    let mut bytes_arr: Vec<u8> = Vec::new();

    let mut buf = [0; 32];

    let path = "./readings.csv";

    match create_csv_writer(path) {
        Ok(mut wtr) => {
            loop {
                match port.read(&mut buf) {
                    Ok(num_bytes) => {
                        if num_bytes > 0 {
                            let bytes = &buf[..num_bytes];

                            let (to_resolve, to_append) = split_byte_array(bytes, '\n');
                            bytes_arr.extend(to_resolve);

                            if to_append.len() > 0 {
                                match str::from_utf8(&bytes_arr) {
                                    Ok(string) => match parse(string) {
                                        Ok(parsed) => {
                                            // SAVE TO CSV HERE

                                            println!("{:?}", parsed.to_tuple());

                                            let record = parsed.to_hashmap();

                                            let res = wtr.serialize(Reading {
                                                temperature: record["temperature"],
                                                pressure: record["pressure"],
                                                windspeed: record["windspeed"],
                                                waterlevel: record["waterlevel"],
                                                humidity: record["humidity"],
                                            });

                                            match res {
                                                Ok(()) => println!("serialize success"),
                                                Err(e) => eprintln!("serialize failed: {e}"),
                                            };

                                            match wtr.flush() {
                                                Ok(()) => println!("write success"),
                                                Err(e) => eprintln!("write failed: {e}"),
                                            }
                                        }

                                        Err(e) => eprintln!("{e}"),
                                    },
                                    Err(e) => eprintln!("{e}"),
                                }

                                bytes_arr.clear();
                                bytes_arr.extend(to_append);
                            }
                        }
                    }
                    Err(e) => return Err(Box::new(e)),
                };
            }
        }
        Err(e) => return Err(e),
    }
}

pub fn init(baud_rate: u32, timeout: u64) {
    loop {
        match open_port(OS, baud_rate, timeout) {
            Ok(mut port) => {
                println!("connected!");
                listen(&mut port).unwrap();
            }
            Err(e) => {
                eprintln!("failed to open port: {:?}", e.description);
                eprintln!("retrying...");
            }
        }
        sleep(Duration::from_secs(1));
    }
}
