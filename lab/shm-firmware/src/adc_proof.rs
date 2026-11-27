//! Lab-only ADC proof, run by proc1 (which owns the ADC and gets its Clocks from proc0):
//! a burst of conversions logged on UART0, at boot and on request
//! (link+0x28 = [`super::ADC_REQUEST`], reply at link+0x2c = the request on success, all-ones on failure).
use crate::rp1;
use rp1_hal::{
    adc::{Blocking, Error, Input, TEMPERATURE},
    log,
    timer::{Timer, WaitTimer},
};
const SAMPLES: u32 = 1000;

type ReadChannel = fn(&mut Blocking<'_, 1>) -> Result<u16, Error>;

// Share the sample loop across channels to fit the combined proof image.
#[inline(never)]
fn burst(adc: &mut Blocking<'_, 1>, read: ReadChannel) -> Result<(u16, u16, u32, u32), Error> {
    let timer = Timer::new();
    let (mut min, mut max, mut sum) = (u16::MAX, 0, 0u32);
    let start = timer.now_32();
    for _ in 0..SAMPLES {
        let v = read(adc)?;
        min = min.min(v);
        max = max.max(v);
        sum += u32::from(v);
    }
    Ok((min, max, sum, (timer.now_32() - start).ticks()))
}

pub fn prove(adc: &mut rp1::Adc, wait: &mut WaitTimer, tag: &str) -> Result<(), Error> {
    let mut adc = adc.blocking(wait);
    // Capture the first conversion after setup. AIN0–3 are passive characterization inputs
    // on this lab board; their voltages and accuracy are not part of the temperature proof.
    let first = adc.read(TEMPERATURE)?;
    let channels: [ReadChannel; 5] = [
        |adc| adc.read(unsafe { Input::<0>::__new() }),
        |adc| adc.read(unsafe { Input::<1>::__new() }),
        |adc| adc.read(unsafe { Input::<2>::__new() }),
        |adc| adc.read(unsafe { Input::<3>::__new() }),
        |adc| adc.read(TEMPERATURE),
    ];
    let mut readings = [(0, 0, 0, 0); 5];
    for (reading, read) in readings.iter_mut().zip(channels) {
        *reading = burst(&mut adc, read)?;
    }
    for (ain, (min, max, sum, us)) in readings.into_iter().enumerate() {
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

pub fn run(adc: Option<&mut rp1::Adc>, wait: &mut WaitTimer, request: u32) {
    let result = match adc {
        Some(adc) => prove(adc, wait, "P1REQ"),
        None => Err(Error::Precondition),
    };
    log!("ADC_RESULT request=0x{:x} {:?}\n", request, result).unwrap();
    super::proof_reply(request, result.is_ok());
}
