use rp1_hal::{gpio::Pin, uart::InitError};

fn bypass(error: InitError<2, (Pin<4>, Pin<5>)>) {
    let _ = error.into_parts();
}

fn discard_owner(error: InitError<2, (Pin<4>, Pin<5>)>) -> Result<(), rp1_hal::uart::Error> {
    Err(error)?;
    Ok(())
}

fn main() {}
