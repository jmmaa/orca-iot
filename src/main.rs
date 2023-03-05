use clap::Parser;
use orca_iot::serial;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 9600)]
    baudrate: u32,

    #[arg(short, long, default_value_t = 100000)]
    timeout: u64,

    #[arg(short, long)]
    path: String,
}

fn main() {
    let args = Args::parse();
    serial::start(args.baudrate, args.timeout, args.path);
}
