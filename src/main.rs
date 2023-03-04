use orca_iot::utils;

fn main() {
    const BAUD_RATE: u32 = 9600;
    const TIMEOUT: u64 = 100 * 1000;

    utils::start(BAUD_RATE, TIMEOUT);
}
