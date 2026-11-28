#include "observer.h"
#include <driver/gpio.h>
#include <driver/pcnt.h>
#include <driver/periph_ctrl.h>
#include <driver/rmt.h>
#include <driver/rtc_io.h>
#include <esp_intr_alloc.h>
#include <esp_timer.h>
#include <soc/gpio_reg.h>
#include <soc/gpio_sig_map.h>
#include <soc/io_mux_reg.h>
#include <soc/mcpwm_reg.h>
#include <soc/mcpwm_struct.h>
#include <soc/rtc_io_reg.h>
#include <soc/soc.h>

namespace {

constexpr uint32_t HZ = 80000000, CAPACITY = 2048, PAGE = 64;
constexpr int64_t WINDOW_US = 5000000;
constexpr unsigned IRQ_SHIFT = MCPWM_CAP0_INT_ST_S;
constexpr uint32_t IRQ_MASK = MCPWM_CAP0_INT_ST_M | MCPWM_CAP1_INT_ST_M | MCPWM_CAP2_INT_ST_M;
constexpr unsigned CAP_FALLING = 1, CAP_RISING = 2;
constexpr rmt_channel_t RMT_CH = RMT_CHANNEL_1;
constexpr gpio_num_t SPI_CLOCK = GPIO_NUM_33;
constexpr uint16_t SPI_IDLE_TICKS = 800; // 10 µs at the undivided APB clock.
constexpr uint8_t PINS[] = {34, 4, 27, 26, 32, 23, 33};

struct Edge {
  uint32_t tick;
  uint8_t pin, high;
};

Edge edges[CAPACITY];
uint8_t pins[3];
unsigned pin_count;
uint32_t nonce, limit;
volatile uint32_t seen;
bool active, spi, expired, installed, spi_latched;
uint32_t spi_last;
int64_t ends;
int16_t counts[3];
uint32_t saturated;
RingbufHandle_t spi_ring;
uint32_t rmt_items[64];
unsigned rmt_count, rmt_records;

void IRAM_ATTR collect_latches(uint32_t status) {
  uint32_t ticks[3];
  const uint32_t polarity = MCPWM0.cap_status.val;
  for (unsigned channel = 0; channel < 3; ++channel) {
    if (status & (1u << (IRQ_SHIFT + channel))) ticks[channel] = MCPWM0.cap_chn[channel].val;
  }
  MCPWM0.int_clr.val = status;
  for (unsigned channel = 0; channel < 3; ++channel) {
    if (!(status & (1u << (IRQ_SHIFT + channel)))) continue;
    const uint32_t total = seen;
    if (total > CAPACITY) return;
    Edge &edge = edges[total % CAPACITY];
    edge.tick = ticks[channel];
    edge.pin = pins[channel];
    edge.high = 1u ^ ((polarity >> channel) & 1u);
    seen = total + 1;
    if (seen > CAPACITY) {
      // Bound interrupt work even when the selected input is faster than the capture service.
      MCPWM0.int_ena.val = 0;
      return;
    }
  }
}

void IRAM_ATTR capture_isr(void *) {
  collect_latches(MCPWM0.int_st.val & IRQ_MASK);
}

void pin_status() {
  const uint64_t oe = REG_READ(GPIO_ENABLE_REG) | (uint64_t(REG_READ(GPIO_ENABLE1_REG)) << 32);
  const uint32_t rtc_oe = REG_READ(RTC_GPIO_ENABLE_REG) >> RTC_GPIO_ENABLE_S;
  for (int pin : PINS) {
    const uint32_t mux = REG_READ(GPIO_PIN_MUX_REG[pin]);
    bool up = mux & FUN_PU;
    bool down = mux & FUN_PD;
    bool output = oe & (1ULL << pin);
    bool rtc_selected = false;
    const int rtc = rtc_io_number_get(gpio_num_t(pin));
    if (rtc >= 0) {
      // IDF uses the RTC pad pull bits on RTC-capable pins, even in GPIO mode.
      const auto &pad = rtc_io_desc[rtc];
      const uint32_t rtc_mux = REG_READ(pad.reg);
      up |= (rtc_mux & pad.pullup) != 0;
      down |= (rtc_mux & pad.pulldown) != 0;
      rtc_selected = (rtc_mux & pad.mux) != 0;
      output |= rtc_selected && (rtc_oe & (1u << rtc));
    }
    Serial.printf("obs_pin pin=%d ie=%u up=%u down=%u oe=%u rtc=%u\n",
                  pin, unsigned((mux & FUN_IE) != 0), unsigned(up), unsigned(down),
                  unsigned(output), unsigned(rtc_selected));
  }
}

bool number(const char *&text, uint32_t &value) {
  if (*text < '0' || *text > '9') return false;
  value = 0;
  do {
    unsigned digit = *text++ - '0';
    if (value > (UINT32_MAX - digit) / 10) return false;
    value = value * 10 + digit;
  } while (*text >= '0' && *text <= '9');
  return true;
}

bool profile(const char *&text, uint8_t *selected, unsigned &size, bool &is_spi) {
  size = 0;
  is_spi = strncmp(text, "spi0 ", 5) == 0;
  if (is_spi) {
    selected[size++] = 26; // CS0
    selected[size++] = 33; // SCLK
    text += 5;
    return true;
  }
  do {
    if (size == 3) return false;
    uint32_t pin;
    if (!number(text, pin)) return false;
    bool allowed = false;
    for (unsigned candidate : PINS) allowed |= candidate == pin;
    for (unsigned i = 0; i < size; ++i) if (selected[i] == pin) return false;
    if (!allowed) return false;
    selected[size++] = pin;
    if (*text == ' ') { ++text; return true; }
  } while (*text++ == ',');
  return false;
}

bool start_rmt() {
  rmt_config_t config = RMT_DEFAULT_CONFIG_RX(SPI_CLOCK, RMT_CH);
  config.clk_div = 1;
  config.rx_config.filter_en = false;
  config.rx_config.idle_threshold = SPI_IDLE_TICKS;
  const esp_err_t configured = rmt_config(&config);
  const bool floating = observer_float_pin(SPI_CLOCK);
  if (configured != ESP_OK || !floating) {
    pinMode(SPI_CLOCK, INPUT);
    return false;
  }
  if (rmt_driver_install(RMT_CH, 512, ESP_INTR_FLAG_IRAM) != ESP_OK) return false;
  if (rmt_get_ringbuf_handle(RMT_CH, &spi_ring) == ESP_OK &&
      rmt_rx_start(RMT_CH, true) == ESP_OK) return true;
  rmt_driver_uninstall(RMT_CH);
  spi_ring = nullptr;
  return false;
}

void stop(bool timed_out) {
  if (!active) return;
  MCPWM0.int_ena.val = 0;
  // The ISR is allocated on this core. Mask it, freeze all latches and then
  // consume pending captures before clearing them. SPI SCLK uses its own anchor.
  for (unsigned channel = 0; channel < pin_count; ++channel) {
    MCPWM0.cap_chn_cfg[channel].val = 0;
    pcnt_counter_pause(pcnt_unit_t(channel));
  }
  const uint32_t pending = MCPWM0.int_raw.val & IRQ_MASK;
  spi_latched = spi && (pending & (1u << (IRQ_SHIFT + 1)));
  if (spi_latched) spi_last = MCPWM0.cap_chn[1].val;
  collect_latches(pending & (spi ? MCPWM_CAP0_INT_ST_M : IRQ_MASK));
  MCPWM0.cap_timer_cfg.val = 0;
  for (unsigned channel = 0; channel < pin_count; ++channel) {
    auto unit = pcnt_unit_t(channel);
    pcnt_get_counter_value(unit, &counts[channel]);
    // Counts also have to agree with the independently expected stimulus count.
    uint32_t status = 0;
    pcnt_get_event_status(unit, &status);
    if (status & PCNT_EVT_H_LIM) saturated |= 1u << channel;
  }
  MCPWM0.int_clr.val = IRQ_MASK;
  if (spi_ring) {
    rmt_rx_stop(RMT_CH);
    size_t size;
    while (void *record = xRingbufferReceive(spi_ring, &size, 0)) {
      if (++rmt_records == 1 && size <= sizeof(rmt_items)) {
        rmt_count = size / sizeof(rmt_items[0]);
        memcpy(rmt_items, record, size);
      }
      vRingbufferReturnItem(spi_ring, record);
    }
    rmt_driver_uninstall(RMT_CH);
    spi_ring = nullptr;
  }
  for (unsigned channel = 0; channel < pin_count; ++channel) pinMode(pins[channel], INPUT);
  active = false;
  expired = timed_out;
}

bool arm(uint32_t id, const uint8_t *selected, unsigned size, uint32_t depth, bool is_spi) {
  if (getApbFrequency() != HZ) return false;
  if (!installed) {
    periph_module_enable(PERIPH_PWM0_MODULE);
    MCPWM0.int_ena.val = 0;
    MCPWM0.int_clr.val = IRQ_MASK;
    if (esp_intr_alloc(ETS_PWM0_INTR_SOURCE, ESP_INTR_FLAG_IRAM | ESP_INTR_FLAG_LEVEL1,
                       capture_isr, nullptr, nullptr) != ESP_OK) return false;
    installed = true;
  }
  // PCNT belongs to the observer. Reset the limit-event latches as well as the
  // counts, since pcnt_counter_clear only promises to clear the count.
  periph_module_reset(PERIPH_PCNT_MODULE);
  for (unsigned channel = 0; channel < size; ++channel) {
    const auto unit = pcnt_unit_t(channel);
    const auto pin = gpio_num_t(selected[channel]);
    pinMode(pin, INPUT);
    gpio_matrix_in(pin, PWM0_CAP0_IN_IDX + channel, false);
    pcnt_config_t config{};
    config.pulse_gpio_num = pin;
    config.ctrl_gpio_num = PCNT_PIN_NOT_USED;
    config.pos_mode = PCNT_COUNT_INC;
    config.neg_mode = PCNT_COUNT_INC;
    config.counter_h_lim = 32767;
    config.counter_l_lim = -1;
    config.unit = unit;
    config.channel = PCNT_CHANNEL_0;
    const esp_err_t configured = pcnt_unit_config(&config);
    // IDF's legacy PCNT driver enables a pull-up even on an input-only net.
    const bool floating = observer_float_pin(pin);
    if (configured != ESP_OK || !floating ||
        pcnt_counter_pause(unit) != ESP_OK || pcnt_filter_disable(unit) != ESP_OK ||
        pcnt_event_enable(unit, PCNT_EVT_H_LIM) != ESP_OK || pcnt_counter_clear(unit) != ESP_OK) {
      pinMode(pin, INPUT);
      return false;
    }
  }
  if (is_spi && !start_rmt()) return false;
  memcpy(pins, selected, size);
  memset(counts, 0, sizeof(counts));
  pin_count = size;
  nonce = id;
  limit = depth;
  seen = saturated = spi_last = rmt_records = rmt_count = 0;
  spi = is_spi;
  expired = spi_latched = false;
  MCPWM0.cap_timer_phase.val = 0;
  // Load phase zero by software sync, then count APB ticks without further syncs.
  mcpwm_cap_timer_cfg_reg_t timer{};
  timer.cap_timer_en = 1;
  timer.cap_synci_en = 1;
  MCPWM0.cap_timer_cfg.val = timer.val;
  timer.cap_sync_sw = 1;
  MCPWM0.cap_timer_cfg.val = timer.val;
  timer.cap_sync_sw = timer.cap_synci_en = 0;
  MCPWM0.cap_timer_cfg.val = timer.val;
  for (unsigned channel = 0; channel < size; ++channel) {
    mcpwm_cap_chn_cfg_reg_t cfg{};
    cfg.capn_en = 1;
    cfg.capn_mode = is_spi && channel == 1 ? CAP_FALLING : CAP_FALLING | CAP_RISING;
    MCPWM0.cap_chn_cfg[channel].val = cfg.val;
    pcnt_counter_resume(pcnt_unit_t(channel));
  }
  MCPWM0.int_clr.val = IRQ_MASK;
  // SPI clocks can overwrite their latch. Use its final negative edge and RMT widths.
  MCPWM0.int_ena.val = (is_spi ? 1u : (1u << size) - 1) << IRQ_SHIFT;
  ends = esp_timer_get_time() + WINDOW_US;
  active = true;
  return true;
}

uint32_t retained(uint32_t total) { return total < limit ? total : limit; }

void status() {
  const uint32_t total = seen;
  Serial.printf("obs nonce=%u active=%u expired=%u count=%u dropped=%u saturated=%u pcnt=%d,%d,%d\n",
                nonce, active, expired, retained(total), total - retained(total),
                saturated, counts[0], counts[1], counts[2]);
  if (spi && !active) {
    Serial.printf("obs_spi latched=%u last=%u records=%u items=%u",
                  spi_latched, spi_last, rmt_records, rmt_count);
    for (unsigned i = 0; i < rmt_count; ++i) Serial.printf(" %08x", rmt_items[i]);
    Serial.println();
  }
}

} // namespace

