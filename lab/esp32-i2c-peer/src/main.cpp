// Lab peer only. Boot-control NMOS sequences match cm5-control-arduino 0.4.0.
#include <Arduino.h>
#include <Wire.h>
#include <driver/rmt.h>
#include <esp_timer.h>
#include <assert.h>
#include "observer.h"

static constexpr uint8_t ADDRESS = 0x2d;
static constexpr int SDA_PIN = 21, SCL_PIN = 22, BOOT_GATE = 18, POWER_GATE = 19;
// The sample mask uses these pins in order, starting at bit 0.
static constexpr int GPIO_INPUTS[] = {4, 23, 25, 26, 27, 32, 33, 34, 35};
static constexpr size_t GPIO_SCAN_SAMPLES = 256;
static constexpr int64_t GPIO_SCAN_INTERVAL_US = 1000;
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
  size_t size = 0;
  bool valid = true;
  while (Wire.available()) {
    const uint8_t value = Wire.read();
    const uint8_t expected = size < 4 ? uint8_t(0x52443144u >> (8 * size)) : uint8_t(size);
    valid &= value == expected;
    ++size;
  }
  // The 96-byte variant forces the RP1 controller to refill its TX FIFO.
  valid &= size == 16 || size == 96;
  portENTER_CRITICAL(&mux);
  ++rx_count; rx_bytes += size; if (!valid) ++bad_frames;
  portEXIT_CRITICAL(&mux);
}
static void slave_start() {
  // The slave derives its SDA sample/hold timing from this rate; at 100000 it misread the
  // address at a real 400 kHz (HW). 400000 also serves 100 kHz masters.
  assert(Wire.begin(ADDRESS, SDA_PIN, SCL_PIN, 400000));
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
static void gpio_input_bias(bool pull_down) {
  for (int pin : GPIO_INPUTS) {
    // GPIO34/35 are input-only and have no internal pulls.
    pinMode(pin, pull_down && pin < 34 ? INPUT_PULLDOWN : INPUT);
    if (!pull_down) observer_float_pin(gpio_num_t(pin));
  }
}
static unsigned gpio_levels() {
  unsigned levels = 0;
  for (size_t i = 0; i < sizeof(GPIO_INPUTS) / sizeof(GPIO_INPUTS[0]); ++i) {
    levels |= unsigned(digitalRead(GPIO_INPUTS[i]) != LOW) << i;
  }
  return levels;
}
static void gpio_read(bool pull_down, bool scan) {
  if (observer_active()) {
    Serial.println("ERR observer_busy");
    return;
  }
  gpio_input_bias(pull_down);
  delayMicroseconds(1000);
  if (!scan) {
    const unsigned levels = gpio_levels();
    gpio_input_bias(false);
    Serial.print("gpio_read");
    for (size_t i = 0; i < sizeof(GPIO_INPUTS) / sizeof(GPIO_INPUTS[0]); ++i) {
      Serial.printf(" %d=%u", GPIO_INPUTS[i], (levels >> i) & 1u);
    }
    Serial.println();
    ok();
    return;
  }

  // Capture locally so USB/serial delays cannot miss a short pulse between host reads.
  uint16_t samples[GPIO_SCAN_SAMPLES];
  int64_t max_late = 0;
  Serial.println("GPIO_SCAN_READY");
  const int64_t start = esp_timer_get_time();
  for (size_t i = 0; i < GPIO_SCAN_SAMPLES; ++i) {
    const int64_t deadline = start + int64_t(i) * GPIO_SCAN_INTERVAL_US;
    int64_t now;
    do {
      now = esp_timer_get_time();
    } while (now < deadline);
    samples[i] = gpio_levels();
    const int64_t late = esp_timer_get_time() - deadline;
    if (late > max_late) max_late = late;
  }
  gpio_input_bias(false);
  Serial.printf("gpio_scan interval_us=%u max_late_us=%lld samples=",
                unsigned(GPIO_SCAN_INTERVAL_US), max_late);
  for (unsigned levels : samples) Serial.printf("%03x", levels);
  Serial.println();
  ok();
}
static void handle(String line) {
  line.trim();
  if (observer_command(line)) return;
  if (line == "version") Serial.println("firmware=cm5-control-trace version=0.8.1-rp1-hal-edge-observer");
  else if (line == "status") status();
  else if (line == "pinmap") {
    Serial.println("GPIO18 nRPIBOOT NMOS gate; GPIO19 power NMOS gate");
    Serial.println("GPIO21 SDA / GPIO22 SCL input/open-drain, RP1 GPIO2/3 I2C1");
  }
  else if (line == "gpio_read" || line == "gpio_read none") return gpio_read(false, false);
  else if (line == "gpio_read down") return gpio_read(true, false);
  else if (line == "gpio_read none scan") return gpio_read(false, true);
  else if (line == "gpio_read down scan") return gpio_read(true, true);
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
  gpio_input_bias(false);
  for (size_t i = 0; i < sizeof(reply); ++i) reply[i] = uint8_t(0x31 + i * 29);
  Serial.begin(115200); Serial.setTimeout(100); slave_start();
}
void loop() {
  observer_poll();
  if (Serial.available()) handle(Serial.readStringUntil('\n'));
}
