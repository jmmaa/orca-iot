use nom::InputIter;
use serialport;

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

fn open_port(b: u32, t: u64) -> serialport::Result<Box<dyn serialport::SerialPort>> {
    // let path = match e {
    //     "windows" => "COM3",
    //     "linux" => "/dev/ttyACM0",
    //     _ => {
    //         eprintln!("Failed to recognize OS");
    //         std::process::exit(1);
    //     }
    // };

    println!("{:?}", serialport::available_ports());

    let port_infos =
        serialport::available_ports().map_or_else(|err| panic!("{err}"), |port_infos| port_infos);
    let port_info = port_infos
        .first()
        .unwrap_or_else(|| panic!("no specified port name!"));

    serialport::new(&port_info.port_name, b)
        .timeout(Duration::from_millis(t))
        .flow_control(serialport::FlowControl::None)
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

fn process_data(port: &mut Box<dyn serialport::SerialPort>, wtr: &mut Writer<File>) {
    let mut to_resolve: Vec<u8> = Vec::new();
    let mut buf = [0; 16];

    loop {
        match port.read(&mut buf) {
            Ok(num) => {
                if num > 0 {
                    let slice = Slicer::new(&buf);

                    let marker_pos = slice.from(0).to(num).position(|b| b == b'$');

                    if let Some(marker_index) = marker_pos {
                        to_resolve.extend(slice.up_to(marker_index));

                        match parse_reading(&to_resolve) {
                            Ok(parsed) => match write_reading(wtr, &parsed) {
                                Ok(()) => println!("success: {:?}", parsed.to_tuple()),
                                Err(e) => eprintln!("{e}"),
                            },
                            Err(e) => eprintln!("{e}"),
                        }

                        to_resolve.clear();
                        to_resolve.extend(slice.after(marker_index).up_to_end());
                    } else {
                        to_resolve.extend(slice.from(0).to(num));
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

pub struct SerialConfig {
    baud_rate: u32,
    timeout: u64,
}

pub fn start(baud_rate: u32, timeout: u64) {
    let path = "./readings.csv";
    let mut wtr = create_csv_writer(path)
        .unwrap_or_else(|err| panic!("cannot read file {path} with error: {err}"));

    loop {
        match open_port(baud_rate, timeout) {
            Ok(mut port) => process_data(&mut port, &mut wtr),
            Err(e) => eprintln!("{e}"),
        }

        sleep(Duration::from_secs(1));
    }
}