bool observer_float_pin(gpio_num_t pin) {
  // GPIO34/35 are input-only and have no programmable pull resistors.
  if (!GPIO_IS_VALID_OUTPUT_GPIO(pin)) return true;
  const esp_err_t result = gpio_set_pull_mode(pin, GPIO_FLOATING);
  // On RTC-capable pins the driver clears the RTC fields. Clear the digital
  // IO_MUX fields too, including GPIO4's reset-default digital pull-down.
  REG_CLR_BIT(GPIO_PIN_MUX_REG[pin], FUN_PU | FUN_PD);
  return result == ESP_OK;
}

void observer_poll() {
  if (active && esp_timer_get_time() >= ends) stop(true);
}

bool observer_active() { return active; }

bool observer_command(const String &line) {
  if (!line.startsWith("obs_")) return false;
  const char *error = "observer_command";
  if (line == "obs_caps") {
    Serial.printf("obs_caps version=1 hz=%u capacity=%u page=%u window_us=%lld pins=34,4,27,26,32,23,33 profile=spi0\n",
                  HZ, CAPACITY, PAGE, WINDOW_US);
    error = nullptr;
  } else if (line == "obs_status") {
    status();
    error = nullptr;
  } else if (line == "obs_pins") {
    pin_status();
    error = nullptr;
  } else if (line.startsWith("obs_arm ")) {
    const char *arg = line.c_str() + 8;
    uint32_t id, depth;
    uint8_t selected[3];
    unsigned size;
    bool is_spi;
    if (number(arg, id) && id && *arg++ == ' ' && profile(arg, selected, size, is_spi) &&
        number(arg, depth) && !*arg && depth && depth <= CAPACITY) {
      error = active ? "observer_busy" :
              arm(id, selected, size, depth, is_spi) ? nullptr : "observer_hardware";
    } else {
      error = "observer_args";
    }
  } else if (line.startsWith("obs_abort ")) {
    const char *arg = line.c_str() + 10;
    uint32_t id;
    if (!number(arg, id) || !id || id != nonce) error = "observer_nonce";
    else if (*arg) error = "observer_args";
    else {
      stop(false);
      nonce = seen = saturated = 0;
      memset(counts, 0, sizeof(counts));
      spi = expired = false;
      error = nullptr;
    }
  } else if (line.startsWith("obs_stop ") || line.startsWith("obs_read ")) {
    const bool read = line.startsWith("obs_read ");
    const char *arg = line.c_str() + 9;
    uint32_t id, offset, length;
    if (!number(arg, id) || !id || id != nonce) {
      error = "observer_nonce";
    } else if (!read && !*arg) {
      stop(false);
      status();
      error = nullptr;
    } else if (read && active) {
      error = "observer_busy";
    } else if (read && *arg++ == ' ' && number(arg, offset) && *arg++ == ' ' &&
               number(arg, length) && !*arg && length && length <= PAGE &&
               offset <= retained(seen) && length <= retained(seen) - offset) {
      Serial.printf("obs_data nonce=%u offset=%u count=%u\n", nonce, offset, length);
      const uint32_t first = seen - retained(seen) + offset;
      for (uint32_t i = 0; i < length; ++i) {
        const Edge &edge = edges[(first + i) % CAPACITY];
        Serial.printf("edge %u %u %u\n", edge.tick, edge.pin, edge.high);
      }
      error = nullptr;
    } else {
      error = "observer_args";
    }
  }
  if (error) Serial.printf("ERR %s\n", error);
  else Serial.println("OK");
  return true;
}
