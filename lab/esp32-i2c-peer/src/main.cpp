// Lab peer only. Boot-control NMOS sequences match cm5-control-arduino 0.4.0.
#include <Arduino.h>
#include <Wire.h>
#include <driver/rmt.h>
#include <assert.h>

static constexpr uint8_t ADDRESS = 0x2d;
static constexpr int SDA_PIN = 21, SCL_PIN = 22, BOOT_GATE = 18, POWER_GATE = 19;
static constexpr rmt_channel_t CHANNEL = RMT_CHANNEL_0;
static portMUX_TYPE mux = portMUX_INITIALIZER_UNLOCKED;
static bool boot_active, power_active, tracing, measuring;
static volatile uint32_t rx_count, rx_bytes, bad_frames, read_count, preload_errors;
static uint8_t reply[64];
static RingbufHandle_t ring;

static void gate(int pin, bool active) {
  // These drive NMOS gates, which only pull the CM5 signals LOW.
  digitalWrite(pin, active ? HIGH : LOW);
  pinMode(pin, OUTPUT);
  if (pin == BOOT_GATE) boot_active = active;
  else power_active = active;
}
static void pulse(uint32_t ms) { gate(POWER_GATE, true); delay(ms); gate(POWER_GATE, false); }
static void ok() { Serial.println("OK"); }
static void preload() {
  if (Wire.slaveWrite(reply, sizeof(reply)) != sizeof(reply)) ++preload_errors;
}
static void on_request() {
  portENTER_CRITICAL(&mux); ++read_count; portEXIT_CRITICAL(&mux);
  // Original ESP32 calls this after STOP. Tests wait for refill before another read.
  preload();
}
static void on_receive(int) {
  uint8_t frame[64]; size_t size = 0;
  while (Wire.available()) {
    const uint8_t value = Wire.read();
    if (size < sizeof(frame)) frame[size] = value;
    ++size;
  }
  // Every proof write is exactly the same 16-byte debug-mailbox frame.
  bool valid = size == 16;
  for (size_t i = 0; i < size && i < sizeof(frame); ++i) {
    const uint8_t expected = i < 4 ? uint8_t(0x52443144u >> (8 * i)) : uint8_t(i);
    valid &= frame[i] == expected;
  }
  portENTER_CRITICAL(&mux);
  ++rx_count; rx_bytes += size; if (!valid) ++bad_frames;
  portEXIT_CRITICAL(&mux);
}
static void slave_start() {
  assert(Wire.begin(ADDRESS, SDA_PIN, SCL_PIN, 100000));
  Wire.onReceive(on_receive); Wire.onRequest(on_request); preload();
}
static void status() {
  Serial.printf("nrpiboot=%s pwr_boot=%s trace=%s\n", boot_active ? "active" : "release",
    power_active ? "active" : "release", tracing ? "armed" : "disarmed");
}
static void stats() {
  portENTER_CRITICAL(&mux);
  const uint32_t count = rx_count, bytes = rx_bytes, bad = bad_frames, reads = read_count;
  portEXIT_CRITICAL(&mux);
  Serial.printf("i2c_addr=0x2d rx=%u bytes=%u bad_frames=%u reads=%u preload_errors=%u\n",
    count, bytes, bad, reads, preload_errors);
}
static void measure_stop() {
  if (!measuring) return;
  ESP_ERROR_CHECK(rmt_rx_stop(CHANNEL));
  ESP_ERROR_CHECK(rmt_driver_uninstall(CHANNEL));
  measuring = false; ring = nullptr; slave_start();
}
static void measure_start() {
  if (measuring) return;
  Wire.end(); pinMode(SDA_PIN, INPUT); pinMode(SCL_PIN, INPUT);
  rmt_config_t config = RMT_DEFAULT_CONFIG_RX(gpio_num_t(SCL_PIN), CHANNEL);
  config.clk_div = 8; // 80MHz APB / 8 = 100ns per tick.
  config.rx_config.filter_en = true;
  config.rx_config.filter_ticks_thresh = 1;
  config.rx_config.idle_threshold = 1000;
  ESP_ERROR_CHECK(rmt_config(&config));
  ESP_ERROR_CHECK(rmt_driver_install(CHANNEL, 2048, 0));
  ESP_ERROR_CHECK(rmt_get_ringbuf_handle(CHANNEL, &ring));
  ESP_ERROR_CHECK(rmt_rx_start(CHANNEL, true)); measuring = true;
}
static void measure_status() {
  Serial.printf("wiring=%s tick_ns=100\n", measuring ? "armed" : "disarmed");
  if (!measuring) return;
  size_t size;
  auto *items = static_cast<rmt_item32_t *>(xRingbufferReceive(ring, &size, 0));
  if (!items) { Serial.println("RMT empty"); return; }
  Serial.printf("RMT items=%u", unsigned(size / sizeof(*items)));
  for (size_t i = 0; i < size / sizeof(*items); ++i)
    Serial.printf(" %u:%u,%u:%u", items[i].level0, items[i].duration0,
      items[i].level1, items[i].duration1);
  Serial.println(); vRingbufferReturnItem(ring, items);
}
static void handle(String line) {
  line.trim();
  if (line == "version") Serial.println("firmware=cm5-control-trace version=0.7.0-rp1-hal-i2c-peer");
  else if (line == "status") status();
  else if (line == "pinmap") {
    Serial.println("GPIO18 nRPIBOOT NMOS gate; GPIO19 power NMOS gate");
    Serial.println("GPIO21 SDA / GPIO22 SCL input/open-drain, RP1 GPIO2/3 I2C1");
  }
  else if (line == "debug_status") { status(); stats(); }
  else if (line == "i2c_stats") stats();
  else if (line == "debug_arm") tracing = true;
  else if (line == "debug_disarm") tracing = false;
  else if (line == "trace_clear") {
    portENTER_CRITICAL(&mux); rx_count = rx_bytes = bad_frames = read_count = 0; portEXIT_CRITICAL(&mux);
  }
  else if (line == "wiring_arm") measure_start();
  else if (line == "wiring_disarm") measure_stop();
  else if (line == "wiring_status") measure_status();
  else if (line == "nrpiboot_on") gate(BOOT_GATE, true);
  else if (line == "nrpiboot_off") gate(BOOT_GATE, false);
  else if (line == "pwr_release") gate(POWER_GATE, false);
  else if (line.startsWith("pwr_press ")) {
    const long ms = line.substring(10).toInt();
    if (ms < 1 || ms > 10000) { Serial.println("ERR duration"); return; }
    pulse(ms);
  }
  else if (line == "force_boot") { gate(BOOT_GATE, false); pulse(10000); delay(1000); pulse(1000); }
  else if (line == "boot_usb") { gate(BOOT_GATE, true); delay(100); pulse(500); }
  else if (line == "boot_normal") { gate(BOOT_GATE, false); delay(100); pulse(500); }
  else if (line == "power_cycle") { pulse(1200); delay(1000); pulse(500); }
  else { Serial.println("ERR unknown_command"); return; }
  ok();
}
void setup() {
  gate(BOOT_GATE, false); gate(POWER_GATE, false);
  for (int pin : {4, 23, 25, 26, 27, 32, 33}) pinMode(pin, INPUT);
  for (size_t i = 0; i < sizeof(reply); ++i) reply[i] = uint8_t(0x31 + i * 29);
  Serial.begin(115200); Serial.setTimeout(100); slave_start();
}
void loop() { if (Serial.available()) handle(Serial.readStringUntil('\n')); }
