# ORCA

O.R.C.A. or Ocean Risk Intensity Classification and Analysis, is an IoT-based early warning system for coastal areas powered with machine learning. This repository covers the IoT layer of the project. Check details below for additional information.

## Links
- [Paper](https://github.com/jmmaa/orca-iot/blob/main/thesis.pdf)

## Building from source

The project uses [arduino-cli](https://arduino.github.io/arduino-cli/0.29/installation/) in compiling the arduino sketch and [rust](https://www.rust-lang.org/learn/get-started) in compiling the receiver for serial communication. It is recommended you install them along with the right setup in the arduino to fully run the project.

To run the project

```cmd
./run
```

### Using only the sketch

uploading the sketch directly to your arduino

```cmd
arduino-cli compile main --profile orca -u -p <port name>
```

If you only need the build files

```cmd
arduino-cli compile main --profile orca --output-dir=./build
```
