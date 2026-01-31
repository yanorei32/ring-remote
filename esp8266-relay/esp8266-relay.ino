#include <Arduino.h>
#if defined(ESP8266)
#include <ESP8266WiFi.h>
#include <WiFiUdp.h>
#endif  // ESP8266
#if defined(ESP32)
#include <WiFi.h>
#include <WiFiUdp.h>
#endif  // ESP32
#include <WiFiClient.h>
#include <WiFiServer.h>

#include "wificred.h"

#define UDPBUF_SIZE 2048
char udpBuffer[UDPBUF_SIZE];

#define UDP_PORT 6464
WiFiUDP Server;

#define RELAY_PORT 0
#define LED_PORT 2

void setup() {
  pinMode(RELAY_PORT, OUTPUT);
  pinMode(LED_PORT, OUTPUT);
  digitalWrite(RELAY_PORT, HIGH);
  digitalWrite(LED_PORT, HIGH);

  Serial.begin(115200, SERIAL_8N1, SERIAL_TX_ONLY);
  delay(100);

  Serial.println();
  Serial.println("RELAY SERVER");

  WiFi.begin(SSID, PSK);

  while (WiFi.status() != WL_CONNECTED) {
    delay(250);
    Serial.print(".");
  }

  Serial.println(WiFi.localIP().toString());
  Server.begin(UDP_PORT);
}

void loop() {
  uint16_t packetLen = Server.parsePacket();
  if (!packetLen) return;

  Serial.print("Packet (len: ");
  Serial.print(packetLen);
  Serial.print(") from ");
  Serial.print(Server.remoteIP());
  Serial.print(":");
  Serial.print(Server.remotePort());
  Serial.println();

  if (packetLen != 1) {
    Serial.println("Ignore non just 1-byte packet");
    return;
  }

  packetLen = Server.read(udpBuffer, UDPBUF_SIZE);

  if (packetLen != 1) {
    Serial.println("Ignore non just 1-byte packet");
    return;
  }

  Serial.print("Data: ");
  Serial.println(udpBuffer[0], HEX);
  digitalWrite(RELAY_PORT, udpBuffer[0] ? LOW : HIGH);
  digitalWrite(LED_PORT, udpBuffer[0] ? LOW : HIGH);
}
