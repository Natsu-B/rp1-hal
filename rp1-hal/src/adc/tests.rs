use super::*;

#[test]
fn offsets_match_the_pac() {
    assert_eq!(
        pac_offsets!(adc: cs, fcs, fifo, inte, intf, ints).map(|offset| BASE + offset),
        [CS, FCS, FIFO, INTE, INTF, INTS]
    );
    assert_eq!(crate::pac::ADC::PTR as usize, BASE);
}

#[test]
fn clk_adc_is_xosc_div_1() {
    let Kind::Linux(clk) = RP1_CLOCKS[id::ADC as usize].kind else {
        panic!()
    };
    assert_eq!(
        clk.choose(crate::clocks::XOSC_HZ, Round::Down),
        Some((0, 1, 50_000_000))
    );
}

#[test]
fn conversions_follow_linux() {
    assert_eq!(raw_to_millivolts(0, VREF_MV), 0);
    assert_eq!(raw_to_millivolts(0xfff, VREF_MV), 3300);
    assert_eq!(raw_to_millivolts(876, VREF_MV), 705);
    // 0.706 V is 27 °C; 1.721 mV per degree, falling with temperature.
    assert_eq!(millivolts_to_millicelsius(706), 27_000);
    assert_eq!(millivolts_to_millicelsius(706 - 1721 / 100), 36_878);
    assert_eq!(millivolts_to_millicelsius(689), 36_878);
    assert_eq!(millivolts_to_millicelsius(723), 17_122);
    assert_eq!(millivolts_to_millicelsius(0), 437_227);
}

pub(super) struct FakeAdc {
    pub cs: u32,
    pub inte: u32,
    pub intf: u32,
    pub flags: u32,
    pub samples: std::collections::VecDeque<u32>,
    pub on_start: Option<u32>,
    pub starts: u8,
    pub fifo_reads: u8,
    pub hold_fifo: bool,
}

extern crate std;

impl Default for FakeAdc {
    fn default() -> Self {
        Self {
            cs: CS_POWERED | CS_READY,
            inte: 0,
            intf: 0,
            flags: 0,
            samples: Default::default(),
            on_start: None,
            starts: 0,
            fifo_reads: 0,
            hold_fifo: false,
        }
    }
}

impl FakeAdc {
    pub(super) fn complete(&mut self, value: u32) {
        assert_eq!(self.cs & CS_READY, 0);
        self.cs |= CS_READY;
        self.samples.push_back(value);
    }
}

impl Registers for FakeAdc {
    fn read(&mut self, address: usize) -> u32 {
        match address {
            CS => self.cs,
            FCS => {
                (self.samples.len() as u32 & 0xf) << 16
                    | (u32::from(self.samples.is_empty()) * FCS_EMPTY)
                    | self.flags
            }
            INTE => self.inte,
            INTF => self.intf,
            INTS => self.intf | (u32::from(!self.samples.is_empty()) & self.inte),
            FIFO => {
                self.fifo_reads += 1;
                if self.hold_fifo {
                    return self.samples[0];
                }
                self.samples.pop_front().expect("read an empty FIFO")
            }
            _ => panic!("unexpected read at {address:#x}"),
        }
    }

    fn write(&mut self, address: usize, value: u32) {
        match address {
            CS => {
                let status = self.cs & (CS_READY | CS_ERR | CS_ERR_STICKY);
                self.cs = value & !(CS_ERR_STICKY | CS_START_ONCE) | status;
                self.cs &= !(value & CS_ERR_STICKY);
                if value & CS_START_ONCE != 0 {
                    assert!(ready(self.cs));
                    self.cs &= !(CS_READY | CS_ERR);
                    self.starts += 1;
                    if let Some(sample) = self.on_start {
                        self.complete(sample);
                    }
                }
            }
            FCS => {
                assert_eq!(value, FCS_ONE_SAMPLE | FCS_FLAGS);
                self.flags &= !value;
            }
            INTE => self.inte = value,
            INTF => self.intf = value,
            _ => panic!("unexpected write at {address:#x}"),
        }
    }
}

#[test]
fn immediate_start_and_sample_own_one_generation() {
    let mut io = FakeAdc::default();
    let mut state = State::Idle;
    assert_eq!(state.sample(&mut io), Err(Error::NoConversion));
    state.start(&mut io, 4).unwrap();
    assert_eq!(state.sample(&mut io), Ok(None));
    assert_eq!(state.start(&mut io, 3), Err(Error::Busy));
    assert_eq!(state.idle(), Err(Error::Busy));
    io.samples.push_back(41);
    assert_eq!(state.sample(&mut io), Ok(None)); // FIFO populated before READY.
    io.cs |= CS_READY;
    assert_eq!(state.sample(&mut io), Ok(Some(41)));
    assert_eq!(state.sample(&mut io), Err(Error::NoConversion));
    assert_eq!(io.starts, 1);
    assert_eq!(io.fifo_reads, 1);
}

#[test]
fn cancel_waits_for_ready_then_drains_before_rearm() {
    let mut io = FakeAdc::default();
    let mut state = State::Idle;
    state.start(&mut io, 4).unwrap();
    assert_eq!(state.cancel(&mut io), Err(Error::Recovering));
    assert_eq!(state.sample(&mut io), Err(Error::Recovering));
    assert_eq!(state.start(&mut io, 4), Err(Error::Recovering));
    assert_eq!(state.idle(), Err(Error::Recovering));
    io.complete(111);
    state.cancel(&mut io).unwrap();
    assert!(io.samples.is_empty());
    state.start(&mut io, 4).unwrap();
    io.complete(222);
    assert_eq!(state.sample(&mut io), Ok(Some(222)));
}

#[test]
fn error_paths_keep_overrun_conversion_and_fifo_state_separate() {
    for (flags, cs, word, error) in [
        (FCS_OVER, CS_ERR, FIFO_ERR, Error::Overrun),
        (FCS_UNDER, 0, 123, Error::FifoState),
        (0, CS_ERR_STICKY, 123, Error::Conversion),
        (0, CS_ERR, 123, Error::Conversion),
        (0, 0, FIFO_ERR, Error::Conversion),
    ] {
        let mut io = FakeAdc::default();
        let mut state = State::Idle;
        state.start(&mut io, 4).unwrap();
        io.complete(word);
        io.flags = flags;
        io.cs |= cs;
        assert_eq!(state.sample(&mut io), Err(error));
        assert_eq!(io.flags, 0);
        assert_eq!(io.cs & CS_ERR_STICKY, 0);
        assert!(io.samples.is_empty());
        state.start(&mut io, 4).unwrap();
        io.complete(123);
        assert_eq!(state.sample(&mut io), Ok(Some(123)));
    }
}

#[test]
fn stuck_fifo_has_a_fixed_drain_budget_and_keeps_ownership() {
    let mut io = FakeAdc {
        hold_fifo: true,
        ..Default::default()
    };
    io.samples.push_back(234);
    let mut state = State::Idle;
    assert_eq!(state.start(&mut io, 4), Err(Error::Recovering));
    assert_eq!(io.fifo_reads, FIFO_DEPTH as u8);
    assert_eq!(io.starts, 0);
    assert_eq!(state.idle(), Err(Error::Recovering));
}
