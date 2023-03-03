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

fn open_port(
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

#[derive(serde::Serialize)]
struct Reading {
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

fn check_marker(bytes: &[u8], marker: u8) -> Option<usize> {
    let mut index = None;

    for (i, b) in bytes.iter().filter(|&&b| b != 0).enumerate() {
        if *b == marker {
            index = Some(i);
            break;
        }
    }

    index
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

fn read_reading<'a>(
    port: &mut Box<dyn serialport::SerialPort>,
    buf: &'a mut [u8],
) -> Result<Buffer<'a>, Box<dyn Error>> {
    match port.read(buf) {
        Ok(_) => {
            if let Some(index) = check_marker(buf, b'\n') {
                Ok((&buf[..index], &buf[index..]))
            } else {
                Ok((buf, &[]))
            }
        }
        Err(e) => Err(Box::new(e)),
    }
}

fn listen(port: &mut Box<dyn serialport::SerialPort>) -> Result<(), Box<dyn Error>> {
    let path = "./readings.csv";
    let mut wtr = create_csv_writer(path)
        .unwrap_or_else(|err| panic!("cannot read file {path} with error: {err}"));

    let mut buffer: Vec<u8> = Vec::new();

    loop {
        match read_reading(port, &mut [0; 32]) {
            Ok((bytes, excess)) => {
                buffer.extend(bytes);

                if !excess.is_empty() {
                    match parse_reading(&buffer) {
                        Ok(parsed) => {
                            let res = write_reading(&mut wtr, &parsed);
                            match res {
                                Ok(()) => println!("success: {:?}", parsed.to_tuple()),
                                Err(e) => eprintln!("{e}"),
                            }
                        }
                        Err(e) => eprintln!("{e}"),
                    }

                    buffer.clear();
                    buffer.extend(excess);
                }
            }
            Err(e) => eprintln!("{e}"),
        }
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
