use serialport;

use csv;
use csv::Writer;

use serde;

use std::env::consts::OS;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::str;
use std::thread::sleep;
use std::time::Duration;

use crate::parser::{parse, ParsedData};

#[derive(serde::Serialize)]
struct Reading {
    temperature: f32,
    pressure: f32,
    windspeed: f32,
    waterlevel: f32,
    humidity: f32,
}

fn open_port(e: &str, b: u32, t: u64) -> serialport::Result<Box<dyn serialport::SerialPort>> {
    let path = match e {
        "windows" => "COM3",
        "linux" => "/dev/ttyACM0",
        _ => {
            eprintln!("Failed to recognize OS");
            std::process::exit(1);
        }
    };

    serialport::new(path, b)
        .timeout(Duration::from_millis(t))
        .flow_control(serialport::FlowControl::None)
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .parity(serialport::Parity::None)
        .open()
}

fn create_csv_writer(p: &str) -> Result<Writer<File>, Box<dyn Error>> {
    match fs::metadata(p) {
        Ok(metadata) => {
            if metadata.is_file() {
                // if file, just append data

                let f = fs::OpenOptions::new().append(true).open(p);
                match f {
                    Ok(file) => {
                        return Ok(csv::WriterBuilder::new()
                            .has_headers(false)
                            .from_writer(file))
                    }
                    Err(e) => Err(Box::new(e)),
                }
            } else {
                // if not a file, create new writer

                match csv::Writer::from_path(p) {
                    Ok(wtr) => Ok(wtr),
                    Err(e) => Err(Box::new(e)),
                }
            }
        }
        Err(_) => {
            // if file not found, create new writer

            match csv::Writer::from_path(p) {
                Ok(wtr) => Ok(wtr),
                Err(e) => Err(Box::new(e)),
            }
        }
    }

    // REFACTOR: maybe make a custom error for cleaner return values
}

fn parse_reading<'a>(buf: &'a [u8]) -> Result<ParsedData, Box<dyn Error + 'a>> {
    match str::from_utf8(buf) {
        Ok(string) => match parse(string) {
            Ok(parsed) => Ok(parsed),

            Err(e) => Err(Box::new(e)),
        },
        Err(e) => Err(Box::new(e)),
    }
}

fn write_reading(wtr: &mut Writer<File>, parsed: &ParsedData) -> Result<(), Box<dyn Error>> {
    let record = parsed.to_hashmap();

    let serialize_res = wtr.serialize(Reading {
        temperature: record["temperature"],
        pressure: record["pressure"],
        windspeed: record["windspeed"],
        waterlevel: record["waterlevel"],
        humidity: record["humidity"],
    });

    let flush_res = wtr.flush();

    match serialize_res {
        Ok(()) => match flush_res {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(e)),
        },
        Err(e) => Err(Box::new(e)),
    }
}

type Buffer<'a> = (&'a [u8], &'a [u8]);

pub fn check_marker(bytes: &[u8], marker: u8) -> Option<usize> {
    let mut index = None;

    for (i, b) in bytes.iter().enumerate() {
        if *b == marker {
            index = Some(i);
            break;
        }
    }

    index
}

pub fn split_buffer(buf: &[u8], marker: u8) -> Buffer<'_> {
    if let Some(i) = check_marker(buf, marker) {
        (&buf[..i], &buf[i + 1..])
    } else {
        (buf, &[])
    }
}

/// starts a serial connection in a loop
///
/// # Arguments
///
/// * `baud_rate` = baud rate for serial connection
///
/// * `timeout` = amount of time to wait for receiving data before timing out
///
///
pub fn start(baud_rate: u32, timeout: u64) {
    let path = "./readings.csv";
    let mut wtr = create_csv_writer(path)
        .unwrap_or_else(|err| panic!("cannot read file {path} with error: {err}"));

    let mut to_resolve: Vec<u8> = Vec::new();
    let mut buf = [0; 32];

    loop {
        match open_port(OS, baud_rate, timeout) {
            Ok(mut port) => {
                loop {
                    match port.read(&mut buf) {
                        Ok(num) => {
                            if num > 0 {
                                let marker = b'$'; // splitting symbol

                                // filter null bytes (unix)
                                // let buffer = &buf
                                //     .iter()
                                //     .filter_map(|&b| if b != 0 { Some(b) } else { None })
                                //     .collect::<Vec<u8>>();

                                let (bytes, excess) = split_buffer(&buf, marker);

                                to_resolve.extend(bytes);

                                if !excess.is_empty() {
                                    match parse_reading(&to_resolve) {
                                        Ok(parsed) => match write_reading(&mut wtr, &parsed) {
                                            Ok(()) => println!("success: {:?}", parsed.to_tuple()),
                                            Err(e) => eprintln!("{e}"),
                                        },
                                        Err(e) => eprintln!("{e}"),
                                    }

                                    to_resolve.clear();
                                    to_resolve.extend(excess);
                                }
                            }
                        }
                        Err(e) => eprintln!("{e}"),
                    }
                }
            }
            Err(e) => eprintln!("{e}"),
        }

        sleep(Duration::from_secs(1));
    }

    // REFACTOR THIS LATER
}
