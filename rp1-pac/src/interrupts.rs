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

#[cfg(all(feature = "rt", target_arch = "arm", target_os = "none"))]
#[allow(non_snake_case)]
unsafe extern "C" {
    fn __rp1_default_irq();
    fn __rp1_p0_IO_BANK0();
    fn __rp1_p0_IO_BANK1();
    fn __rp1_p0_IO_BANK2();
    fn __rp1_p0_AUDIO_IN();
    fn __rp1_p0_AUDIO_OUT();
    fn __rp1_p0_PWM0();
    fn __rp1_p0_ETH();
    fn __rp1_p0_I2C0();
    fn __rp1_p0_I2C1();
    fn __rp1_p0_I2C2();
    fn __rp1_p0_I2C3();
    fn __rp1_p0_I2C4();
    fn __rp1_p0_I2C5();
    fn __rp1_p0_I2C6();
    fn __rp1_p0_I2S0();
    fn __rp1_p0_I2S1();
    fn __rp1_p0_I2S2();
    fn __rp1_p0_SDIO0();
    fn __rp1_p0_SDIO1();
    fn __rp1_p0_SPI0();
    fn __rp1_p0_SPI1();
    fn __rp1_p0_SPI2();
    fn __rp1_p0_SPI3();
    fn __rp1_p0_SPI4();
    fn __rp1_p0_SPI5();
    fn __rp1_p0_UART0();
    fn __rp1_p0_TIMER_0();
    fn __rp1_p0_TIMER_1();
    fn __rp1_p0_TIMER_2();
    fn __rp1_p0_TIMER_3();
    fn __rp1_p0_USBHOST0();
    fn __rp1_p0_USBHOST0_0();
    fn __rp1_p0_USBHOST0_1();
    fn __rp1_p0_USBHOST0_2();
    fn __rp1_p0_USBHOST0_3();
    fn __rp1_p0_USBHOST1();
    fn __rp1_p0_USBHOST1_0();
    fn __rp1_p0_USBHOST1_1();
    fn __rp1_p0_USBHOST1_2();
    fn __rp1_p0_USBHOST1_3();
    fn __rp1_p0_DMA();
    fn __rp1_p0_PWM1();
    fn __rp1_p0_UART1();
    fn __rp1_p0_UART2();
    fn __rp1_p0_UART3();
    fn __rp1_p0_UART4();
    fn __rp1_p0_UART5();
    fn __rp1_p0_MIPI0();
    fn __rp1_p0_MIPI1();
    fn __rp1_p0_VIDEO_OUT();
    fn __rp1_p0_PIO_0();
    fn __rp1_p0_PIO_1();
    fn __rp1_p0_ADC_FIFO();
    fn __rp1_p0_PCIE_OUT();
    fn __rp1_p0_SPI6();
    fn __rp1_p0_SPI7();
    fn __rp1_p0_SPI8();
    fn __rp1_p0_PROC_MISC();
    fn __rp1_p0_SYSCFG();
    fn __rp1_p0_CLOCKS_DEFAULT();
    fn __rp1_p0_VBUSCTRL();
    fn __rp1_p1_IO_BANK0();
    fn __rp1_p1_IO_BANK1();
    fn __rp1_p1_IO_BANK2();
    fn __rp1_p1_AUDIO_IN();
    fn __rp1_p1_AUDIO_OUT();
    fn __rp1_p1_PWM0();
    fn __rp1_p1_ETH();
    fn __rp1_p1_I2C0();
    fn __rp1_p1_I2C1();
    fn __rp1_p1_I2C2();
    fn __rp1_p1_I2C3();
    fn __rp1_p1_I2C4();
    fn __rp1_p1_I2C5();
    fn __rp1_p1_I2C6();
    fn __rp1_p1_I2S0();
    fn __rp1_p1_I2S1();
    fn __rp1_p1_I2S2();
    fn __rp1_p1_SDIO0();
    fn __rp1_p1_SDIO1();
    fn __rp1_p1_SPI0();
    fn __rp1_p1_SPI1();
    fn __rp1_p1_SPI2();
    fn __rp1_p1_SPI3();
    fn __rp1_p1_SPI4();
    fn __rp1_p1_SPI5();
    fn __rp1_p1_UART0();
    fn __rp1_p1_TIMER_0();
    fn __rp1_p1_TIMER_1();
    fn __rp1_p1_TIMER_2();
    fn __rp1_p1_TIMER_3();
    fn __rp1_p1_USBHOST0();
    fn __rp1_p1_USBHOST0_0();
    fn __rp1_p1_USBHOST0_1();
    fn __rp1_p1_USBHOST0_2();
    fn __rp1_p1_USBHOST0_3();
    fn __rp1_p1_USBHOST1();
    fn __rp1_p1_USBHOST1_0();
    fn __rp1_p1_USBHOST1_1();
    fn __rp1_p1_USBHOST1_2();
    fn __rp1_p1_USBHOST1_3();
    fn __rp1_p1_DMA();
    fn __rp1_p1_PWM1();
    fn __rp1_p1_UART1();
    fn __rp1_p1_UART2();
    fn __rp1_p1_UART3();
    fn __rp1_p1_UART4();
    fn __rp1_p1_UART5();
    fn __rp1_p1_MIPI0();
    fn __rp1_p1_MIPI1();
    fn __rp1_p1_VIDEO_OUT();
    fn __rp1_p1_PIO_0();
    fn __rp1_p1_PIO_1();
    fn __rp1_p1_ADC_FIFO();
    fn __rp1_p1_PCIE_OUT();
    fn __rp1_p1_SPI6();
    fn __rp1_p1_SPI7();
    fn __rp1_p1_SPI8();
    fn __rp1_p1_PROC_MISC();
    fn __rp1_p1_SYSCFG();
    fn __rp1_p1_CLOCKS_DEFAULT();
    fn __rp1_p1_VBUSCTRL();
}

