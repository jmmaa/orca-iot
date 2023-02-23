# ORCA

O.R.C.A. or Ocean Risk Intensity Classification and Analysis, is an IoT-based early warning system for coastal areas powered with machine learning. This repository covers the IoT layer of the project.

**Note: This project is still in development!**

### Peripherals

- Arduino Uno R3
- JSNSR04T Module
- DHT11 Module
- GY-BMP280 3.3v Module
- Hall Effect Sensor Module

## Download

TODO

## Building from source

The project uses [arduino-cli](https://arduino.github.io/arduino-cli/0.29/installation/) in compiling the arduino sketch and [rust](https://www.rust-lang.org/learn/get-started) in compiling the receiver for serial communication. It is recommended you install them along with the right setup in the arduino to fully run the project.

To run the project

```
./run
```

### Using only the sketch

uploading the sketch directly to your arduino

```
arduino-cli compile main --profile orca -u -p <port name>
```

If you only need the build files

```
arduino-cli compile main --profile orca --output-dir=./build
```
