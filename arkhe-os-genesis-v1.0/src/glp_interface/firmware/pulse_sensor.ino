// pulse_sensor.ino — Biometric Entropy Sensor for MERKABAH-7
// Leitura simples de biometria para entropia SHM

void setup() {
  Serial.begin(9600);
}

void loop() {
  // Lê sensor analógico (A0) ou flutuação de voltagem (pino solto)
  // No MERKABAH-7, o ruído biométrico é usado como fonte de entropia [H]
  int sensorValue = analogRead(A0);

  // Envia dado bruto. O Python fará a normalização.
  Serial.println(sensorValue);

  // Taxa de amostragem: ~50Hz
  delay(20);
}
