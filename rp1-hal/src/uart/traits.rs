//! Stream readiness and opt-in serial and embedded-io compatibility traits.

use super::*;

trait RxReady {
    fn ready_to_read(&self) -> Result<bool, Error>;
}

trait TxReady {
    fn ready_to_write(&self) -> Result<bool, Error>;
}

impl<const N: u8, const PIN: u8, I> RxReady for Rx<N, PIN, I> {
    fn ready_to_read(&self) -> Result<bool, Error> {
        self.state.ready_to_read(N)
    }
}

impl<const N: u8, const PIN: u8, I> TxReady for Tx<N, PIN, I> {
    fn ready_to_write(&self) -> Result<bool, Error> {
        self.state.ready_to_write(N)
    }
}

impl<const N: u8, const TX: u8, const RX: u8, I> RxReady for Uart<N, PinTuple<TX, RX>, I>
where
    Pin<TX>: TxPin<N>,
    Pin<RX>: RxPin<N>,
{
    fn ready_to_read(&self) -> Result<bool, Error> {
        self.halves.1.ready_to_read()
    }
}

impl<const N: u8, const TX: u8, const RX: u8, I> TxReady for Uart<N, PinTuple<TX, RX>, I>
where
    Pin<TX>: TxPin<N>,
    Pin<RX>: RxPin<N>,
{
    fn ready_to_write(&self) -> Result<bool, Error> {
        self.halves.0.ready_to_write()
    }
}

impl RxState {
    pub(super) fn ready_to_read(&self, n: u8) -> Result<bool, Error> {
        if self.resetting {
            return Err(Error::Busy);
        }
        Ok(Mmio.read(base(n) + FR) & FR_RXFE == 0 || Mmio.read(base(n) + RIS) & RX_ERRORS != 0)
    }
}

impl TxState {
    pub(super) fn ready_to_write(&self, n: u8) -> Result<bool, Error> {
        if self.resetting {
            return Err(Error::Busy);
        }
        Ok(Mmio.read(base(n) + FR) & FR_TXFF == 0)
    }
}

impl<T: RxReady> embedded_io::ReadReady for Polling<'_, T> {
    fn read_ready(&mut self) -> Result<bool, Error> {
        self.driver().ready_to_read()
    }
}

impl<T: TxReady> embedded_io::WriteReady for Polling<'_, T> {
    fn write_ready(&mut self) -> Result<bool, Error> {
        self.driver().ready_to_write()
    }
}

#[cfg(feature = "embedded-io-07")]
mod io07 {
    use super::*;
    use embedded_io_07::{ErrorKind, ErrorType, Read, ReadReady, Write, WriteReady};

    impl embedded_io_07::Error for Error {
        fn kind(&self) -> ErrorKind {
            match self {
                Self::Break | Self::Parity | Self::Framing | Self::Overrun => {
                    ErrorKind::InvalidData
                }
                Self::Timeout => ErrorKind::TimedOut,
                Self::Invalid | Self::InvalidDeadline => ErrorKind::InvalidInput,
                _ => ErrorKind::Other,
            }
        }
    }

    macro_rules! forward_io {
        ($adapter:ident<$($arg:ident),+>; $($generic:tt)+) => {
            impl<$($generic)+> ErrorType for $adapter<'_, $($arg),+> {
                type Error = Error;
            }

            impl<$($generic)+> Read for $adapter<'_, $($arg),+>
            where
                Self: embedded_io::Read<Error = Error>,
            {
                fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
                    embedded_io::Read::read(self, buf)
                }
            }

            impl<$($generic)+> Write for $adapter<'_, $($arg),+>
            where
                Self: embedded_io::Write<Error = Error>,
            {
                fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
                    embedded_io::Write::write(self, buf)
                }

                fn flush(&mut self) -> Result<(), Error> {
                    embedded_io::Write::flush(self)
                }
            }

            impl<$($generic)+> ReadReady for $adapter<'_, $($arg),+>
            where
                Self: embedded_io::ReadReady<Error = Error>,
            {
                fn read_ready(&mut self) -> Result<bool, Error> {
                    embedded_io::ReadReady::read_ready(self)
                }
            }

            impl<$($generic)+> WriteReady for $adapter<'_, $($arg),+>
            where
                Self: embedded_io::WriteReady<Error = Error>,
            {
                fn write_ready(&mut self) -> Result<bool, Error> {
                    embedded_io::WriteReady::write_ready(self)
                }
            }
        };
    }

    forward_io!(Polling<T>; T);
    forward_io!(Blocking<N, CORE>; const CORE: u8, const N: u8);
    forward_io!(BlockingRx<N, CORE>; const CORE: u8, const N: u8);
    forward_io!(BlockingTx<N, CORE>; const CORE: u8, const N: u8);
}

#[cfg(any(feature = "eh-nb", feature = "eh02"))]
mod nonblocking {
    use super::*;

