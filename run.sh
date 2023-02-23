arduino-cli compile main --profile orca -u -p /dev/ttyACM0
cargo build --release
"./target/release/orca-iot"