use rp1_pinmux::*;

#[test]
fn every_assigned_cell_round_trips_and_blanks_do_not() {
    for (pin, functions) in BANK0.iter().enumerate() {
        for (func, &signal) in functions.iter().enumerate() {
            if signal != Signal::None {
                assert_eq!(funcsel(pin as u8, signal), Some(func as u8));
                assert!(candidate_pins(signal).any(|p| p == pin as u8));
            }
        }
        assert_eq!(funcsel(pin as u8, Signal::None), None);
    }
    assert!(candidate_pins(Signal::None).next().is_none());
}

#[test]
fn invalid_pins_instances_and_lanes_have_no_route() {
    for pin in [28, 53, 255] {
        assert_eq!(funcsel(pin, Signal::Uart(0, UartSignal::Tx)), None);
    }
    for signal in [
        Signal::Uart(5, UartSignal::Tx),
        Signal::I2c(4, I2cSignal::Sda),
        Signal::Spi(8, SpiSignal::Sclk),
        Signal::Pwm(1, PwmSignal::Channel0),
        Signal::I2s(0, I2sSignal::Sdi(4)),
        Signal::Dpi(DpiSignal::D(24)),
        Signal::Pio(28),
    ] {
        assert!(candidate_pins(signal).next().is_none(), "{signal:?}");
    }
}

#[test]
fn alternate_routes_are_complete_and_sorted() {
    assert_eq!(
        candidate_pins(Signal::I2c(3, I2cSignal::Sda)).collect::<Vec<_>>(),
        [6, 14, 22]
    );
    assert_eq!(
        candidate_pins(Signal::Gpclk(1)).collect::<Vec<_>>(),
        [5, 18, 21]
    );
    assert_eq!(
        candidate_pins(Signal::Pwm(0, PwmSignal::Channel2)).collect::<Vec<_>>(),
        [14, 18]
    );
    assert_eq!(funcsel(14, Signal::Pwm(0, PwmSignal::Channel2)), Some(0));
    assert_eq!(funcsel(18, Signal::Pwm(0, PwmSignal::Channel2)), Some(3));
    assert_eq!(
        candidate_pins(Signal::Spi(1, SpiSignal::Cs(1))).collect::<Vec<_>>(),
        [17, 27]
    );
}

#[test]
fn spi_slave_bus_directions_and_quad_lanes_are_preserved() {
    assert_eq!(funcsel(9, Signal::Spi(4, SpiSignal::Miso)), Some(8));
    assert_eq!(funcsel(10, Signal::Spi(4, SpiSignal::Mosi)), Some(8));
    assert_eq!(funcsel(0, Signal::Spi(0, SpiSignal::Sio3)), Some(0));
    assert_eq!(funcsel(1, Signal::Spi(0, SpiSignal::Sio2)), Some(0));
}

#[test]
fn uart_data_and_flow_control_routes() {
    for (n, start, func) in [(0, 14, 4), (1, 0, 2), (2, 4, 2), (3, 8, 2), (4, 12, 2)] {
        for (offset, role) in [
            UartSignal::Tx,
            UartSignal::Rx,
            UartSignal::Cts,
            UartSignal::Rts,
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(
                funcsel(start + offset as u8, Signal::Uart(n, role)),
                Some(func)
            );
        }
    }
}