#[cfg(all(feature = "rt", target_arch = "arm", target_os = "none"))]
#[doc(hidden)]
#[repr(C)]
pub union Vector {
    _handler: unsafe extern "C" fn(),
    _reserved: u32,
}

#[cfg(all(feature = "rt", target_arch = "arm", target_os = "none"))]
#[doc(hidden)]
#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".vectors.proc0.irq")]
pub static __RP1_IRQS0: [Vector; NVIC_INTERRUPT_COUNT] = [
    // IRQ 0: IO_BANK0
    Vector {
        _handler: __rp1_p0_IO_BANK0,
    },
    // IRQ 1: IO_BANK1
    Vector {
        _handler: __rp1_p0_IO_BANK1,
    },
    // IRQ 2: IO_BANK2
    Vector {
        _handler: __rp1_p0_IO_BANK2,
    },
    // IRQ 3: AUDIO_IN
    Vector {
        _handler: __rp1_p0_AUDIO_IN,
    },
    // IRQ 4: AUDIO_OUT
    Vector {
        _handler: __rp1_p0_AUDIO_OUT,
    },
    // IRQ 5: PWM0
    Vector {
        _handler: __rp1_p0_PWM0,
    },
    // IRQ 6: ETH
    Vector {
        _handler: __rp1_p0_ETH,
    },
    // IRQ 7: I2C0
    Vector {
        _handler: __rp1_p0_I2C0,
    },
    // IRQ 8: I2C1
    Vector {
        _handler: __rp1_p0_I2C1,
    },
    // IRQ 9: I2C2
    Vector {
        _handler: __rp1_p0_I2C2,
    },
    // IRQ 10: I2C3
    Vector {
        _handler: __rp1_p0_I2C3,
    },
    // IRQ 11: I2C4
    Vector {
        _handler: __rp1_p0_I2C4,
    },
    // IRQ 12: I2C5
    Vector {
        _handler: __rp1_p0_I2C5,
    },
    // IRQ 13: I2C6
    Vector {
        _handler: __rp1_p0_I2C6,
    },
    // IRQ 14: I2S0
    Vector {
        _handler: __rp1_p0_I2S0,
    },
    // IRQ 15: I2S1
    Vector {
        _handler: __rp1_p0_I2S1,
    },
    // IRQ 16: I2S2
    Vector {
        _handler: __rp1_p0_I2S2,
    },
    // IRQ 17: SDIO0
    Vector {
        _handler: __rp1_p0_SDIO0,
    },
    // IRQ 18: SDIO1
    Vector {
        _handler: __rp1_p0_SDIO1,
    },
    // IRQ 19: SPI0
    Vector {
        _handler: __rp1_p0_SPI0,
    },
    // IRQ 20: SPI1
    Vector {
        _handler: __rp1_p0_SPI1,
    },
    // IRQ 21: SPI2
    Vector {
        _handler: __rp1_p0_SPI2,
    },
    // IRQ 22: SPI3
    Vector {
        _handler: __rp1_p0_SPI3,
    },
    // IRQ 23: SPI4
    Vector {
        _handler: __rp1_p0_SPI4,
    },
    // IRQ 24: SPI5
    Vector {
        _handler: __rp1_p0_SPI5,
    },
    // IRQ 25: UART0
    Vector {
        _handler: __rp1_p0_UART0,
    },
    // IRQ 26: TIMER_0
    Vector {
        _handler: __rp1_p0_TIMER_0,
    },
    // IRQ 27: TIMER_1
    Vector {
        _handler: __rp1_p0_TIMER_1,
    },
    // IRQ 28: TIMER_2
    Vector {
        _handler: __rp1_p0_TIMER_2,
    },
    // IRQ 29: TIMER_3
    Vector {
        _handler: __rp1_p0_TIMER_3,
    },
    // IRQ 30: USBHOST0
    Vector {
        _handler: __rp1_p0_USBHOST0,
    },
    // IRQ 31: USBHOST0_0
    Vector {
        _handler: __rp1_p0_USBHOST0_0,
    },
    // IRQ 32: USBHOST0_1
    Vector {
        _handler: __rp1_p0_USBHOST0_1,
    },
    // IRQ 33: USBHOST0_2
    Vector {
        _handler: __rp1_p0_USBHOST0_2,
    },
    // IRQ 34: USBHOST0_3
    Vector {
        _handler: __rp1_p0_USBHOST0_3,
    },
    // IRQ 35: USBHOST1
    Vector {
        _handler: __rp1_p0_USBHOST1,
    },
    // IRQ 36: USBHOST1_0
    Vector {
        _handler: __rp1_p0_USBHOST1_0,
    },
    // IRQ 37: USBHOST1_1
    Vector {
        _handler: __rp1_p0_USBHOST1_1,
    },
    // IRQ 38: USBHOST1_2
    Vector {
        _handler: __rp1_p0_USBHOST1_2,
    },
    // IRQ 39: USBHOST1_3
    Vector {
        _handler: __rp1_p0_USBHOST1_3,
    },
    // IRQ 40: DMA
    Vector {
        _handler: __rp1_p0_DMA,
    },
    // IRQ 41: PWM1
    Vector {
        _handler: __rp1_p0_PWM1,
    },
    // IRQ 42: UART1
    Vector {
        _handler: __rp1_p0_UART1,
    },
    // IRQ 43: UART2
    Vector {
        _handler: __rp1_p0_UART2,
    },
    // IRQ 44: UART3
    Vector {
        _handler: __rp1_p0_UART3,
    },
    // IRQ 45: UART4
    Vector {
        _handler: __rp1_p0_UART4,
    },
    // IRQ 46: UART5
    Vector {
        _handler: __rp1_p0_UART5,
    },
    // IRQ 47: MIPI0
    Vector {
        _handler: __rp1_p0_MIPI0,
    },
    // IRQ 48: MIPI1
    Vector {
        _handler: __rp1_p0_MIPI1,
    },
    // IRQ 49: VIDEO_OUT
    Vector {
        _handler: __rp1_p0_VIDEO_OUT,
    },
    // IRQ 50: PIO_0
    Vector {
        _handler: __rp1_p0_PIO_0,
    },
    // IRQ 51: PIO_1
    Vector {
        _handler: __rp1_p0_PIO_1,
    },
    // IRQ 52: ADC_FIFO
    Vector {
        _handler: __rp1_p0_ADC_FIFO,
    },
    // IRQ 53: PCIE_OUT
    Vector {
        _handler: __rp1_p0_PCIE_OUT,
    },
    // IRQ 54: SPI6
    Vector {
        _handler: __rp1_p0_SPI6,
    },
    // IRQ 55: SPI7
    Vector {
        _handler: __rp1_p0_SPI7,
    },
    // IRQ 56: SPI8
    Vector {
        _handler: __rp1_p0_SPI8,
    },
    // IRQ 57: PROC_MISC
    Vector {
        _handler: __rp1_p0_PROC_MISC,
    },
    // IRQ 58: SYSCFG
    Vector {
        _handler: __rp1_p0_SYSCFG,
    },
    // IRQ 59: CLOCKS_DEFAULT
    Vector {
        _handler: __rp1_p0_CLOCKS_DEFAULT,
    },
    // IRQ 60: VBUSCTRL
    Vector {
        _handler: __rp1_p0_VBUSCTRL,
    },
    // IRQ 61: reserved
    Vector {
        _handler: __rp1_default_irq,
    },
    // IRQ 62: reserved
    Vector {
        _handler: __rp1_default_irq,
    },
    // IRQ 63: reserved
    Vector {
        _handler: __rp1_default_irq,
    },
];

