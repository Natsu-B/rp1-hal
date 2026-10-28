//! Clock-protocol message dispatch only. Mailbox transport and Linux clock voting are pending.
pub const CLOCK_PROTOCOL: u8 = 0x14;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Success = 0,
    NotSupported = -1,
    InvalidParameters = -2,
    NotFound = -4,
    HardwareError = -9,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub message: u8,
    pub protocol: u8,
    pub token: u16,
}
impl Header {
    pub fn parse(word: u32) -> Result<Self, Status> {
        if word & 0xf000_0300 != 0 {
            return Err(Status::InvalidParameters);
        }
        Ok(Self {
            message: word as u8,
            protocol: (word >> 10) as u8,
            token: ((word >> 18) & 0x3ff) as u16,
        })
    }
    pub fn word(self) -> u32 {
        u32::from(self.message) | u32::from(self.protocol) << 10 | u32::from(self.token) << 18
    }
}
pub trait ClockBackend {
    fn count(&self) -> u32;
    fn rate(&mut self, id: u32) -> Result<u64, Status>;
    fn enable(&mut self, id: u32, enabled: bool) -> Result<(), Status>;
}
/// Response words exclude the SCMI header. First word is always signed status.
pub fn dispatch(
    backend: &mut impl ClockBackend,
    header: Header,
    request: &[u32],
    response: &mut [u32; 4],
) -> usize {
    response.fill(0);
    let result = (|| {
        if header.protocol != CLOCK_PROTOCOL {
            return Err(Status::NotSupported);
        }
        match header.message {
            0 if request.is_empty() => {
                response[1] = 0x0001_0000;
                Ok(2)
            }
            1 if request.is_empty() => {
                response[1] = backend.count();
                Ok(2)
            }
            2 if request.len() == 1 => {
                if !matches!(request[0], 0 | 1 | 2 | 6 | 7) {
                    return Err(Status::NotSupported);
                }
                Ok(2)
            }
            6 if request.len() == 1 => {
                let rate = backend.rate(request[0])?;
                response[1] = rate as u32;
                response[2] = (rate >> 32) as u32;
                Ok(3)
            }
            7 if request.len() == 2 && request[1] <= 1 => {
                backend.enable(request[0], request[1] == 1)?;
                Ok(1)
            }
            0 | 1 | 2 | 6 | 7 => Err(Status::InvalidParameters),
            _ => Err(Status::NotSupported),
        }
    })();
    match result {
        Ok(n) => n,
        Err(status) => {
            response[0] = status as i32 as u32;
            1
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Backend;
    impl ClockBackend for Backend {
        fn count(&self) -> u32 {
            1
        }
        fn rate(&mut self, id: u32) -> Result<u64, Status> {
            if id == 0 {
                Ok(200_000_000)
            } else {
                Err(Status::NotFound)
            }
        }
        fn enable(&mut self, _: u32, _: bool) -> Result<(), Status> {
            Err(Status::NotSupported)
        }
    }
    #[test]
    fn dispatch_valid_invalid_and_unknown() {
        let mut b = Backend;
        let mut out = [0; 4];
        let h = Header {
            message: 6,
            protocol: CLOCK_PROTOCOL,
            token: 7,
        };
        assert_eq!(Header::parse(h.word()), Ok(h));
        assert_eq!(dispatch(&mut b, h, &[0], &mut out), 3);
        assert_eq!(out[1], 200_000_000);
        assert_eq!(dispatch(&mut b, h, &[], &mut out), 1);
        assert_eq!(out[0], (-2i32) as u32);
        assert_eq!(
            dispatch(&mut b, Header { message: 9, ..h }, &[], &mut out),
            1
        );
        assert_eq!(out[0], u32::MAX);
    }
}
