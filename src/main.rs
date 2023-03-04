use orca_iot::serial;

fn main() {
    const BAUD_RATE: u32 = 115200;
    const TIMEOUT: u64 = 100 * 1000;

    serial::start(BAUD_RATE, TIMEOUT);
}