#[cfg(all(feature = "rt", target_arch = "arm", target_os = "none"))]
#[doc(hidden)]
#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".vectors.proc1.irq")]
pub static __RP1_IRQS1: [Vector; NVIC_INTERRUPT_COUNT] = [
    // IRQ 0: IO_BANK0
    Vector {
        _handler: __rp1_p1_IO_BANK0,
    },
    // IRQ 1: IO_BANK1
    Vector {
        _handler: __rp1_p1_IO_BANK1,
    },
    // IRQ 2: IO_BANK2
    Vector {
        _handler: __rp1_p1_IO_BANK2,
    },
    // IRQ 3: AUDIO_IN
    Vector {
        _handler: __rp1_p1_AUDIO_IN,
    },
    // IRQ 4: AUDIO_OUT
    Vector {
        _handler: __rp1_p1_AUDIO_OUT,
    },
    // IRQ 5: PWM0
    Vector {
        _handler: __rp1_p1_PWM0,
    },
    // IRQ 6: ETH
    Vector {
        _handler: __rp1_p1_ETH,
    },
    // IRQ 7: I2C0
    Vector {
        _handler: __rp1_p1_I2C0,
    },
    // IRQ 8: I2C1
    Vector {
        _handler: __rp1_p1_I2C1,
    },
    // IRQ 9: I2C2
    Vector {
        _handler: __rp1_p1_I2C2,
    },
    // IRQ 10: I2C3
    Vector {
        _handler: __rp1_p1_I2C3,
    },
    // IRQ 11: I2C4
    Vector {
        _handler: __rp1_p1_I2C4,
    },
    // IRQ 12: I2C5
    Vector {
        _handler: __rp1_p1_I2C5,
    },
    // IRQ 13: I2C6
    Vector {
        _handler: __rp1_p1_I2C6,
    },
    // IRQ 14: I2S0
    Vector {
        _handler: __rp1_p1_I2S0,
    },
    // IRQ 15: I2S1
    Vector {
        _handler: __rp1_p1_I2S1,
    },
    // IRQ 16: I2S2
    Vector {
        _handler: __rp1_p1_I2S2,
    },
    // IRQ 17: SDIO0
    Vector {
        _handler: __rp1_p1_SDIO0,
    },
    // IRQ 18: SDIO1
    Vector {
        _handler: __rp1_p1_SDIO1,
    },
    // IRQ 19: SPI0
    Vector {
        _handler: __rp1_p1_SPI0,
    },
    // IRQ 20: SPI1
    Vector {
        _handler: __rp1_p1_SPI1,
    },
    // IRQ 21: SPI2
    Vector {
        _handler: __rp1_p1_SPI2,
    },
    // IRQ 22: SPI3
    Vector {
        _handler: __rp1_p1_SPI3,
    },
    // IRQ 23: SPI4
    Vector {
        _handler: __rp1_p1_SPI4,
    },
    // IRQ 24: SPI5
    Vector {
        _handler: __rp1_p1_SPI5,
    },
    // IRQ 25: UART0
    Vector {
        _handler: __rp1_p1_UART0,
    },
    // IRQ 26: TIMER_0
    Vector {
        _handler: __rp1_p1_TIMER_0,
    },
    // IRQ 27: TIMER_1
    Vector {
        _handler: __rp1_p1_TIMER_1,
    },
    // IRQ 28: TIMER_2
    Vector {
        _handler: __rp1_p1_TIMER_2,
    },
    // IRQ 29: TIMER_3
    Vector {
        _handler: __rp1_p1_TIMER_3,
    },
    // IRQ 30: USBHOST0
    Vector {
        _handler: __rp1_p1_USBHOST0,
    },
    // IRQ 31: USBHOST0_0
    Vector {
        _handler: __rp1_p1_USBHOST0_0,
    },
    // IRQ 32: USBHOST0_1
    Vector {
        _handler: __rp1_p1_USBHOST0_1,
    },
    // IRQ 33: USBHOST0_2
    Vector {
        _handler: __rp1_p1_USBHOST0_2,
    },
    // IRQ 34: USBHOST0_3
    Vector {
        _handler: __rp1_p1_USBHOST0_3,
    },
    // IRQ 35: USBHOST1
    Vector {
        _handler: __rp1_p1_USBHOST1,
    },
    // IRQ 36: USBHOST1_0
    Vector {
        _handler: __rp1_p1_USBHOST1_0,
    },
    // IRQ 37: USBHOST1_1
    Vector {
        _handler: __rp1_p1_USBHOST1_1,
    },
    // IRQ 38: USBHOST1_2
    Vector {
        _handler: __rp1_p1_USBHOST1_2,
    },
    // IRQ 39: USBHOST1_3
    Vector {
        _handler: __rp1_p1_USBHOST1_3,
    },
    // IRQ 40: DMA
    Vector {
        _handler: __rp1_p1_DMA,
    },
    // IRQ 41: PWM1
    Vector {
        _handler: __rp1_p1_PWM1,
    },
    // IRQ 42: UART1
    Vector {
        _handler: __rp1_p1_UART1,
    },
    // IRQ 43: UART2
    Vector {
        _handler: __rp1_p1_UART2,
    },
    // IRQ 44: UART3
    Vector {
        _handler: __rp1_p1_UART3,
    },
    // IRQ 45: UART4
    Vector {
        _handler: __rp1_p1_UART4,
    },
    // IRQ 46: UART5
    Vector {
        _handler: __rp1_p1_UART5,
    },
    // IRQ 47: MIPI0
    Vector {
        _handler: __rp1_p1_MIPI0,
    },
    // IRQ 48: MIPI1
    Vector {
        _handler: __rp1_p1_MIPI1,
    },
    // IRQ 49: VIDEO_OUT
    Vector {
        _handler: __rp1_p1_VIDEO_OUT,
    },
    // IRQ 50: PIO_0
    Vector {
        _handler: __rp1_p1_PIO_0,
    },
    // IRQ 51: PIO_1
    Vector {
        _handler: __rp1_p1_PIO_1,
    },
    // IRQ 52: ADC_FIFO
    Vector {
        _handler: __rp1_p1_ADC_FIFO,
    },
    // IRQ 53: PCIE_OUT
    Vector {
        _handler: __rp1_p1_PCIE_OUT,
    },
    // IRQ 54: SPI6
    Vector {
        _handler: __rp1_p1_SPI6,
    },
    // IRQ 55: SPI7
    Vector {
        _handler: __rp1_p1_SPI7,
    },
    // IRQ 56: SPI8
    Vector {
        _handler: __rp1_p1_SPI8,
    },
    // IRQ 57: PROC_MISC
    Vector {
        _handler: __rp1_p1_PROC_MISC,
    },
    // IRQ 58: SYSCFG
    Vector {
        _handler: __rp1_p1_SYSCFG,
    },
    // IRQ 59: CLOCKS_DEFAULT
    Vector {
        _handler: __rp1_p1_CLOCKS_DEFAULT,
    },
    // IRQ 60: VBUSCTRL
    Vector {
        _handler: __rp1_p1_VBUSCTRL,
    },
    // IRQ 61: reserved
    Vector {
        _handler: __rp1_default_irq,
    },
    // IRQ 62: reserved
    Vector {
        _handler: __rp1_default_irq,
    },
    // IRQ 63: reserved
    Vector {
        _handler: __rp1_default_irq,
    },
];
