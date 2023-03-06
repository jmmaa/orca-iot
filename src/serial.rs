use nom::InputIter;
use serialport::{
    self, DataBits, FlowControl, Parity, SerialPortType::UsbPort, StopBits, UsbPortInfo,
};

use csv;
use csv::Writer;

use serde;

use std::error::Error;
use std::fs;
use std::fs::File;
use std::str;
use std::thread::sleep;
use std::time::Duration;

use crate::parser::{parse, ParsedData};
use crate::utils::Slicer;

#[derive(serde::Serialize)]
struct Reading {
    temperature: f32,
    pressure: f32,
    windspeed: f32,
    waterlevel: f32,
    humidity: f32,
}

#[derive(Debug)]
struct FindPortError<'a> {
    description: &'a str,
}

impl<'a> std::fmt::Display for FindPortError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl<'a> std::error::Error for FindPortError<'a> {
    fn description(&self) -> &str {
        self.description
    }
}

fn find_port() -> Result<String, Box<dyn Error>> {
    let ports = serialport::available_ports();

    match ports {
        Ok(_ports) => {
            for port in _ports {
                if let UsbPort(UsbPortInfo {
                    manufacturer: Some(m),
                    ..
                }) = port.port_type
                {
                    if m.as_str().contains("Arduino") {
                        println!("found Arduino on port {:?}", port.port_name);

                        return Ok(port.port_name);
                    }
                }
            }

            Err(Box::new(FindPortError {
                description: "cannot find a working port",
            }))
        }
        Err(e) => Err(Box::new(e)),
    }
}

fn open_port(b: u32, t: u64) -> Result<Box<dyn serialport::SerialPort>, Box<dyn Error>> {
    match find_port() {
        Ok(path) => {
            let port = serialport::new(path, b)
                .flow_control(FlowControl::None)
                .data_bits(DataBits::Eight)
                .stop_bits(StopBits::One)
                .timeout(Duration::from_millis(t))
                .parity(Parity::None)
                .open();

            match port {
                Ok(p) => Ok(p),
                Err(e) => Err(Box::new(e)),
            }
        }
        Err(e) => Err(e),
    }
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

fn process_data(port: &mut Box<dyn serialport::SerialPort>, wtr: &mut Writer<File>) {
    let mut to_resolve: Vec<u8> = Vec::new();
    let mut buf = [0; 4];

    loop {
        match port.read(&mut buf) {
            Ok(num) => {
                if num > 0 {
                    let bytes_read = Slicer::new(&buf).to_before(num);
                    let slice = Slicer::new(bytes_read);

                    let marker_pos = slice.to_end().position(|b| b == b'$');

                    if let Some(marker_index) = marker_pos {
                        to_resolve.extend(slice.to_before(marker_index));

                        match parse_reading(&to_resolve) {
                            Ok(parsed) => match write_reading(wtr, &parsed) {
                                Ok(()) => println!("success: {:?}", parsed.to_tuple()),
                                Err(e) => eprintln!("{e}"),
                            },
                            Err(e) => eprintln!("{e}"),
                        }

                        to_resolve.clear();
                        to_resolve.extend(slice.from_after(marker_index).to_end());
                    } else {
                        to_resolve.extend(slice.to_before(num));
                    }
                }
            }
            Err(e) => {
                eprintln!("{e}");
                break;
            }
        }
    }
}

pub fn start(baudrate: u32, timeout: u64, path: String) {
    let mut wtr = create_csv_writer(path.as_str())
        .unwrap_or_else(|err| panic!("cannot read file {path} with error: {err}"));

    loop {
        match open_port(baudrate, timeout) {
            Ok(mut port) => process_data(&mut port, &mut wtr),
            Err(e) => eprintln!("{e}"),
        }

        sleep(Duration::from_secs(1));
    }
}

// CREATE BINARY FOR THIS
// CLI
//
