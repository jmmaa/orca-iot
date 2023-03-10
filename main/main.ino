
// lcd
#include <LiquidCrystal_I2C.h>

// i2c (BMP280)
#include "i2c_BMP280.h"

// Humidity Sensor
#include "DHT.h"

#define DHT_TYPE            DHT11
#define DHT11_PIN           4
#define ANEMOMETER_PIN      5
#define JSNSR04T_ECHO_PIN   6
#define JSNSR04T_TRIG_PIN   7


// timing
float serialInterval = 60; // s // change this  for timing serial prints
float updateInterval = 1; // s // change this for timing value updates
float LCDInterval = 300; // this is in millis // ms // change this for LCD interval
unsigned long previousSerialSeconds = 0;
unsigned long previousUpdateSeconds = 0;
unsigned long previousLCDMillis = 0;

// lcd
LiquidCrystal_I2C lcd(0x27,16, 1);
String outputMessage = "                ";
int pos = 0;


// DHT11
DHT dht(DHT11_PIN, DHT_TYPE);
float humidity;


float avgHumidity;

// BMP280
BMP280 bmp280;
float temperature;
float pascal;
float millibar;


float avgTemperature;
float avgPressure;

// JSNSR04T
long duration;
float distance;

float avgDistance;

// Anemometer
float windspeed;
float signals = 0;
bool hovered = false;

float avgWindspeed;



void setup()
{
    Serial.begin(9600);

    // lcd
    lcd.init();
    lcd.clear();
    lcd.backlight();
    lcd.leftToRight();


    // BMP280
    bmp280.initialize();
    bmp280.setEnabled(0);
    bmp280.triggerMeasurement();

    // DHT11
    dht.begin();
    

    // JSNSR04T
    pinMode(OUTPUT, JSNSR04T_TRIG_PIN);
    pinMode(INPUT, JSNSR04T_ECHO_PIN);

    // Anemometer
    pinMode(INPUT, ANEMOMETER_PIN);


    // // init
    // updateValues();

}

void loop() {


    // ANEMOMETER
    int status = digitalRead(ANEMOMETER_PIN);


    if (status == 0) {

        if (!hovered) {
            
            signals++;
            hovered = true;
        }

    } else {

        hovered = false;
    }

    // update values
    
    if ((millis()/1000) - previousUpdateSeconds >= updateInterval) {

        updateValues();

        previousUpdateSeconds += updateInterval;
    }

    // send serial values

    if ((millis()/1000) - previousSerialSeconds >= serialInterval) {

        printSerialValues();
        
        previousSerialSeconds += serialInterval;
    }


    // LCD
    if (millis() - previousLCDMillis >= LCDInterval) {
        
        
        lcd.clear();
        lcd.setCursor(0, 0);

        lcd.print(outputMessage.substring(pos, pos+16));

        if (pos < outputMessage.length()) {
            pos++;
        } else {
            pos = 0;
        }

        previousLCDMillis += LCDInterval;
    }
    
}


void printSerialValues() {

        // // Output
    Serial.print("temperature:");
    Serial.print(avgTemperature / serialInterval);

    Serial.print("pressure:");
    Serial.print(avgPressure / serialInterval);


    Serial.print("windspeed:");
    Serial.print(avgWindspeed / serialInterval);
    avgWindspeed = 0;


    Serial.print("waterlevel:");
    Serial.print(avgDistance / serialInterval);

    Serial.print("humidity:");
    Serial.print(avgHumidity / serialInterval);
    
    Serial.print("$"); // marker
}


void updateValues() {


    // Anemometer
    float circumference = (2 * 3.1415926535 * 0.08);
    float arc = (circumference) / 3;
    float factor = 3;

    windspeed = (signals / serialInterval) * arc * factor; //
    avgWindspeed += windspeed; 

    signals = 0; //reset


    // BMP280
    bmp280.awaitMeasurement();
    bmp280.getTemperature(temperature);
    bmp280.getPressure(pascal); // millibar

    avgTemperature += temperature;

    millibar = pascal / 100;
    avgPressure += millibar;

    bmp280.triggerMeasurement();


    // DHT11
    humidity = dht.readHumidity();

    avgHumidity += humidity;


    // JSNSR04T
    digitalWrite(JSNSR04T_TRIG_PIN, LOW);
    delayMicroseconds(3);
    digitalWrite(JSNSR04T_TRIG_PIN, HIGH);
    delayMicroseconds(10);
    digitalWrite(JSNSR04T_TRIG_PIN, LOW);
    duration = pulseIn(JSNSR04T_ECHO_PIN, HIGH);
    distance = duration*0.034/2; // Get the max range of the sensor


    avgDistance += distance;


    // lcd update
    outputMessage = "                ";
    outputMessage += "temperature: ";
    outputMessage += String(temperature);
    outputMessage += " ";
    outputMessage += "pressure: ";
    outputMessage += String(millibar);
    outputMessage += " ";
    outputMessage += "wind speed: ";
    outputMessage += String(windspeed);
    outputMessage += " ";
    outputMessage += "water level: ";
    outputMessage += String(distance);
    outputMessage += " ";
    outputMessage += "humidity: ";
    outputMessage += String(humidity);
    outputMessage += " "; 
}