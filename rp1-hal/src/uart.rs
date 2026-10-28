use crate::{Error, poll, read, write};
use core::fmt;

pub struct Uart<const N: u8> {
    _private: (),
}
pub struct Token<const N: u8> {
    _private: (),
}
impl<const N: u8> Token<N> {
    /// # Safety
    /// Generated resources only: exactly one token for the declared UART owner.
    pub const unsafe fn new() -> Self {
        assert!(N < 6);
        Self { _private: () }
    }
    pub fn init<const TX: u8, const RX: u8, const F: u8>(
        self,
        _tx: crate::gpio::Pin<TX, crate::gpio::Function<F>>,
        _rx: crate::gpio::Pin<RX, crate::gpio::Function<F>>,
        clocks: &crate::clocks::Clocks,
        baud: u32,
    ) -> Result<Uart<N>, Error> {
        write(0x4001_7004, 1 << (26 + N));
        poll(0x4001_401c, 1 << (26 + N), 1 << (26 + N))?;
        unsafe { Uart::new(clocks.uart_hz, baud) }
    }
}
pub fn divisors(clock_hz: u32, baud: u32) -> Result<(u32, u32), Error> {
    if baud == 0 {
        return Err(Error::Invalid);
    }
    let scaled = (u64::from(clock_hz) * 4 + u64::from(baud) / 2) / u64::from(baud);
    if !(64..=(65535 * 64 + 63)).contains(&scaled) {
        return Err(Error::Invalid);
    }
    Ok(((scaled / 64) as u32, (scaled % 64) as u32))
}
impl<const N: u8> Uart<N> {
    /// # Safety
    /// Sole declared owner of UART N; its clocks and pin mux are configured and stable.
    pub unsafe fn new(clock_hz: u32, baud: u32) -> Result<Self, Error> {
        if N > 5 {
            return Err(Error::Invalid);
        }
        let (integer, fraction) = divisors(clock_hz, baud)?;
        let base = 0x4003_0000 + N as usize * 0x4000;
        poll(base + 0x18, 1 << 3, 0)?;
        write(base + 0x30, 0);
        write(base + 0x38, 0);
        write(base + 0x44, 0x7ff);
        write(base + 4, 0);
        write(base + 0x24, integer);
        write(base + 0x28, fraction);
        write(base + 0x2c, 0x70);
        write(base + 0x30, 0x301);
        Ok(Self { _private: () })
    }
    pub fn write_byte(&mut self, byte: u8) -> Result<(), Error> {
        let base = 0x4003_0000 + N as usize * 0x4000;
        poll(base + 0x18, 1 << 5, 0)?;
        write(base, u32::from(byte));
        Ok(())
    }
    pub fn read_byte(&mut self) -> Result<Option<u8>, Error> {
        let base = 0x4003_0000 + N as usize * 0x4000;
        if read(base + 0x18) & (1 << 4) != 0 {
            return Ok(None);
        }
        let data = read(base);
        if data & 0xf00 != 0 {
            write(base + 4, 0);
            return Err(Error::Hardware);
        }
        Ok(Some(data as u8))
    }
    pub fn flush(&mut self) -> Result<(), Error> {
        poll(0x4003_0018 + N as usize * 0x4000, 1 << 3, 0)
    }
}
impl<const N: u8> fmt::Write for Uart<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            self.write_byte(b).map_err(|_| fmt::Error)?;
        }
        Ok(())
    }
}
impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}
impl<const N: u8> embedded_io::ErrorType for Uart<N> {
    type Error = Error;
}
impl<const N: u8> embedded_io::Write for Uart<N> {
    fn write(&mut self, b: &[u8]) -> Result<usize, Error> {
        for byte in b {
            self.write_byte(*byte)?;
        }
        Ok(b.len())
    }
    fn flush(&mut self) -> Result<(), Error> {
        Uart::flush(self)
    }
}
impl<const N: u8> embedded_io::Read for Uart<N> {
    fn read(&mut self, b: &mut [u8]) -> Result<usize, Error> {
        if b.is_empty() {
            return Ok(0);
        }
        for _ in 0..100_000 {
            if let Some(byte) = self.read_byte()? {
                b[0] = byte;
                return Ok(1);
            }
        }
        Err(Error::Timeout)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn baud_divisor_and_invalid_inputs() {
        assert_eq!(divisors(50_000_000, 115200), Ok((27, 8)));
        assert_eq!(divisors(0, 115200), Err(Error::Invalid));
        assert_eq!(divisors(50_000_000, 0), Err(Error::Invalid));
        assert_eq!(divisors(50_000_000, 1), Err(Error::Invalid));
    }
}
