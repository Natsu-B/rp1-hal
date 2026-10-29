//! Lab-only ADC proof, run by proc1 (which owns the ADC and gets its Clocks from proc0):
//! a burst of conversions logged on UART0, at boot and on request
//! (link+0x28 = [`REQUEST`], reply at link+0x2c = the request on success, all-ones on failure).
use crate::rp1;
use embedded_hal_0_2::adc::OneShot;
use rp1_hal::{
    adc::{Adc, Error, TEMPERATURE, TemperatureSensor},
    log,
};
pub const REQUEST: u32 = 0xadc;
const LINK_REQUEST: usize = rp1_proto::LINK_START as usize + 0x28;
const SAMPLES: u32 = 1000;

fn now() -> u32 {
    unsafe { (0x400a_c028 as *const u32).read_volatile() }
}

/// min, max, sum of `SAMPLES` conversions of `ain` and the elapsed µs.
fn burst(adc: &mut Adc, ain: u8) -> Result<(u16, u16, u32, u32), Error> {
    let (mut min, mut max, mut sum) = (u16::MAX, 0, 0u32);
    let start = now();
    for _ in 0..SAMPLES {
        let v = adc.read(ain)?;
        min = min.min(v);
        max = max.max(v);
        sum += u32::from(v);
    }
    Ok((min, max, sum, now().wrapping_sub(start)))
}

pub fn prove(adc: &mut Adc, tag: &str) -> Result<(), Error> {
    // embedded-hal 0.2 path; the first conversion after the sensor is powered.
    let first = nb_block(OneShot::read(adc, &mut TemperatureSensor {}))?;
    for ain in 0..=TEMPERATURE {
        let (min, max, sum, us) = burst(adc, ain)?;
        log!(
            "ADC_{} ain={} min={} max={} sum={} us={}\n",
            tag,
            ain,
            min,
            max,
            sum,
            us
        )
        .unwrap();
    }
    let t = adc.read_temperature()?;
    log!("ADC_{} first={} temp_mc={} PASS\n", tag, first, t).unwrap();
    Ok(())
}

fn nb_block(r: nb::Result<u16, Error>) -> Result<u16, Error> {
    match r {
        Ok(v) => Ok(v),
        Err(nb::Error::Other(e)) => Err(e),
        Err(nb::Error::WouldBlock) => unreachable!("Adc::read blocks"),
    }
}

pub fn run(adc: Option<&mut Adc>, request: u32) {
    let result = match adc {
        Some(adc) => prove(adc, "P1REQ"),
        None => Err(Error::Precondition),
    };
    log!("ADC_RESULT request=0x{:x} {:?}\n", request, result).unwrap();
    unsafe {
        (LINK_REQUEST as *mut u32).write_volatile(0);
        ((LINK_REQUEST + 4) as *mut u32).write_volatile(if result.is_ok() {
            request
        } else {
            0xffff_ffff
        });
    }
    rp1_proto::barrier();
}