    fn received(value: Result<Option<u8>, Error>) -> nb::Result<u8, Error> {
        value
            .map_err(nb::Error::Other)?
            .ok_or(nb::Error::WouldBlock)
    }

    fn written(ready: Result<bool, Error>) -> nb::Result<(), Error> {
        if ready.map_err(nb::Error::Other)? {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    #[cfg(feature = "eh-nb")]
    mod modern {
        use super::*;
        use embedded_hal_nb::serial::{ErrorKind, ErrorType, Read, Write};

        impl embedded_hal_nb::serial::Error for Error {
            fn kind(&self) -> ErrorKind {
                match self {
                    Self::Overrun => ErrorKind::Overrun,
                    Self::Parity => ErrorKind::Parity,
                    Self::Break | Self::Framing => ErrorKind::FrameFormat,
                    _ => ErrorKind::Other,
                }
            }
        }

        impl<const N: u8, const PIN: u8, I> ErrorType for Rx<N, PIN, I> {
            type Error = Error;
        }
        impl<const N: u8, const PIN: u8, I> ErrorType for Tx<N, PIN, I> {
            type Error = Error;
        }
        impl<const N: u8, const TX: u8, const RX: u8, I> ErrorType for Uart<N, PinTuple<TX, RX>, I>
        where
            Pin<TX>: TxPin<N>,
            Pin<RX>: RxPin<N>,
        {
            type Error = Error;
        }

        impl_serial!();
    }

    #[cfg(feature = "eh02")]
    mod legacy {
        use super::*;
        use embedded_hal_0_2::serial::{Read, Write};
        impl_serial!(
            type Error = Error;
        );
    }

    // Both serial versions have the same methods. Only 0.2 declares Error on
    // each operation trait instead of an ErrorType supertrait.
    macro_rules! impl_serial {
        ($($error:tt)*) => {
            impl<const N: u8, const PIN: u8, I> Read<u8> for Rx<N, PIN, I> {
                $($error)*
                fn read(&mut self) -> nb::Result<u8, Error> {
                    received(self.try_read_byte())
                }
            }

            impl<const N: u8, const PIN: u8, I> Write<u8> for Tx<N, PIN, I> {
                $($error)*
                fn write(&mut self, word: u8) -> nb::Result<(), Error> {
                    written(self.try_write_byte(word))
                }
                fn flush(&mut self) -> nb::Result<(), Error> {
                    written(self.try_flush())
                }
            }

            impl<const N: u8, const TX: u8, const RX: u8, I> Read<u8> for Uart<N, PinTuple<TX, RX>, I>
            where
                Pin<TX>: TxPin<N>,
                Pin<RX>: RxPin<N>,
            {
                $($error)*
                fn read(&mut self) -> nb::Result<u8, Error> {
                    received(self.try_read_byte())
                }
            }

            impl<const N: u8, const TX: u8, const RX: u8, I> Write<u8> for Uart<N, PinTuple<TX, RX>, I>
            where
                Pin<TX>: TxPin<N>,
                Pin<RX>: RxPin<N>,
            {
                $($error)*
                fn write(&mut self, word: u8) -> nb::Result<(), Error> {
                    written(self.try_write_byte(word))
                }
                fn flush(&mut self) -> nb::Result<(), Error> {
                    written(self.try_flush())
                }
            }
        };
    }
    use impl_serial;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_trait_surface_covers_uart_and_both_halves() {
        fn read<T: embedded_io::Read<Error = Error> + embedded_io::ReadReady>() {}
        fn write<T: embedded_io::Write<Error = Error> + embedded_io::WriteReady + fmt::Write>() {}
        read::<Polling<'_, Rx<2, 5>>>();
        write::<Polling<'_, Tx<2, 4>>>();
        read::<Polling<'_, Uart<2, PinTuple<4, 5>>>>();
        write::<Polling<'_, Uart<2, PinTuple<4, 5>>>>();
        read::<BlockingRx<'_, 1, 0>>();
        write::<BlockingTx<'_, 1, 0>>();
        read::<Blocking<'_, 1, 0>>();
        write::<Blocking<'_, 1, 0>>();
    }

    #[cfg(feature = "embedded-io-07")]
    #[test]
    fn both_versions_are_available_on_the_same_adapter() {
        fn read<T: embedded_io::Read + embedded_io_07::Read + embedded_io_07::ReadReady>() {}
        fn write<T: embedded_io::Write + embedded_io_07::Write + embedded_io_07::WriteReady>() {}
        read::<BlockingRx<'_, 1, 0>>();
        write::<BlockingTx<'_, 1, 0>>();
        read::<Polling<'_, Uart<2, PinTuple<4, 5>>>>();
        write::<Polling<'_, Uart<2, PinTuple<4, 5>>>>();
    }
}
