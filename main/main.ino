
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
float timeMultiplier = 1000;

float serialMultiplier = 5;
float serialInterval = serialMultiplier * timeMultiplier;

float LCDMultiplier = 0.3;
float LCDInterval = LCDMultiplier * timeMultiplier;

unsigned long previousSerialMillis = 0;
unsigned long previousLCDMillis = 0;

// lcd
LiquidCrystal_I2C lcd(0x27,16, 1);
int position = 0;
int topPos = 0;
int botPos = 0;
String topStr;
String botStr;

// lcd new
String outputMessage = "                ";
int pos = 0;


// DHT11
DHT dht(DHT11_PIN, DHT_TYPE);
int humidity;

// BMP280
BMP280 bmp280;
float temperature, pascal;



// JSNSR04T
long duration;
float distance;

// Anemometer
float signals = 0;
bool hovered = false;



void setup()
{
    Serial.begin(115200);

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


    // init
    updateValues();

}

void loop() {

    
    int currentMillis  = millis();

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

    //

    
    if (millis() - previousSerialMillis >= serialInterval) {

        updateValues();

        previousSerialMillis += serialInterval;
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




void updateValues() {


    // Anemometer
    float circumference = (2 * 3.1415926535 * 0.08);
    float arc = (circumference * 120) / 360;
    // float factor = 2;
    float signalsPerInterval = signals / serialMultiplier;
    float windspeed = signalsPerInterval * arc;
    signals = 0; //reset


    // BMP280
    bmp280.awaitMeasurement();
    bmp280.getTemperature(temperature);
    bmp280.getPressure(pascal);
    bmp280.triggerMeasurement();


    // DHT11
    humidity = dht.readHumidity();


    // JSNSR04T
    digitalWrite(JSNSR04T_TRIG_PIN, LOW);
    delayMicroseconds(3);
    digitalWrite(JSNSR04T_TRIG_PIN, HIGH);
    delayMicroseconds(10);
    digitalWrite(JSNSR04T_TRIG_PIN, LOW);
    duration = pulseIn(JSNSR04T_ECHO_PIN, HIGH);
    distance = duration*0.034/2;

    // // Output
    Serial.print("windspeed:");
    Serial.print(windspeed);
    Serial.print("pressure:");
    Serial.print(pascal);
    Serial.print("temperature:");
    Serial.print(temperature);
    Serial.print("humidity:");
    Serial.print(humidity);
    Serial.print("waterlevel:");
    Serial.print(distance);
    Serial.print("\n"); // marker


    // lcd update
    outputMessage = "                ";
    outputMessage += "temperature: ";
    outputMessage += String(temperature);
    outputMessage += " ";
    outputMessage += "pressure: ";
    outputMessage += String(pascal);
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