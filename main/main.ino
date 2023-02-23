
// lcd
#include <LiquidCrystal_I2C.h>

// i2c (BMP280)
#include "i2c_BMP280.h"

// Humidity Sensor
#include "DHT.h"

#define DHT_TYPE            DHT11
#define DHT11_PIN           4
#define IROA_PIN            5
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
LiquidCrystal_I2C lcd(0x27,16,2);
int position = 0;
int topPos = 0;
int botPos = 0;
String topStr;
String botStr;

// DHT11
DHT dht(DHT11_PIN, DHT_TYPE);
int humidity;

// BMP280
BMP280 bmp280;
float temperature, pascal;



// JSNSR04T
long duration;
float distance;

// IR Obstacle Avoidance
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

    // IR Obstacle Avoidance Sensor
    pinMode(INPUT, IROA_PIN);
}

void loop() {

    
    int currentMillis  = millis();

    // IR Obstacle Avoidance Sensor
    int status = digitalRead(IROA_PIN);


    if (status == 0) {

        if (!hovered) {
            
            signals++;
            hovered = true;
        }

    } else {
        hovered = false;
    }

    
    if (millis() - previousSerialMillis >= serialInterval) {


        // IROA
        float circumference = (2 * 3.1415926535 * 0.08)/3 ;
        float factor = 2;
        float signalsPerSecond = signals / serialMultiplier;
        float windspeed = signalsPerSecond * circumference * factor;

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

        // Output

        Serial.print("windspeed:");
        Serial.print(windspeed);
        Serial.print("pressure:");
        Serial.print(pascal);
        Serial.print("temperature:");
        Serial.print(temperature);
        Serial.print("humidity:");
        Serial.print(humidity);
        Serial.print("waterheight:");
        Serial.println(distance);


        // lcd update
        topStr = "                ";

        topStr += "temperature: ";
        topStr += String(temperature);
        topStr += " ";

        topStr += "pressure: ";
        topStr += String(pascal);
        topStr += " ";

        botStr = "                ";

        botStr += "wind: ";
        botStr += String(windspeed);
        botStr += " ";


        botStr += "water: ";
        botStr += String(distance);
        botStr += " ";


        botStr += "humidity: ";
        botStr += String(humidity);

        
    
        
        previousSerialMillis += serialInterval;
    }

    // LCD

    if (millis() - previousLCDMillis >= LCDInterval) {

    
        lcd.setCursor(0, 0);
        lcd.print(topStr);
        lcd.setCursor(0, 1);
        lcd.print(botStr);


        if (topPos < topStr.length()) {
            lcd.scrollDisplayLeft();
            topPos++;
        } else {
            topPos = 0;
        }
        
        if (botPos < botStr.length()) {
            lcd.scrollDisplayLeft();
            botPos++;
        } else {
            botPos = 0;
        }


        previousLCDMillis += LCDInterval;
    }
    
}