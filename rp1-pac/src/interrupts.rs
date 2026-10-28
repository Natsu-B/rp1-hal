// Generated from rp1-regdb/data/device.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

pub const NVIC_PRIO_BITS: u8 = 3;
pub const NVIC_INTERRUPT_COUNT: usize = 64;
pub const DEFINED_INTERRUPT_COUNT: usize = 61;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum Interrupt {
    IO_BANK0 = 0,
    IO_BANK1 = 1,
    IO_BANK2 = 2,
    AUDIO_IN = 3,
    AUDIO_OUT = 4,
    PWM0 = 5,
    ETH = 6,
    I2C0 = 7,
    I2C1 = 8,
    I2C2 = 9,
    I2C3 = 10,
    I2C4 = 11,
    I2C5 = 12,
    I2C6 = 13,
    I2S0 = 14,
    I2S1 = 15,
    I2S2 = 16,
    SDIO0 = 17,
    SDIO1 = 18,
    SPI0 = 19,
    SPI1 = 20,
    SPI2 = 21,
    SPI3 = 22,
    SPI4 = 23,
    SPI5 = 24,
    UART0 = 25,
    TIMER_0 = 26,
    TIMER_1 = 27,
    TIMER_2 = 28,
    TIMER_3 = 29,
    USBHOST0 = 30,
    USBHOST0_0 = 31,
    USBHOST0_1 = 32,
    USBHOST0_2 = 33,
    USBHOST0_3 = 34,
    USBHOST1 = 35,
    USBHOST1_0 = 36,
    USBHOST1_1 = 37,
    USBHOST1_2 = 38,
    USBHOST1_3 = 39,
    DMA = 40,
    PWM1 = 41,
    UART1 = 42,
    UART2 = 43,
    UART3 = 44,
    UART4 = 45,
    UART5 = 46,
    MIPI0 = 47,
    MIPI1 = 48,
    VIDEO_OUT = 49,
    PIO_0 = 50,
    PIO_1 = 51,
    ADC_FIFO = 52,
    PCIE_OUT = 53,
    SPI6 = 54,
    SPI7 = 55,
    SPI8 = 56,
    PROC_MISC = 57,
    SYSCFG = 58,
    CLOCKS_DEFAULT = 59,
    VBUSCTRL = 60,
}

unsafe impl cortex_m::interrupt::InterruptNumber for Interrupt {
    #[inline(always)]
    fn number(self) -> u16 {
        self as u16
    }
}

#[cfg(feature = "vectors")]
#[allow(non_snake_case)]
unsafe extern "C" {
    fn IO_BANK0();
    fn IO_BANK1();
    fn IO_BANK2();
    fn AUDIO_IN();
    fn AUDIO_OUT();
    fn PWM0();
    fn ETH();
    fn I2C0();
    fn I2C1();
    fn I2C2();
    fn I2C3();
    fn I2C4();
    fn I2C5();
    fn I2C6();
    fn I2S0();
    fn I2S1();
    fn I2S2();
    fn SDIO0();
    fn SDIO1();
    fn SPI0();
    fn SPI1();
    fn SPI2();
    fn SPI3();
    fn SPI4();
    fn SPI5();
    fn UART0();
    fn TIMER_0();
    fn TIMER_1();
    fn TIMER_2();
    fn TIMER_3();
    fn USBHOST0();
    fn USBHOST0_0();
    fn USBHOST0_1();
    fn USBHOST0_2();
    fn USBHOST0_3();
    fn USBHOST1();
    fn USBHOST1_0();
    fn USBHOST1_1();
    fn USBHOST1_2();
    fn USBHOST1_3();
    fn DMA();
    fn PWM1();
    fn UART1();
    fn UART2();
    fn UART3();
    fn UART4();
    fn UART5();
    fn MIPI0();
    fn MIPI1();
    fn VIDEO_OUT();
    fn PIO_0();
    fn PIO_1();
    fn ADC_FIFO();
    fn PCIE_OUT();
    fn SPI6();
    fn SPI7();
    fn SPI8();
    fn PROC_MISC();
    fn SYSCFG();
    fn CLOCKS_DEFAULT();
    fn VBUSCTRL();
}

#[cfg(feature = "vectors")]
#[doc(hidden)]
#[repr(C)]
pub union Vector {
    _handler: unsafe extern "C" fn(),
    _reserved: u32,
}

