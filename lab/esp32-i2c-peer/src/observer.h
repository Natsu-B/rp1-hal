#pragma once
#include <Arduino.h>
#include <driver/gpio.h>

bool observer_command(const String &line);
bool observer_active();
bool observer_float_pin(gpio_num_t pin);
void observer_poll();
