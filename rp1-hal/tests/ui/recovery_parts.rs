use rp1_hal::{gpio::Pin, uart::InitError};

fn bypass(error: InitError<2, (Pin<4>, Pin<5>)>) {
    let _ = error.into_parts();
}

fn discard_owner(error: InitError<2, (Pin<4>, Pin<5>)>) -> Result<(), rp1_hal::uart::Error> {
    Err(error)?;
    Ok(())
}

fn main() {}

fn bypass_i2c(error: rp1_hal::i2c::InitError<1, (Pin<2>, Pin<3>)>) {
    let _ = error.into_parts();
}

fn bypass_spi(error: rp1_hal::spi::InitError<0, (Pin<11>, Pin<10>, Pin<9>)>) {
    let _ = error.into_parts();
}