#[cfg(feature = "vectors")]
#[doc(hidden)]
#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".vector_table.interrupts")]
pub static __INTERRUPTS: [Vector; NVIC_INTERRUPT_COUNT] = [
    // IRQ 0: IO_BANK0
    Vector { _handler: IO_BANK0 },
    // IRQ 1: IO_BANK1
    Vector { _handler: IO_BANK1 },
    // IRQ 2: IO_BANK2
    Vector { _handler: IO_BANK2 },
    // IRQ 3: AUDIO_IN
    Vector { _handler: AUDIO_IN },
    // IRQ 4: AUDIO_OUT
    Vector {
        _handler: AUDIO_OUT,
    },
    // IRQ 5: PWM0
    Vector { _handler: PWM0 },
    // IRQ 6: ETH
    Vector { _handler: ETH },
    // IRQ 7: I2C0
    Vector { _handler: I2C0 },
    // IRQ 8: I2C1
    Vector { _handler: I2C1 },
    // IRQ 9: I2C2
    Vector { _handler: I2C2 },
    // IRQ 10: I2C3
    Vector { _handler: I2C3 },
    // IRQ 11: I2C4
    Vector { _handler: I2C4 },
    // IRQ 12: I2C5
    Vector { _handler: I2C5 },
    // IRQ 13: I2C6
    Vector { _handler: I2C6 },
    // IRQ 14: I2S0
    Vector { _handler: I2S0 },
    // IRQ 15: I2S1
    Vector { _handler: I2S1 },
    // IRQ 16: I2S2
    Vector { _handler: I2S2 },
    // IRQ 17: SDIO0
    Vector { _handler: SDIO0 },
    // IRQ 18: SDIO1
    Vector { _handler: SDIO1 },
    // IRQ 19: SPI0
    Vector { _handler: SPI0 },
    // IRQ 20: SPI1
    Vector { _handler: SPI1 },
    // IRQ 21: SPI2
    Vector { _handler: SPI2 },
    // IRQ 22: SPI3
    Vector { _handler: SPI3 },
    // IRQ 23: SPI4
    Vector { _handler: SPI4 },
    // IRQ 24: SPI5
    Vector { _handler: SPI5 },
    // IRQ 25: UART0
    Vector { _handler: UART0 },
    // IRQ 26: TIMER_0
    Vector { _handler: TIMER_0 },
    // IRQ 27: TIMER_1
    Vector { _handler: TIMER_1 },
    // IRQ 28: TIMER_2
    Vector { _handler: TIMER_2 },
    // IRQ 29: TIMER_3
    Vector { _handler: TIMER_3 },
    // IRQ 30: USBHOST0
    Vector { _handler: USBHOST0 },
    // IRQ 31: USBHOST0_0
    Vector {
        _handler: USBHOST0_0,
    },
    // IRQ 32: USBHOST0_1
    Vector {
        _handler: USBHOST0_1,
    },
    // IRQ 33: USBHOST0_2
    Vector {
        _handler: USBHOST0_2,
    },
    // IRQ 34: USBHOST0_3
    Vector {
        _handler: USBHOST0_3,
    },
    // IRQ 35: USBHOST1
    Vector { _handler: USBHOST1 },
    // IRQ 36: USBHOST1_0
    Vector {
        _handler: USBHOST1_0,
    },
    // IRQ 37: USBHOST1_1
    Vector {
        _handler: USBHOST1_1,
    },
    // IRQ 38: USBHOST1_2
    Vector {
        _handler: USBHOST1_2,
    },
    // IRQ 39: USBHOST1_3
    Vector {
        _handler: USBHOST1_3,
    },
    // IRQ 40: DMA
    Vector { _handler: DMA },
    // IRQ 41: PWM1
    Vector { _handler: PWM1 },
    // IRQ 42: UART1
    Vector { _handler: UART1 },
    // IRQ 43: UART2
    Vector { _handler: UART2 },
    // IRQ 44: UART3
    Vector { _handler: UART3 },
    // IRQ 45: UART4
    Vector { _handler: UART4 },
    // IRQ 46: UART5
    Vector { _handler: UART5 },
    // IRQ 47: MIPI0
    Vector { _handler: MIPI0 },
    // IRQ 48: MIPI1
    Vector { _handler: MIPI1 },
    // IRQ 49: VIDEO_OUT
    Vector {
        _handler: VIDEO_OUT,
    },
    // IRQ 50: PIO_0
    Vector { _handler: PIO_0 },
    // IRQ 51: PIO_1
    Vector { _handler: PIO_1 },
    // IRQ 52: ADC_FIFO
    Vector { _handler: ADC_FIFO },
    // IRQ 53: PCIE_OUT
    Vector { _handler: PCIE_OUT },
    // IRQ 54: SPI6
    Vector { _handler: SPI6 },
    // IRQ 55: SPI7
    Vector { _handler: SPI7 },
    // IRQ 56: SPI8
    Vector { _handler: SPI8 },
    // IRQ 57: PROC_MISC
    Vector {
        _handler: PROC_MISC,
    },
    // IRQ 58: SYSCFG
    Vector { _handler: SYSCFG },
    // IRQ 59: CLOCKS_DEFAULT
    Vector {
        _handler: CLOCKS_DEFAULT,
    },
    // IRQ 60: VBUSCTRL
    Vector { _handler: VBUSCTRL },
    // IRQ 61: reserved
    Vector { _reserved: 0 },
    // IRQ 62: reserved
    Vector { _reserved: 0 },
    // IRQ 63: reserved
    Vector { _reserved: 0 },
];
