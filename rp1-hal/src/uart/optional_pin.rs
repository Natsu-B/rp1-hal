//! Host fixture for adding UART2 CTS without changing the required pin tuple or callers.
use super::*;
use crate::{fugit::RateExtU32, gpio::ErasedPin};

type Required = (Pin<4>, Pin<5>);
type Uart2Parts = Parts<2, Required>;
type Uart2 = Uart<2, Required>;

impl<P> Pins<2, P> {
    fn cts(self, pin: Pin<6>) -> Self {
        Pins(self.0, Some(pin.erase()))
    }

    fn take_cts(&mut self) -> Option<ErasedPin> {
        self.1.take()
    }
}

fn assemble(token: Token<2>, tx: Pin<4>, rx: Pin<5>) -> Uart2Parts {
    let pins: Pins<2, Required> = Pins::<2>::builder().tx(tx).rx(rx);
    Parts::new(token, pins)
}

fn add_cts(token: Token<2>, tx: Pin<4>, rx: Pin<5>, cts: Pin<6>) -> Uart2Parts {
    let pins: Pins<2, Required> = Pins::<2>::builder().tx(tx).cts(cts).rx(rx);
    Parts::new(token, pins)
}

#[allow(dead_code)]
fn unchanged_driver_calls(
    parts: Uart2Parts,
    clocks: &Clocks,
) -> Result<Uart2, InitError<2, Required>> {
    let uart = parts.init(clocks, 115_200.Hz())?;
    let (tx, rx): (Tx<2, 4>, Rx<2, 5>) = uart.split();
    Ok(Uart::join(tx, rx))
}

#[test]
fn optional_cts_keeps_required_aliases_and_call_shape() {
    // Own the pins without touching MMIO.
    let token = unsafe { Token::<2>::new() };
    let (tx, rx, cts) = unsafe { (Pin::new(), Pin::new(), Pin::new()) };
    let start = |parts: Uart2Parts| -> Uart2 {
        parts
            .init_on(
                &mut tests::Fake::new(),
                50.MHz(),
                115_200.Hz().into(),
                false,
                |tx, rx| (tx.assume(), rx.assume()),
            )
            .unwrap()
    };
    let uart = start(assemble(token, tx, rx));
    let (token, mut pins) = uart.into_parts().into_parts();
    assert!(pins.take_cts().is_none());
    let (tx, rx) = pins.into_pins();

    let (tx, rx): (Tx<2, 4>, Rx<2, 5>) = start(add_cts(token, tx, rx, cts)).split();
    let (_, mut pins) = Uart::join(tx, rx).into_parts().into_parts();
    let cts = pins.take_cts().unwrap().try_into_pin::<7>().err().unwrap();
    let _: Pin<6> = cts.try_into_pin().ok().unwrap();
}
