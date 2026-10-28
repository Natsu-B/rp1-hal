// Generated from rp1-regdb/data/peripherals/clocks-main.toml.
// Do not edit manually.
// Regenerate with: cargo run -p rp1-regdb -- generate

use crate::Reg;

/// RP1 CLOCKS_MAIN register block with supplemental register semantics. Alias behavior: normal read/write access. Register layout recovered from the Linux RP1 common-clock driver. Unknown offsets remain split RAW arrays.
/// Atomic view: Rp1Native alias windows; ownership remains with this logical peripheral.
#[repr(C)]
pub struct RegisterBlock {
    gpclk_oe_ctrl: GPCLK_OE_CTRL,
    _reserved0: [u8; 16],
    clk_sys_ctrl: CLK_SYS_CTRL,
    clk_sys_div_int: CLK_SYS_DIV_INT,
    _reserved1: [u8; 4],
    clk_sys_sel: CLK_SYS_SEL,
    clk_slow_sys_ctrl: CLK_SLOW_SYS_CTRL,
    clk_slow_sys_div_int: CLK_SLOW_SYS_DIV_INT,
    _reserved2: [u8; 4],
    clk_slow_sys_sel: CLK_SLOW_SYS_SEL,
    _reserved3: [u8; 16],
    clk_dma_ctrl: CLK_DMA_CTRL,
    clk_dma_div_int: CLK_DMA_DIV_INT,
    _reserved4: [u8; 4],
    clk_dma_sel: CLK_DMA_SEL,
    clk_uart_ctrl: CLK_UART_CTRL,
    clk_uart_div_int: CLK_UART_DIV_INT,
    _reserved5: [u8; 4],
    clk_uart_sel: CLK_UART_SEL,
    clk_eth_ctrl: CLK_ETH_CTRL,
    clk_eth_div_int: CLK_ETH_DIV_INT,
    _reserved6: [u8; 4],
    clk_eth_sel: CLK_ETH_SEL,
    clk_pwm0_ctrl: CLK_PWM0_CTRL,
    clk_pwm0_div_int: CLK_PWM0_DIV_INT,
    clk_pwm0_div_frac: CLK_PWM0_DIV_FRAC,
    clk_pwm0_sel: CLK_PWM0_SEL,
    clk_pwm1_ctrl: CLK_PWM1_CTRL,
    clk_pwm1_div_int: CLK_PWM1_DIV_INT,
    clk_pwm1_div_frac: CLK_PWM1_DIV_FRAC,
    clk_pwm1_sel: CLK_PWM1_SEL,
    clk_audio_in_ctrl: CLK_AUDIO_IN_CTRL,
    clk_audio_in_div_int: CLK_AUDIO_IN_DIV_INT,
    _reserved7: [u8; 4],
    clk_audio_in_sel: CLK_AUDIO_IN_SEL,
    clk_audio_out_ctrl: CLK_AUDIO_OUT_CTRL,
    clk_audio_out_div_int: CLK_AUDIO_OUT_DIV_INT,
    _reserved8: [u8; 4],
    clk_audio_out_sel: CLK_AUDIO_OUT_SEL,
    clk_i2s_ctrl: CLK_I2S_CTRL,
    clk_i2s_div_int: CLK_I2S_DIV_INT,
    _reserved9: [u8; 4],
    clk_i2s_sel: CLK_I2S_SEL,
    clk_mipi0_cfg_ctrl: CLK_MIPI0_CFG_CTRL,
    clk_mipi0_cfg_div_int: CLK_MIPI0_CFG_DIV_INT,
    _reserved10: [u8; 4],
    clk_mipi0_cfg_sel: CLK_MIPI0_CFG_SEL,
    clk_mipi1_cfg_ctrl: CLK_MIPI1_CFG_CTRL,
    clk_mipi1_cfg_div_int: CLK_MIPI1_CFG_DIV_INT,
    _reserved11: [u8; 4],
    clk_mipi1_cfg_sel: CLK_MIPI1_CFG_SEL,
    clk_pcie_aux_ctrl: CLK_PCIE_AUX_CTRL,
    clk_pcie_aux_div_int: CLK_PCIE_AUX_DIV_INT,
    _reserved12: [u8; 4],
    clk_pcie_aux_sel: CLK_PCIE_AUX_SEL,
    clk_usbh0_microframe_ctrl: CLK_USBH0_MICROFRAME_CTRL,
    clk_usbh0_microframe_div_int: CLK_USBH0_MICROFRAME_DIV_INT,
    _reserved13: [u8; 4],
    clk_usbh0_microframe_sel: CLK_USBH0_MICROFRAME_SEL,
    clk_usbh1_microframe_ctrl: CLK_USBH1_MICROFRAME_CTRL,
    clk_usbh1_microframe_div_int: CLK_USBH1_MICROFRAME_DIV_INT,
    _reserved14: [u8; 4],
    clk_usbh1_microframe_sel: CLK_USBH1_MICROFRAME_SEL,
    clk_usbh0_suspend_ctrl: CLK_USBH0_SUSPEND_CTRL,
    clk_usbh0_suspend_div_int: CLK_USBH0_SUSPEND_DIV_INT,
    _reserved15: [u8; 4],
    clk_usbh0_suspend_sel: CLK_USBH0_SUSPEND_SEL,
    clk_usbh1_suspend_ctrl: CLK_USBH1_SUSPEND_CTRL,
    clk_usbh1_suspend_div_int: CLK_USBH1_SUSPEND_DIV_INT,
    _reserved16: [u8; 4],
    clk_usbh1_suspend_sel: CLK_USBH1_SUSPEND_SEL,
    clk_eth_tsu_ctrl: CLK_ETH_TSU_CTRL,
    clk_eth_tsu_div_int: CLK_ETH_TSU_DIV_INT,
    _reserved17: [u8; 4],
    clk_eth_tsu_sel: CLK_ETH_TSU_SEL,
    clk_adc_ctrl: CLK_ADC_CTRL,
    clk_adc_div_int: CLK_ADC_DIV_INT,
    _reserved18: [u8; 4],
    clk_adc_sel: CLK_ADC_SEL,
    clk_sdio_timer_ctrl: CLK_SDIO_TIMER_CTRL,
    clk_sdio_timer_div_int: CLK_SDIO_TIMER_DIV_INT,
    _reserved19: [u8; 4],
    clk_sdio_timer_sel: CLK_SDIO_TIMER_SEL,
    clk_sdio_alt_src_ctrl: CLK_SDIO_ALT_SRC_CTRL,
    clk_sdio_alt_src_div_int: CLK_SDIO_ALT_SRC_DIV_INT,
    _reserved20: [u8; 4],
    clk_sdio_alt_src_sel: CLK_SDIO_ALT_SRC_SEL,
    clk_gp0_ctrl: CLK_GP0_CTRL,
    clk_gp0_div_int: CLK_GP0_DIV_INT,
    clk_gp0_div_frac: CLK_GP0_DIV_FRAC,
    clk_gp0_sel: CLK_GP0_SEL,
    clk_gp1_ctrl: CLK_GP1_CTRL,
    clk_gp1_div_int: CLK_GP1_DIV_INT,
    clk_gp1_div_frac: CLK_GP1_DIV_FRAC,
    clk_gp1_sel: CLK_GP1_SEL,
    clk_gp2_ctrl: CLK_GP2_CTRL,
    clk_gp2_div_int: CLK_GP2_DIV_INT,
    clk_gp2_div_frac: CLK_GP2_DIV_FRAC,
    clk_gp2_sel: CLK_GP2_SEL,
    clk_gp3_ctrl: CLK_GP3_CTRL,
    clk_gp3_div_int: CLK_GP3_DIV_INT,
    clk_gp3_div_frac: CLK_GP3_DIV_FRAC,
    clk_gp3_sel: CLK_GP3_SEL,
    clk_gp4_ctrl: CLK_GP4_CTRL,
    clk_gp4_div_int: CLK_GP4_DIV_INT,
    clk_gp4_div_frac: CLK_GP4_DIV_FRAC,
    clk_gp4_sel: CLK_GP4_SEL,
    clk_gp5_ctrl: CLK_GP5_CTRL,
    clk_gp5_div_int: CLK_GP5_DIV_INT,
    clk_gp5_div_frac: CLK_GP5_DIV_FRAC,
    clk_gp5_sel: CLK_GP5_SEL,
    _reserved21: [u8; 56],
    clk_sys_resus_ctrl: CLK_SYS_RESUS_CTRL,
    _reserved22: [u8; 4],
    clk_slow_sys_resus_ctrl: CLK_SLOW_SYS_RESUS_CTRL,
    _reserved23: [u8; 4],
    fc0_ref_khz: FC0_REF_KHZ,
    fc0_min_khz: FC0_MIN_KHZ,
    fc0_max_khz: FC0_MAX_KHZ,
    fc0_delay: FC0_DELAY,
    fc0_interval: FC0_INTERVAL,
    fc0_src: FC0_SRC,
    fc0_status: FC0_STATUS,
    fc0_result: FC0_RESULT,
}

impl RegisterBlock {
    /// General-purpose clock output-enable control; individual bit names remain raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn gpclk_oe_ctrl(&self) -> &GPCLK_OE_CTRL {
        &self.gpclk_oe_ctrl
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_sys_ctrl(&self) -> &CLK_SYS_CTRL {
        &self.clk_sys_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_sys_div_int(&self) -> &CLK_SYS_DIV_INT {
        &self.clk_sys_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_sys_sel(&self) -> &CLK_SYS_SEL {
        &self.clk_sys_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_slow_sys_ctrl(&self) -> &CLK_SLOW_SYS_CTRL {
        &self.clk_slow_sys_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_slow_sys_div_int(&self) -> &CLK_SLOW_SYS_DIV_INT {
        &self.clk_slow_sys_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_slow_sys_sel(&self) -> &CLK_SLOW_SYS_SEL {
        &self.clk_slow_sys_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_dma_ctrl(&self) -> &CLK_DMA_CTRL {
        &self.clk_dma_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_dma_div_int(&self) -> &CLK_DMA_DIV_INT {
        &self.clk_dma_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_dma_sel(&self) -> &CLK_DMA_SEL {
        &self.clk_dma_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_uart_ctrl(&self) -> &CLK_UART_CTRL {
        &self.clk_uart_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_uart_div_int(&self) -> &CLK_UART_DIV_INT {
        &self.clk_uart_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_uart_sel(&self) -> &CLK_UART_SEL {
        &self.clk_uart_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_eth_ctrl(&self) -> &CLK_ETH_CTRL {
        &self.clk_eth_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_eth_div_int(&self) -> &CLK_ETH_DIV_INT {
        &self.clk_eth_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_eth_sel(&self) -> &CLK_ETH_SEL {
        &self.clk_eth_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_pwm0_ctrl(&self) -> &CLK_PWM0_CTRL {
        &self.clk_pwm0_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_pwm0_div_int(&self) -> &CLK_PWM0_DIV_INT {
        &self.clk_pwm0_div_int
    }
    /// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_pwm0_div_frac(&self) -> &CLK_PWM0_DIV_FRAC {
        &self.clk_pwm0_div_frac
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_pwm0_sel(&self) -> &CLK_PWM0_SEL {
        &self.clk_pwm0_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_pwm1_ctrl(&self) -> &CLK_PWM1_CTRL {
        &self.clk_pwm1_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_pwm1_div_int(&self) -> &CLK_PWM1_DIV_INT {
        &self.clk_pwm1_div_int
    }
    /// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_pwm1_div_frac(&self) -> &CLK_PWM1_DIV_FRAC {
        &self.clk_pwm1_div_frac
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_pwm1_sel(&self) -> &CLK_PWM1_SEL {
        &self.clk_pwm1_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_audio_in_ctrl(&self) -> &CLK_AUDIO_IN_CTRL {
        &self.clk_audio_in_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_audio_in_div_int(&self) -> &CLK_AUDIO_IN_DIV_INT {
        &self.clk_audio_in_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_audio_in_sel(&self) -> &CLK_AUDIO_IN_SEL {
        &self.clk_audio_in_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_audio_out_ctrl(&self) -> &CLK_AUDIO_OUT_CTRL {
        &self.clk_audio_out_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_audio_out_div_int(&self) -> &CLK_AUDIO_OUT_DIV_INT {
        &self.clk_audio_out_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_audio_out_sel(&self) -> &CLK_AUDIO_OUT_SEL {
        &self.clk_audio_out_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_i2s_ctrl(&self) -> &CLK_I2S_CTRL {
        &self.clk_i2s_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_i2s_div_int(&self) -> &CLK_I2S_DIV_INT {
        &self.clk_i2s_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_i2s_sel(&self) -> &CLK_I2S_SEL {
        &self.clk_i2s_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_mipi0_cfg_ctrl(&self) -> &CLK_MIPI0_CFG_CTRL {
        &self.clk_mipi0_cfg_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_mipi0_cfg_div_int(&self) -> &CLK_MIPI0_CFG_DIV_INT {
        &self.clk_mipi0_cfg_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_mipi0_cfg_sel(&self) -> &CLK_MIPI0_CFG_SEL {
        &self.clk_mipi0_cfg_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_mipi1_cfg_ctrl(&self) -> &CLK_MIPI1_CFG_CTRL {
        &self.clk_mipi1_cfg_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_mipi1_cfg_div_int(&self) -> &CLK_MIPI1_CFG_DIV_INT {
        &self.clk_mipi1_cfg_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_mipi1_cfg_sel(&self) -> &CLK_MIPI1_CFG_SEL {
        &self.clk_mipi1_cfg_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_pcie_aux_ctrl(&self) -> &CLK_PCIE_AUX_CTRL {
        &self.clk_pcie_aux_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_pcie_aux_div_int(&self) -> &CLK_PCIE_AUX_DIV_INT {
        &self.clk_pcie_aux_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_pcie_aux_sel(&self) -> &CLK_PCIE_AUX_SEL {
        &self.clk_pcie_aux_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_usbh0_microframe_ctrl(&self) -> &CLK_USBH0_MICROFRAME_CTRL {
        &self.clk_usbh0_microframe_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_usbh0_microframe_div_int(&self) -> &CLK_USBH0_MICROFRAME_DIV_INT {
        &self.clk_usbh0_microframe_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_usbh0_microframe_sel(&self) -> &CLK_USBH0_MICROFRAME_SEL {
        &self.clk_usbh0_microframe_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_usbh1_microframe_ctrl(&self) -> &CLK_USBH1_MICROFRAME_CTRL {
        &self.clk_usbh1_microframe_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_usbh1_microframe_div_int(&self) -> &CLK_USBH1_MICROFRAME_DIV_INT {
        &self.clk_usbh1_microframe_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_usbh1_microframe_sel(&self) -> &CLK_USBH1_MICROFRAME_SEL {
        &self.clk_usbh1_microframe_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_usbh0_suspend_ctrl(&self) -> &CLK_USBH0_SUSPEND_CTRL {
        &self.clk_usbh0_suspend_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_usbh0_suspend_div_int(&self) -> &CLK_USBH0_SUSPEND_DIV_INT {
        &self.clk_usbh0_suspend_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_usbh0_suspend_sel(&self) -> &CLK_USBH0_SUSPEND_SEL {
        &self.clk_usbh0_suspend_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_usbh1_suspend_ctrl(&self) -> &CLK_USBH1_SUSPEND_CTRL {
        &self.clk_usbh1_suspend_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_usbh1_suspend_div_int(&self) -> &CLK_USBH1_SUSPEND_DIV_INT {
        &self.clk_usbh1_suspend_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_usbh1_suspend_sel(&self) -> &CLK_USBH1_SUSPEND_SEL {
        &self.clk_usbh1_suspend_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_eth_tsu_ctrl(&self) -> &CLK_ETH_TSU_CTRL {
        &self.clk_eth_tsu_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_eth_tsu_div_int(&self) -> &CLK_ETH_TSU_DIV_INT {
        &self.clk_eth_tsu_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_eth_tsu_sel(&self) -> &CLK_ETH_TSU_SEL {
        &self.clk_eth_tsu_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_adc_ctrl(&self) -> &CLK_ADC_CTRL {
        &self.clk_adc_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_adc_div_int(&self) -> &CLK_ADC_DIV_INT {
        &self.clk_adc_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_adc_sel(&self) -> &CLK_ADC_SEL {
        &self.clk_adc_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_sdio_timer_ctrl(&self) -> &CLK_SDIO_TIMER_CTRL {
        &self.clk_sdio_timer_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_sdio_timer_div_int(&self) -> &CLK_SDIO_TIMER_DIV_INT {
        &self.clk_sdio_timer_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_sdio_timer_sel(&self) -> &CLK_SDIO_TIMER_SEL {
        &self.clk_sdio_timer_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_sdio_alt_src_ctrl(&self) -> &CLK_SDIO_ALT_SRC_CTRL {
        &self.clk_sdio_alt_src_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_sdio_alt_src_div_int(&self) -> &CLK_SDIO_ALT_SRC_DIV_INT {
        &self.clk_sdio_alt_src_div_int
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_sdio_alt_src_sel(&self) -> &CLK_SDIO_ALT_SRC_SEL {
        &self.clk_sdio_alt_src_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp0_ctrl(&self) -> &CLK_GP0_CTRL {
        &self.clk_gp0_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp0_div_int(&self) -> &CLK_GP0_DIV_INT {
        &self.clk_gp0_div_int
    }
    /// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp0_div_frac(&self) -> &CLK_GP0_DIV_FRAC {
        &self.clk_gp0_div_frac
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_gp0_sel(&self) -> &CLK_GP0_SEL {
        &self.clk_gp0_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp1_ctrl(&self) -> &CLK_GP1_CTRL {
        &self.clk_gp1_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp1_div_int(&self) -> &CLK_GP1_DIV_INT {
        &self.clk_gp1_div_int
    }
    /// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp1_div_frac(&self) -> &CLK_GP1_DIV_FRAC {
        &self.clk_gp1_div_frac
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_gp1_sel(&self) -> &CLK_GP1_SEL {
        &self.clk_gp1_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp2_ctrl(&self) -> &CLK_GP2_CTRL {
        &self.clk_gp2_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp2_div_int(&self) -> &CLK_GP2_DIV_INT {
        &self.clk_gp2_div_int
    }
    /// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp2_div_frac(&self) -> &CLK_GP2_DIV_FRAC {
        &self.clk_gp2_div_frac
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_gp2_sel(&self) -> &CLK_GP2_SEL {
        &self.clk_gp2_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp3_ctrl(&self) -> &CLK_GP3_CTRL {
        &self.clk_gp3_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp3_div_int(&self) -> &CLK_GP3_DIV_INT {
        &self.clk_gp3_div_int
    }
    /// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp3_div_frac(&self) -> &CLK_GP3_DIV_FRAC {
        &self.clk_gp3_div_frac
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_gp3_sel(&self) -> &CLK_GP3_SEL {
        &self.clk_gp3_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp4_ctrl(&self) -> &CLK_GP4_CTRL {
        &self.clk_gp4_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp4_div_int(&self) -> &CLK_GP4_DIV_INT {
        &self.clk_gp4_div_int
    }
    /// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp4_div_frac(&self) -> &CLK_GP4_DIV_FRAC {
        &self.clk_gp4_div_frac
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_gp4_sel(&self) -> &CLK_GP4_SEL {
        &self.clk_gp4_sel
    }
    /// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp5_ctrl(&self) -> &CLK_GP5_CTRL {
        &self.clk_gp5_ctrl
    }
    /// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp5_div_int(&self) -> &CLK_GP5_DIV_INT {
        &self.clk_gp5_div_int
    }
    /// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_gp5_div_frac(&self) -> &CLK_GP5_DIV_FRAC {
        &self.clk_gp5_div_frac
    }
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn clk_gp5_sel(&self) -> &CLK_GP5_SEL {
        &self.clk_gp5_sel
    }
    /// System-clock resuscitation control; fields remain raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_sys_resus_ctrl(&self) -> &CLK_SYS_RESUS_CTRL {
        &self.clk_sys_resus_ctrl
    }
    /// Slow-system-clock resuscitation control; fields remain raw. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn clk_slow_sys_resus_ctrl(&self) -> &CLK_SLOW_SYS_RESUS_CTRL {
        &self.clk_slow_sys_resus_ctrl
    }
    /// Frequency-counter reference frequency in kHz. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn fc0_ref_khz(&self) -> &FC0_REF_KHZ {
        &self.fc0_ref_khz
    }
    /// Frequency-counter minimum expected kHz. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn fc0_min_khz(&self) -> &FC0_MIN_KHZ {
        &self.fc0_min_khz
    }
    /// Frequency-counter maximum expected kHz. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn fc0_max_khz(&self) -> &FC0_MAX_KHZ {
        &self.fc0_max_khz
    }
    /// Frequency-counter start delay. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn fc0_delay(&self) -> &FC0_DELAY {
        &self.fc0_delay
    }
    /// Frequency-counter measurement interval. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn fc0_interval(&self) -> &FC0_INTERVAL {
        &self.fc0_interval
    }
    /// Frequency-counter source selector. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write Plain.
    /// Write API is not exposed because the verified operation semantics are insufficient.
    #[inline(always)]
    pub const fn fc0_src(&self) -> &FC0_SRC {
        &self.fc0_src
    }
    /// Frequency-counter status. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn fc0_status(&self) -> &FC0_STATUS {
        &self.fc0_status
    }
    /// Frequency-counter result; fractional portion begins at bit 5. \[Source: Linux RP1 common-clock driver.\]
    /// Hardware access: read Plain; write None.
    #[inline(always)]
    pub const fn fc0_result(&self) -> &FC0_RESULT {
        &self.fc0_result
    }
}

/// General-purpose clock output-enable control; individual bit names remain raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type GPCLK_OE_CTRL = Reg<gpclk_oe_ctrl::GPCLK_OE_CTRL_SPEC>;
/// General-purpose clock output-enable control; individual bit names remain raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod gpclk_oe_ctrl {
    pub struct GPCLK_OE_CTRL_SPEC;
    impl crate::RegisterSpec for GPCLK_OE_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for GPCLK_OE_CTRL_SPEC {}
    pub type R = crate::R<GPCLK_OE_CTRL_SPEC>;
    impl crate::UnsafeWritable for GPCLK_OE_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for GPCLK_OE_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for GPCLK_OE_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for GPCLK_OE_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// General-purpose clock output-enable control; individual bit names remain raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_SYS_CTRL = Reg<clk_sys_ctrl::CLK_SYS_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_sys_ctrl {
    pub struct CLK_SYS_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_SYS_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_SYS_CTRL_SPEC {}
    pub type R = crate::R<CLK_SYS_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_SYS_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_SYS_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_SYS_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_SYS_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_SYS_DIV_INT = Reg<clk_sys_div_int::CLK_SYS_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_sys_div_int {
    pub struct CLK_SYS_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_SYS_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_SYS_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_SYS_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_SYS_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_SYS_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_SYS_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_SYS_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 24-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_SYS_SEL = Reg<clk_sys_sel::CLK_SYS_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_sys_sel {
    pub struct CLK_SYS_SEL_SPEC;
    impl crate::RegisterSpec for CLK_SYS_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_SYS_SEL_SPEC {}
    pub type R = crate::R<CLK_SYS_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_SLOW_SYS_CTRL = Reg<clk_slow_sys_ctrl::CLK_SLOW_SYS_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_slow_sys_ctrl {
    pub struct CLK_SLOW_SYS_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_SLOW_SYS_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_SLOW_SYS_CTRL_SPEC {}
    pub type R = crate::R<CLK_SLOW_SYS_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_SLOW_SYS_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_SLOW_SYS_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_SLOW_SYS_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_SLOW_SYS_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_SLOW_SYS_DIV_INT = Reg<clk_slow_sys_div_int::CLK_SLOW_SYS_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_slow_sys_div_int {
    pub struct CLK_SLOW_SYS_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_SLOW_SYS_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_SLOW_SYS_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_SLOW_SYS_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_SLOW_SYS_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_SLOW_SYS_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_SLOW_SYS_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_SLOW_SYS_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_SLOW_SYS_SEL = Reg<clk_slow_sys_sel::CLK_SLOW_SYS_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_slow_sys_sel {
    pub struct CLK_SLOW_SYS_SEL_SPEC;
    impl crate::RegisterSpec for CLK_SLOW_SYS_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_SLOW_SYS_SEL_SPEC {}
    pub type R = crate::R<CLK_SLOW_SYS_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_DMA_CTRL = Reg<clk_dma_ctrl::CLK_DMA_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_dma_ctrl {
    pub struct CLK_DMA_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_DMA_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_DMA_CTRL_SPEC {}
    pub type R = crate::R<CLK_DMA_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_DMA_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_DMA_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_DMA_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_DMA_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_DMA_DIV_INT = Reg<clk_dma_div_int::CLK_DMA_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_dma_div_int {
    pub struct CLK_DMA_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_DMA_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_DMA_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_DMA_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_DMA_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_DMA_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_DMA_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_DMA_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_DMA_SEL = Reg<clk_dma_sel::CLK_DMA_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_dma_sel {
    pub struct CLK_DMA_SEL_SPEC;
    impl crate::RegisterSpec for CLK_DMA_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_DMA_SEL_SPEC {}
    pub type R = crate::R<CLK_DMA_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_UART_CTRL = Reg<clk_uart_ctrl::CLK_UART_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_uart_ctrl {
    pub struct CLK_UART_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_UART_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_UART_CTRL_SPEC {}
    pub type R = crate::R<CLK_UART_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_UART_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_UART_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_UART_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_UART_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_UART_DIV_INT = Reg<clk_uart_div_int::CLK_UART_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_uart_div_int {
    pub struct CLK_UART_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_UART_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_UART_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_UART_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_UART_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_UART_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_UART_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_UART_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_UART_SEL = Reg<clk_uart_sel::CLK_UART_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_uart_sel {
    pub struct CLK_UART_SEL_SPEC;
    impl crate::RegisterSpec for CLK_UART_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_UART_SEL_SPEC {}
    pub type R = crate::R<CLK_UART_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_ETH_CTRL = Reg<clk_eth_ctrl::CLK_ETH_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_eth_ctrl {
    pub struct CLK_ETH_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_ETH_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_ETH_CTRL_SPEC {}
    pub type R = crate::R<CLK_ETH_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_ETH_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_ETH_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_ETH_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_ETH_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_ETH_DIV_INT = Reg<clk_eth_div_int::CLK_ETH_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_eth_div_int {
    pub struct CLK_ETH_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_ETH_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_ETH_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_ETH_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_ETH_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_ETH_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_ETH_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_ETH_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_ETH_SEL = Reg<clk_eth_sel::CLK_ETH_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_eth_sel {
    pub struct CLK_ETH_SEL_SPEC;
    impl crate::RegisterSpec for CLK_ETH_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_ETH_SEL_SPEC {}
    pub type R = crate::R<CLK_ETH_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_PWM0_CTRL = Reg<clk_pwm0_ctrl::CLK_PWM0_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_pwm0_ctrl {
    pub struct CLK_PWM0_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_PWM0_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_PWM0_CTRL_SPEC {}
    pub type R = crate::R<CLK_PWM0_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_PWM0_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_PWM0_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_PWM0_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_PWM0_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_PWM0_DIV_INT = Reg<clk_pwm0_div_int::CLK_PWM0_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_pwm0_div_int {
    pub struct CLK_PWM0_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_PWM0_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_PWM0_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_PWM0_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_PWM0_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_PWM0_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_PWM0_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_PWM0_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 16-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_PWM0_DIV_FRAC = Reg<clk_pwm0_div_frac::CLK_PWM0_DIV_FRAC_SPEC>;
/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_pwm0_div_frac {
    pub struct CLK_PWM0_DIV_FRAC_SPEC;
    impl crate::RegisterSpec for CLK_PWM0_DIV_FRAC_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_PWM0_DIV_FRAC_SPEC {}
    pub type R = crate::R<CLK_PWM0_DIV_FRAC_SPEC>;
    impl crate::UnsafeWritable for CLK_PWM0_DIV_FRAC_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_PWM0_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_PWM0_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_PWM0_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Fractional divider value.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_PWM0_SEL = Reg<clk_pwm0_sel::CLK_PWM0_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_pwm0_sel {
    pub struct CLK_PWM0_SEL_SPEC;
    impl crate::RegisterSpec for CLK_PWM0_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_PWM0_SEL_SPEC {}
    pub type R = crate::R<CLK_PWM0_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_PWM1_CTRL = Reg<clk_pwm1_ctrl::CLK_PWM1_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_pwm1_ctrl {
    pub struct CLK_PWM1_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_PWM1_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_PWM1_CTRL_SPEC {}
    pub type R = crate::R<CLK_PWM1_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_PWM1_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_PWM1_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_PWM1_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_PWM1_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_PWM1_DIV_INT = Reg<clk_pwm1_div_int::CLK_PWM1_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_pwm1_div_int {
    pub struct CLK_PWM1_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_PWM1_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_PWM1_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_PWM1_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_PWM1_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_PWM1_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_PWM1_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_PWM1_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 16-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_PWM1_DIV_FRAC = Reg<clk_pwm1_div_frac::CLK_PWM1_DIV_FRAC_SPEC>;
/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_pwm1_div_frac {
    pub struct CLK_PWM1_DIV_FRAC_SPEC;
    impl crate::RegisterSpec for CLK_PWM1_DIV_FRAC_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_PWM1_DIV_FRAC_SPEC {}
    pub type R = crate::R<CLK_PWM1_DIV_FRAC_SPEC>;
    impl crate::UnsafeWritable for CLK_PWM1_DIV_FRAC_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_PWM1_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_PWM1_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_PWM1_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Fractional divider value.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_PWM1_SEL = Reg<clk_pwm1_sel::CLK_PWM1_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_pwm1_sel {
    pub struct CLK_PWM1_SEL_SPEC;
    impl crate::RegisterSpec for CLK_PWM1_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_PWM1_SEL_SPEC {}
    pub type R = crate::R<CLK_PWM1_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_AUDIO_IN_CTRL = Reg<clk_audio_in_ctrl::CLK_AUDIO_IN_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_audio_in_ctrl {
    pub struct CLK_AUDIO_IN_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_AUDIO_IN_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_AUDIO_IN_CTRL_SPEC {}
    pub type R = crate::R<CLK_AUDIO_IN_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_AUDIO_IN_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_AUDIO_IN_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_AUDIO_IN_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_AUDIO_IN_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_AUDIO_IN_DIV_INT = Reg<clk_audio_in_div_int::CLK_AUDIO_IN_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_audio_in_div_int {
    pub struct CLK_AUDIO_IN_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_AUDIO_IN_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_AUDIO_IN_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_AUDIO_IN_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_AUDIO_IN_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_AUDIO_IN_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_AUDIO_IN_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_AUDIO_IN_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_AUDIO_IN_SEL = Reg<clk_audio_in_sel::CLK_AUDIO_IN_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_audio_in_sel {
    pub struct CLK_AUDIO_IN_SEL_SPEC;
    impl crate::RegisterSpec for CLK_AUDIO_IN_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_AUDIO_IN_SEL_SPEC {}
    pub type R = crate::R<CLK_AUDIO_IN_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_AUDIO_OUT_CTRL = Reg<clk_audio_out_ctrl::CLK_AUDIO_OUT_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_audio_out_ctrl {
    pub struct CLK_AUDIO_OUT_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_AUDIO_OUT_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_AUDIO_OUT_CTRL_SPEC {}
    pub type R = crate::R<CLK_AUDIO_OUT_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_AUDIO_OUT_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_AUDIO_OUT_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_AUDIO_OUT_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_AUDIO_OUT_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_AUDIO_OUT_DIV_INT = Reg<clk_audio_out_div_int::CLK_AUDIO_OUT_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_audio_out_div_int {
    pub struct CLK_AUDIO_OUT_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_AUDIO_OUT_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_AUDIO_OUT_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_AUDIO_OUT_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_AUDIO_OUT_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_AUDIO_OUT_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_AUDIO_OUT_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_AUDIO_OUT_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_AUDIO_OUT_SEL = Reg<clk_audio_out_sel::CLK_AUDIO_OUT_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_audio_out_sel {
    pub struct CLK_AUDIO_OUT_SEL_SPEC;
    impl crate::RegisterSpec for CLK_AUDIO_OUT_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_AUDIO_OUT_SEL_SPEC {}
    pub type R = crate::R<CLK_AUDIO_OUT_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_I2S_CTRL = Reg<clk_i2s_ctrl::CLK_I2S_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_i2s_ctrl {
    pub struct CLK_I2S_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_I2S_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_I2S_CTRL_SPEC {}
    pub type R = crate::R<CLK_I2S_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_I2S_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_I2S_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_I2S_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_I2S_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_I2S_DIV_INT = Reg<clk_i2s_div_int::CLK_I2S_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_i2s_div_int {
    pub struct CLK_I2S_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_I2S_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_I2S_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_I2S_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_I2S_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_I2S_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_I2S_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_I2S_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_I2S_SEL = Reg<clk_i2s_sel::CLK_I2S_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_i2s_sel {
    pub struct CLK_I2S_SEL_SPEC;
    impl crate::RegisterSpec for CLK_I2S_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_I2S_SEL_SPEC {}
    pub type R = crate::R<CLK_I2S_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_MIPI0_CFG_CTRL = Reg<clk_mipi0_cfg_ctrl::CLK_MIPI0_CFG_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_mipi0_cfg_ctrl {
    pub struct CLK_MIPI0_CFG_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_MIPI0_CFG_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_MIPI0_CFG_CTRL_SPEC {}
    pub type R = crate::R<CLK_MIPI0_CFG_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_MIPI0_CFG_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_MIPI0_CFG_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_MIPI0_CFG_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_MIPI0_CFG_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_MIPI0_CFG_DIV_INT = Reg<clk_mipi0_cfg_div_int::CLK_MIPI0_CFG_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_mipi0_cfg_div_int {
    pub struct CLK_MIPI0_CFG_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_MIPI0_CFG_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_MIPI0_CFG_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_MIPI0_CFG_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_MIPI0_CFG_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_MIPI0_CFG_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_MIPI0_CFG_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_MIPI0_CFG_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_MIPI0_CFG_SEL = Reg<clk_mipi0_cfg_sel::CLK_MIPI0_CFG_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_mipi0_cfg_sel {
    pub struct CLK_MIPI0_CFG_SEL_SPEC;
    impl crate::RegisterSpec for CLK_MIPI0_CFG_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_MIPI0_CFG_SEL_SPEC {}
    pub type R = crate::R<CLK_MIPI0_CFG_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_MIPI1_CFG_CTRL = Reg<clk_mipi1_cfg_ctrl::CLK_MIPI1_CFG_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_mipi1_cfg_ctrl {
    pub struct CLK_MIPI1_CFG_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_MIPI1_CFG_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_MIPI1_CFG_CTRL_SPEC {}
    pub type R = crate::R<CLK_MIPI1_CFG_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_MIPI1_CFG_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_MIPI1_CFG_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_MIPI1_CFG_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_MIPI1_CFG_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_MIPI1_CFG_DIV_INT = Reg<clk_mipi1_cfg_div_int::CLK_MIPI1_CFG_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_mipi1_cfg_div_int {
    pub struct CLK_MIPI1_CFG_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_MIPI1_CFG_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_MIPI1_CFG_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_MIPI1_CFG_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_MIPI1_CFG_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_MIPI1_CFG_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_MIPI1_CFG_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_MIPI1_CFG_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_MIPI1_CFG_SEL = Reg<clk_mipi1_cfg_sel::CLK_MIPI1_CFG_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_mipi1_cfg_sel {
    pub struct CLK_MIPI1_CFG_SEL_SPEC;
    impl crate::RegisterSpec for CLK_MIPI1_CFG_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_MIPI1_CFG_SEL_SPEC {}
    pub type R = crate::R<CLK_MIPI1_CFG_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_PCIE_AUX_CTRL = Reg<clk_pcie_aux_ctrl::CLK_PCIE_AUX_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_pcie_aux_ctrl {
    pub struct CLK_PCIE_AUX_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_PCIE_AUX_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_PCIE_AUX_CTRL_SPEC {}
    pub type R = crate::R<CLK_PCIE_AUX_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_PCIE_AUX_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_PCIE_AUX_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_PCIE_AUX_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_PCIE_AUX_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_PCIE_AUX_DIV_INT = Reg<clk_pcie_aux_div_int::CLK_PCIE_AUX_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_pcie_aux_div_int {
    pub struct CLK_PCIE_AUX_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_PCIE_AUX_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_PCIE_AUX_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_PCIE_AUX_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_PCIE_AUX_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_PCIE_AUX_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_PCIE_AUX_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_PCIE_AUX_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_PCIE_AUX_SEL = Reg<clk_pcie_aux_sel::CLK_PCIE_AUX_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_pcie_aux_sel {
    pub struct CLK_PCIE_AUX_SEL_SPEC;
    impl crate::RegisterSpec for CLK_PCIE_AUX_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_PCIE_AUX_SEL_SPEC {}
    pub type R = crate::R<CLK_PCIE_AUX_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_USBH0_MICROFRAME_CTRL = Reg<clk_usbh0_microframe_ctrl::CLK_USBH0_MICROFRAME_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_usbh0_microframe_ctrl {
    pub struct CLK_USBH0_MICROFRAME_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_USBH0_MICROFRAME_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_USBH0_MICROFRAME_CTRL_SPEC {}
    pub type R = crate::R<CLK_USBH0_MICROFRAME_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_USBH0_MICROFRAME_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_USBH0_MICROFRAME_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_USBH0_MICROFRAME_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_USBH0_MICROFRAME_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_USBH0_MICROFRAME_DIV_INT =
    Reg<clk_usbh0_microframe_div_int::CLK_USBH0_MICROFRAME_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_usbh0_microframe_div_int {
    pub struct CLK_USBH0_MICROFRAME_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_USBH0_MICROFRAME_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_USBH0_MICROFRAME_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_USBH0_MICROFRAME_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_USBH0_MICROFRAME_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_USBH0_MICROFRAME_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_USBH0_MICROFRAME_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_USBH0_MICROFRAME_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_USBH0_MICROFRAME_SEL = Reg<clk_usbh0_microframe_sel::CLK_USBH0_MICROFRAME_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_usbh0_microframe_sel {
    pub struct CLK_USBH0_MICROFRAME_SEL_SPEC;
    impl crate::RegisterSpec for CLK_USBH0_MICROFRAME_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_USBH0_MICROFRAME_SEL_SPEC {}
    pub type R = crate::R<CLK_USBH0_MICROFRAME_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_USBH1_MICROFRAME_CTRL = Reg<clk_usbh1_microframe_ctrl::CLK_USBH1_MICROFRAME_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_usbh1_microframe_ctrl {
    pub struct CLK_USBH1_MICROFRAME_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_USBH1_MICROFRAME_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_USBH1_MICROFRAME_CTRL_SPEC {}
    pub type R = crate::R<CLK_USBH1_MICROFRAME_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_USBH1_MICROFRAME_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_USBH1_MICROFRAME_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_USBH1_MICROFRAME_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_USBH1_MICROFRAME_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_USBH1_MICROFRAME_DIV_INT =
    Reg<clk_usbh1_microframe_div_int::CLK_USBH1_MICROFRAME_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_usbh1_microframe_div_int {
    pub struct CLK_USBH1_MICROFRAME_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_USBH1_MICROFRAME_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_USBH1_MICROFRAME_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_USBH1_MICROFRAME_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_USBH1_MICROFRAME_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_USBH1_MICROFRAME_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_USBH1_MICROFRAME_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_USBH1_MICROFRAME_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_USBH1_MICROFRAME_SEL = Reg<clk_usbh1_microframe_sel::CLK_USBH1_MICROFRAME_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_usbh1_microframe_sel {
    pub struct CLK_USBH1_MICROFRAME_SEL_SPEC;
    impl crate::RegisterSpec for CLK_USBH1_MICROFRAME_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_USBH1_MICROFRAME_SEL_SPEC {}
    pub type R = crate::R<CLK_USBH1_MICROFRAME_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_USBH0_SUSPEND_CTRL = Reg<clk_usbh0_suspend_ctrl::CLK_USBH0_SUSPEND_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_usbh0_suspend_ctrl {
    pub struct CLK_USBH0_SUSPEND_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_USBH0_SUSPEND_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_USBH0_SUSPEND_CTRL_SPEC {}
    pub type R = crate::R<CLK_USBH0_SUSPEND_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_USBH0_SUSPEND_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_USBH0_SUSPEND_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_USBH0_SUSPEND_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_USBH0_SUSPEND_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_USBH0_SUSPEND_DIV_INT = Reg<clk_usbh0_suspend_div_int::CLK_USBH0_SUSPEND_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_usbh0_suspend_div_int {
    pub struct CLK_USBH0_SUSPEND_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_USBH0_SUSPEND_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_USBH0_SUSPEND_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_USBH0_SUSPEND_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_USBH0_SUSPEND_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_USBH0_SUSPEND_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_USBH0_SUSPEND_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_USBH0_SUSPEND_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_USBH0_SUSPEND_SEL = Reg<clk_usbh0_suspend_sel::CLK_USBH0_SUSPEND_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_usbh0_suspend_sel {
    pub struct CLK_USBH0_SUSPEND_SEL_SPEC;
    impl crate::RegisterSpec for CLK_USBH0_SUSPEND_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_USBH0_SUSPEND_SEL_SPEC {}
    pub type R = crate::R<CLK_USBH0_SUSPEND_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_USBH1_SUSPEND_CTRL = Reg<clk_usbh1_suspend_ctrl::CLK_USBH1_SUSPEND_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_usbh1_suspend_ctrl {
    pub struct CLK_USBH1_SUSPEND_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_USBH1_SUSPEND_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_USBH1_SUSPEND_CTRL_SPEC {}
    pub type R = crate::R<CLK_USBH1_SUSPEND_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_USBH1_SUSPEND_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_USBH1_SUSPEND_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_USBH1_SUSPEND_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_USBH1_SUSPEND_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_USBH1_SUSPEND_DIV_INT = Reg<clk_usbh1_suspend_div_int::CLK_USBH1_SUSPEND_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_usbh1_suspend_div_int {
    pub struct CLK_USBH1_SUSPEND_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_USBH1_SUSPEND_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_USBH1_SUSPEND_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_USBH1_SUSPEND_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_USBH1_SUSPEND_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_USBH1_SUSPEND_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_USBH1_SUSPEND_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_USBH1_SUSPEND_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_USBH1_SUSPEND_SEL = Reg<clk_usbh1_suspend_sel::CLK_USBH1_SUSPEND_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_usbh1_suspend_sel {
    pub struct CLK_USBH1_SUSPEND_SEL_SPEC;
    impl crate::RegisterSpec for CLK_USBH1_SUSPEND_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_USBH1_SUSPEND_SEL_SPEC {}
    pub type R = crate::R<CLK_USBH1_SUSPEND_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_ETH_TSU_CTRL = Reg<clk_eth_tsu_ctrl::CLK_ETH_TSU_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_eth_tsu_ctrl {
    pub struct CLK_ETH_TSU_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_ETH_TSU_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_ETH_TSU_CTRL_SPEC {}
    pub type R = crate::R<CLK_ETH_TSU_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_ETH_TSU_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_ETH_TSU_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_ETH_TSU_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_ETH_TSU_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_ETH_TSU_DIV_INT = Reg<clk_eth_tsu_div_int::CLK_ETH_TSU_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_eth_tsu_div_int {
    pub struct CLK_ETH_TSU_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_ETH_TSU_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_ETH_TSU_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_ETH_TSU_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_ETH_TSU_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_ETH_TSU_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_ETH_TSU_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_ETH_TSU_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_ETH_TSU_SEL = Reg<clk_eth_tsu_sel::CLK_ETH_TSU_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_eth_tsu_sel {
    pub struct CLK_ETH_TSU_SEL_SPEC;
    impl crate::RegisterSpec for CLK_ETH_TSU_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_ETH_TSU_SEL_SPEC {}
    pub type R = crate::R<CLK_ETH_TSU_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_ADC_CTRL = Reg<clk_adc_ctrl::CLK_ADC_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_adc_ctrl {
    pub struct CLK_ADC_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_ADC_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_ADC_CTRL_SPEC {}
    pub type R = crate::R<CLK_ADC_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_ADC_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_ADC_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_ADC_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_ADC_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_ADC_DIV_INT = Reg<clk_adc_div_int::CLK_ADC_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_adc_div_int {
    pub struct CLK_ADC_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_ADC_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_ADC_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_ADC_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_ADC_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_ADC_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_ADC_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_ADC_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_ADC_SEL = Reg<clk_adc_sel::CLK_ADC_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_adc_sel {
    pub struct CLK_ADC_SEL_SPEC;
    impl crate::RegisterSpec for CLK_ADC_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_ADC_SEL_SPEC {}
    pub type R = crate::R<CLK_ADC_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_SDIO_TIMER_CTRL = Reg<clk_sdio_timer_ctrl::CLK_SDIO_TIMER_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_sdio_timer_ctrl {
    pub struct CLK_SDIO_TIMER_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_SDIO_TIMER_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_SDIO_TIMER_CTRL_SPEC {}
    pub type R = crate::R<CLK_SDIO_TIMER_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_SDIO_TIMER_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_SDIO_TIMER_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_SDIO_TIMER_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_SDIO_TIMER_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_SDIO_TIMER_DIV_INT = Reg<clk_sdio_timer_div_int::CLK_SDIO_TIMER_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_sdio_timer_div_int {
    pub struct CLK_SDIO_TIMER_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_SDIO_TIMER_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_SDIO_TIMER_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_SDIO_TIMER_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_SDIO_TIMER_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_SDIO_TIMER_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_SDIO_TIMER_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_SDIO_TIMER_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_SDIO_TIMER_SEL = Reg<clk_sdio_timer_sel::CLK_SDIO_TIMER_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_sdio_timer_sel {
    pub struct CLK_SDIO_TIMER_SEL_SPEC;
    impl crate::RegisterSpec for CLK_SDIO_TIMER_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_SDIO_TIMER_SEL_SPEC {}
    pub type R = crate::R<CLK_SDIO_TIMER_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_SDIO_ALT_SRC_CTRL = Reg<clk_sdio_alt_src_ctrl::CLK_SDIO_ALT_SRC_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_sdio_alt_src_ctrl {
    pub struct CLK_SDIO_ALT_SRC_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_SDIO_ALT_SRC_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_SDIO_ALT_SRC_CTRL_SPEC {}
    pub type R = crate::R<CLK_SDIO_ALT_SRC_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_SDIO_ALT_SRC_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_SDIO_ALT_SRC_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_SDIO_ALT_SRC_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_SDIO_ALT_SRC_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_SDIO_ALT_SRC_DIV_INT = Reg<clk_sdio_alt_src_div_int::CLK_SDIO_ALT_SRC_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_sdio_alt_src_div_int {
    pub struct CLK_SDIO_ALT_SRC_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_SDIO_ALT_SRC_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_SDIO_ALT_SRC_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_SDIO_ALT_SRC_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_SDIO_ALT_SRC_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_SDIO_ALT_SRC_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_SDIO_ALT_SRC_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_SDIO_ALT_SRC_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 8-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_SDIO_ALT_SRC_SEL = Reg<clk_sdio_alt_src_sel::CLK_SDIO_ALT_SRC_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_sdio_alt_src_sel {
    pub struct CLK_SDIO_ALT_SRC_SEL_SPEC;
    impl crate::RegisterSpec for CLK_SDIO_ALT_SRC_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_SDIO_ALT_SRC_SEL_SPEC {}
    pub type R = crate::R<CLK_SDIO_ALT_SRC_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP0_CTRL = Reg<clk_gp0_ctrl::CLK_GP0_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp0_ctrl {
    pub struct CLK_GP0_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_GP0_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP0_CTRL_SPEC {}
    pub type R = crate::R<CLK_GP0_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_GP0_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP0_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP0_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP0_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP0_DIV_INT = Reg<clk_gp0_div_int::CLK_GP0_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp0_div_int {
    pub struct CLK_GP0_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_GP0_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP0_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_GP0_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_GP0_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP0_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP0_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP0_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 16-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP0_DIV_FRAC = Reg<clk_gp0_div_frac::CLK_GP0_DIV_FRAC_SPEC>;
/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp0_div_frac {
    pub struct CLK_GP0_DIV_FRAC_SPEC;
    impl crate::RegisterSpec for CLK_GP0_DIV_FRAC_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP0_DIV_FRAC_SPEC {}
    pub type R = crate::R<CLK_GP0_DIV_FRAC_SPEC>;
    impl crate::UnsafeWritable for CLK_GP0_DIV_FRAC_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP0_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP0_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP0_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Fractional divider value.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_GP0_SEL = Reg<clk_gp0_sel::CLK_GP0_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_gp0_sel {
    pub struct CLK_GP0_SEL_SPEC;
    impl crate::RegisterSpec for CLK_GP0_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP0_SEL_SPEC {}
    pub type R = crate::R<CLK_GP0_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP1_CTRL = Reg<clk_gp1_ctrl::CLK_GP1_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp1_ctrl {
    pub struct CLK_GP1_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_GP1_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP1_CTRL_SPEC {}
    pub type R = crate::R<CLK_GP1_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_GP1_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP1_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP1_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP1_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP1_DIV_INT = Reg<clk_gp1_div_int::CLK_GP1_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp1_div_int {
    pub struct CLK_GP1_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_GP1_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP1_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_GP1_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_GP1_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP1_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP1_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP1_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 16-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP1_DIV_FRAC = Reg<clk_gp1_div_frac::CLK_GP1_DIV_FRAC_SPEC>;
/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp1_div_frac {
    pub struct CLK_GP1_DIV_FRAC_SPEC;
    impl crate::RegisterSpec for CLK_GP1_DIV_FRAC_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP1_DIV_FRAC_SPEC {}
    pub type R = crate::R<CLK_GP1_DIV_FRAC_SPEC>;
    impl crate::UnsafeWritable for CLK_GP1_DIV_FRAC_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP1_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP1_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP1_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Fractional divider value.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_GP1_SEL = Reg<clk_gp1_sel::CLK_GP1_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_gp1_sel {
    pub struct CLK_GP1_SEL_SPEC;
    impl crate::RegisterSpec for CLK_GP1_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP1_SEL_SPEC {}
    pub type R = crate::R<CLK_GP1_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP2_CTRL = Reg<clk_gp2_ctrl::CLK_GP2_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp2_ctrl {
    pub struct CLK_GP2_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_GP2_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP2_CTRL_SPEC {}
    pub type R = crate::R<CLK_GP2_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_GP2_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP2_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP2_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP2_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP2_DIV_INT = Reg<clk_gp2_div_int::CLK_GP2_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp2_div_int {
    pub struct CLK_GP2_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_GP2_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP2_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_GP2_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_GP2_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP2_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP2_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP2_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 16-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP2_DIV_FRAC = Reg<clk_gp2_div_frac::CLK_GP2_DIV_FRAC_SPEC>;
/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp2_div_frac {
    pub struct CLK_GP2_DIV_FRAC_SPEC;
    impl crate::RegisterSpec for CLK_GP2_DIV_FRAC_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP2_DIV_FRAC_SPEC {}
    pub type R = crate::R<CLK_GP2_DIV_FRAC_SPEC>;
    impl crate::UnsafeWritable for CLK_GP2_DIV_FRAC_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP2_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP2_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP2_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Fractional divider value.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_GP2_SEL = Reg<clk_gp2_sel::CLK_GP2_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_gp2_sel {
    pub struct CLK_GP2_SEL_SPEC;
    impl crate::RegisterSpec for CLK_GP2_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP2_SEL_SPEC {}
    pub type R = crate::R<CLK_GP2_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP3_CTRL = Reg<clk_gp3_ctrl::CLK_GP3_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp3_ctrl {
    pub struct CLK_GP3_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_GP3_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP3_CTRL_SPEC {}
    pub type R = crate::R<CLK_GP3_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_GP3_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP3_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP3_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP3_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP3_DIV_INT = Reg<clk_gp3_div_int::CLK_GP3_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp3_div_int {
    pub struct CLK_GP3_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_GP3_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP3_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_GP3_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_GP3_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP3_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP3_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP3_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 16-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP3_DIV_FRAC = Reg<clk_gp3_div_frac::CLK_GP3_DIV_FRAC_SPEC>;
/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp3_div_frac {
    pub struct CLK_GP3_DIV_FRAC_SPEC;
    impl crate::RegisterSpec for CLK_GP3_DIV_FRAC_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP3_DIV_FRAC_SPEC {}
    pub type R = crate::R<CLK_GP3_DIV_FRAC_SPEC>;
    impl crate::UnsafeWritable for CLK_GP3_DIV_FRAC_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP3_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP3_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP3_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Fractional divider value.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_GP3_SEL = Reg<clk_gp3_sel::CLK_GP3_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_gp3_sel {
    pub struct CLK_GP3_SEL_SPEC;
    impl crate::RegisterSpec for CLK_GP3_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP3_SEL_SPEC {}
    pub type R = crate::R<CLK_GP3_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP4_CTRL = Reg<clk_gp4_ctrl::CLK_GP4_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp4_ctrl {
    pub struct CLK_GP4_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_GP4_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP4_CTRL_SPEC {}
    pub type R = crate::R<CLK_GP4_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_GP4_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP4_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP4_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP4_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP4_DIV_INT = Reg<clk_gp4_div_int::CLK_GP4_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp4_div_int {
    pub struct CLK_GP4_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_GP4_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP4_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_GP4_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_GP4_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP4_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP4_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP4_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 16-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP4_DIV_FRAC = Reg<clk_gp4_div_frac::CLK_GP4_DIV_FRAC_SPEC>;
/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp4_div_frac {
    pub struct CLK_GP4_DIV_FRAC_SPEC;
    impl crate::RegisterSpec for CLK_GP4_DIV_FRAC_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP4_DIV_FRAC_SPEC {}
    pub type R = crate::R<CLK_GP4_DIV_FRAC_SPEC>;
    impl crate::UnsafeWritable for CLK_GP4_DIV_FRAC_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP4_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP4_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP4_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Fractional divider value.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_GP4_SEL = Reg<clk_gp4_sel::CLK_GP4_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_gp4_sel {
    pub struct CLK_GP4_SEL_SPEC;
    impl crate::RegisterSpec for CLK_GP4_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP4_SEL_SPEC {}
    pub type R = crate::R<CLK_GP4_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP5_CTRL = Reg<clk_gp5_ctrl::CLK_GP5_CTRL_SPEC>;
/// Generated-clock control. Primary SRC begins at bit 0 but its clock-specific width remains raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp5_ctrl {
    pub struct CLK_GP5_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_GP5_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP5_CTRL_SPEC {}
    pub type R = crate::R<CLK_GP5_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_GP5_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP5_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP5_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP5_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Auxiliary parent selector.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod auxsrc {
        pub type R = crate::FieldReader<u32>;
    }
    /// Enable generated clock output.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod enable {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn auxsrc(&self) -> auxsrc::R {
            crate::FieldReader::new((self.bits() >> 5) & 0x1f)
        }
        #[inline(always)]
        pub fn enable(&self) -> enable::R {
            crate::BitReader::new(((self.bits() >> 11) & 1) != 0)
        }
    }
}

/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP5_DIV_INT = Reg<clk_gp5_div_int::CLK_GP5_DIV_INT_SPEC>;
/// Integer clock divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp5_div_int {
    pub struct CLK_GP5_DIV_INT_SPEC;
    impl crate::RegisterSpec for CLK_GP5_DIV_INT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP5_DIV_INT_SPEC {}
    pub type R = crate::R<CLK_GP5_DIV_INT_SPEC>;
    impl crate::UnsafeWritable for CLK_GP5_DIV_INT_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP5_DIV_INT_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP5_DIV_INT_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP5_DIV_INT_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// 16-bit integer divider.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_GP5_DIV_FRAC = Reg<clk_gp5_div_frac::CLK_GP5_DIV_FRAC_SPEC>;
/// Sixteen-bit fractional divider. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_gp5_div_frac {
    pub struct CLK_GP5_DIV_FRAC_SPEC;
    impl crate::RegisterSpec for CLK_GP5_DIV_FRAC_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP5_DIV_FRAC_SPEC {}
    pub type R = crate::R<CLK_GP5_DIV_FRAC_SPEC>;
    impl crate::UnsafeWritable for CLK_GP5_DIV_FRAC_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_GP5_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_GP5_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_GP5_DIV_FRAC_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Fractional divider value.
    /// Field write behavior: Plain.
    /// Typed write API is not exposed by register policy.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffff)
        }
    }
}

/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type CLK_GP5_SEL = Reg<clk_gp5_sel::CLK_GP5_SEL_SPEC>;
/// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod clk_gp5_sel {
    pub struct CLK_GP5_SEL_SPEC;
    impl crate::RegisterSpec for CLK_GP5_SEL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_GP5_SEL_SPEC {}
    pub type R = crate::R<CLK_GP5_SEL_SPEC>;
    /// Generated-clock selected-source status; bit encoding is clock-specific and intentionally left raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// System-clock resuscitation control; fields remain raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_SYS_RESUS_CTRL = Reg<clk_sys_resus_ctrl::CLK_SYS_RESUS_CTRL_SPEC>;
/// System-clock resuscitation control; fields remain raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_sys_resus_ctrl {
    pub struct CLK_SYS_RESUS_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_SYS_RESUS_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_SYS_RESUS_CTRL_SPEC {}
    pub type R = crate::R<CLK_SYS_RESUS_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_SYS_RESUS_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_SYS_RESUS_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_SYS_RESUS_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_SYS_RESUS_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// System-clock resuscitation control; fields remain raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Slow-system-clock resuscitation control; fields remain raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type CLK_SLOW_SYS_RESUS_CTRL = Reg<clk_slow_sys_resus_ctrl::CLK_SLOW_SYS_RESUS_CTRL_SPEC>;
/// Slow-system-clock resuscitation control; fields remain raw. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod clk_slow_sys_resus_ctrl {
    pub struct CLK_SLOW_SYS_RESUS_CTRL_SPEC;
    impl crate::RegisterSpec for CLK_SLOW_SYS_RESUS_CTRL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for CLK_SLOW_SYS_RESUS_CTRL_SPEC {}
    pub type R = crate::R<CLK_SLOW_SYS_RESUS_CTRL_SPEC>;
    impl crate::UnsafeWritable for CLK_SLOW_SYS_RESUS_CTRL_SPEC {}
    impl crate::UnsafeAtomicSet for CLK_SLOW_SYS_RESUS_CTRL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for CLK_SLOW_SYS_RESUS_CTRL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for CLK_SLOW_SYS_RESUS_CTRL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Slow-system-clock resuscitation control; fields remain raw. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Frequency-counter reference frequency in kHz. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type FC0_REF_KHZ = Reg<fc0_ref_khz::FC0_REF_KHZ_SPEC>;
/// Frequency-counter reference frequency in kHz. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod fc0_ref_khz {
    pub struct FC0_REF_KHZ_SPEC;
    impl crate::RegisterSpec for FC0_REF_KHZ_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for FC0_REF_KHZ_SPEC {}
    pub type R = crate::R<FC0_REF_KHZ_SPEC>;
    impl crate::UnsafeWritable for FC0_REF_KHZ_SPEC {}
    impl crate::UnsafeAtomicSet for FC0_REF_KHZ_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for FC0_REF_KHZ_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for FC0_REF_KHZ_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Frequency-counter reference frequency in kHz. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Frequency-counter minimum expected kHz. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type FC0_MIN_KHZ = Reg<fc0_min_khz::FC0_MIN_KHZ_SPEC>;
/// Frequency-counter minimum expected kHz. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod fc0_min_khz {
    pub struct FC0_MIN_KHZ_SPEC;
    impl crate::RegisterSpec for FC0_MIN_KHZ_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for FC0_MIN_KHZ_SPEC {}
    pub type R = crate::R<FC0_MIN_KHZ_SPEC>;
    impl crate::UnsafeWritable for FC0_MIN_KHZ_SPEC {}
    impl crate::UnsafeAtomicSet for FC0_MIN_KHZ_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for FC0_MIN_KHZ_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for FC0_MIN_KHZ_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Frequency-counter minimum expected kHz. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Frequency-counter maximum expected kHz. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type FC0_MAX_KHZ = Reg<fc0_max_khz::FC0_MAX_KHZ_SPEC>;
/// Frequency-counter maximum expected kHz. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod fc0_max_khz {
    pub struct FC0_MAX_KHZ_SPEC;
    impl crate::RegisterSpec for FC0_MAX_KHZ_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for FC0_MAX_KHZ_SPEC {}
    pub type R = crate::R<FC0_MAX_KHZ_SPEC>;
    impl crate::UnsafeWritable for FC0_MAX_KHZ_SPEC {}
    impl crate::UnsafeAtomicSet for FC0_MAX_KHZ_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for FC0_MAX_KHZ_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for FC0_MAX_KHZ_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Frequency-counter maximum expected kHz. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Frequency-counter start delay. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type FC0_DELAY = Reg<fc0_delay::FC0_DELAY_SPEC>;
/// Frequency-counter start delay. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod fc0_delay {
    pub struct FC0_DELAY_SPEC;
    impl crate::RegisterSpec for FC0_DELAY_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for FC0_DELAY_SPEC {}
    pub type R = crate::R<FC0_DELAY_SPEC>;
    impl crate::UnsafeWritable for FC0_DELAY_SPEC {}
    impl crate::UnsafeAtomicSet for FC0_DELAY_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for FC0_DELAY_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for FC0_DELAY_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Frequency-counter start delay. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Frequency-counter measurement interval. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type FC0_INTERVAL = Reg<fc0_interval::FC0_INTERVAL_SPEC>;
/// Frequency-counter measurement interval. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod fc0_interval {
    pub struct FC0_INTERVAL_SPEC;
    impl crate::RegisterSpec for FC0_INTERVAL_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for FC0_INTERVAL_SPEC {}
    pub type R = crate::R<FC0_INTERVAL_SPEC>;
    impl crate::UnsafeWritable for FC0_INTERVAL_SPEC {}
    impl crate::UnsafeAtomicSet for FC0_INTERVAL_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for FC0_INTERVAL_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for FC0_INTERVAL_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Frequency-counter measurement interval. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Frequency-counter source selector. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub type FC0_SRC = Reg<fc0_src::FC0_SRC_SPEC>;
/// Frequency-counter source selector. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write Plain.
/// Write API is not exposed because the verified operation semantics are insufficient.
pub mod fc0_src {
    pub struct FC0_SRC_SPEC;
    impl crate::RegisterSpec for FC0_SRC_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for FC0_SRC_SPEC {}
    pub type R = crate::R<FC0_SRC_SPEC>;
    impl crate::UnsafeWritable for FC0_SRC_SPEC {}
    impl crate::UnsafeAtomicSet for FC0_SRC_SPEC {
        const OFFSET: usize = 0x2000;
    }
    impl crate::UnsafeAtomicClear for FC0_SRC_SPEC {
        const OFFSET: usize = 0x3000;
    }
    impl crate::UnsafeAtomicXor for FC0_SRC_SPEC {
        const OFFSET: usize = 0x1000;
    }
    /// Frequency-counter source selector. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: Plain.
    /// Typed write API is not exposed because write behavior is not hardware/official verified.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

/// Frequency-counter status. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type FC0_STATUS = Reg<fc0_status::FC0_STATUS_SPEC>;
/// Frequency-counter status. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod fc0_status {
    pub struct FC0_STATUS_SPEC;
    impl crate::RegisterSpec for FC0_STATUS_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for FC0_STATUS_SPEC {}
    pub type R = crate::R<FC0_STATUS_SPEC>;
    /// Measurement complete.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod done {
        pub type R = crate::BitReader;
    }
    /// Measurement running.
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod running {
        pub type R = crate::BitReader;
    }
    impl R {
        #[inline(always)]
        pub fn done(&self) -> done::R {
            crate::BitReader::new(((self.bits() >> 4) & 1) != 0)
        }
        #[inline(always)]
        pub fn running(&self) -> running::R {
            crate::BitReader::new(((self.bits() >> 8) & 1) != 0)
        }
    }
}

/// Frequency-counter result; fractional portion begins at bit 5. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub type FC0_RESULT = Reg<fc0_result::FC0_RESULT_SPEC>;
/// Frequency-counter result; fractional portion begins at bit 5. \[Source: Linux RP1 common-clock driver.\]
/// Hardware access: read Plain; write None.
pub mod fc0_result {
    pub struct FC0_RESULT_SPEC;
    impl crate::RegisterSpec for FC0_RESULT_SPEC {
        type Ux = u32;
    }
    impl crate::Readable for FC0_RESULT_SPEC {}
    pub type R = crate::R<FC0_RESULT_SPEC>;
    /// Frequency-counter result; fractional portion begins at bit 5. \[Source: Linux RP1 common-clock driver.\]
    /// Field write behavior: ReadOnly.
    /// Read-only field; no writer is generated.
    pub mod value {
        pub type R = crate::FieldReader<u32>;
    }
    impl R {
        #[inline(always)]
        pub fn value(&self) -> value::R {
            crate::FieldReader::new((self.bits() >> 0) & 0xffffffff)
        }
    }
}

#[cfg(test)]
mod layout_tests {
    use super::RegisterBlock;
    #[test]
    fn gpclk_oe_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, gpclk_oe_ctrl), 0x0);
    }
    #[test]
    fn clk_sys_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_sys_ctrl), 0x14);
    }
    #[test]
    fn clk_sys_div_int_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_sys_div_int), 0x18);
    }
    #[test]
    fn clk_sys_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_sys_sel), 0x20);
    }
    #[test]
    fn clk_slow_sys_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_slow_sys_ctrl),
            0x24
        );
    }
    #[test]
    fn clk_slow_sys_div_int_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_slow_sys_div_int),
            0x28
        );
    }
    #[test]
    fn clk_slow_sys_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_slow_sys_sel), 0x30);
    }
    #[test]
    fn clk_dma_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_dma_ctrl), 0x44);
    }
    #[test]
    fn clk_dma_div_int_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_dma_div_int), 0x48);
    }
    #[test]
    fn clk_dma_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_dma_sel), 0x50);
    }
    #[test]
    fn clk_uart_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_uart_ctrl), 0x54);
    }
    #[test]
    fn clk_uart_div_int_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_uart_div_int), 0x58);
    }
    #[test]
    fn clk_uart_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_uart_sel), 0x60);
    }
    #[test]
    fn clk_eth_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_eth_ctrl), 0x64);
    }
    #[test]
    fn clk_eth_div_int_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_eth_div_int), 0x68);
    }
    #[test]
    fn clk_eth_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_eth_sel), 0x70);
    }
    #[test]
    fn clk_pwm0_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_pwm0_ctrl), 0x74);
    }
    #[test]
    fn clk_pwm0_div_int_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_pwm0_div_int), 0x78);
    }
    #[test]
    fn clk_pwm0_div_frac_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_pwm0_div_frac),
            0x7c
        );
    }
    #[test]
    fn clk_pwm0_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_pwm0_sel), 0x80);
    }
    #[test]
    fn clk_pwm1_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_pwm1_ctrl), 0x84);
    }
    #[test]
    fn clk_pwm1_div_int_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_pwm1_div_int), 0x88);
    }
    #[test]
    fn clk_pwm1_div_frac_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_pwm1_div_frac),
            0x8c
        );
    }
    #[test]
    fn clk_pwm1_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_pwm1_sel), 0x90);
    }
    #[test]
    fn clk_audio_in_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_audio_in_ctrl),
            0x94
        );
    }
    #[test]
    fn clk_audio_in_div_int_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_audio_in_div_int),
            0x98
        );
    }
    #[test]
    fn clk_audio_in_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_audio_in_sel), 0xa0);
    }
    #[test]
    fn clk_audio_out_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_audio_out_ctrl),
            0xa4
        );
    }
    #[test]
    fn clk_audio_out_div_int_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_audio_out_div_int),
            0xa8
        );
    }
    #[test]
    fn clk_audio_out_sel_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_audio_out_sel),
            0xb0
        );
    }
    #[test]
    fn clk_i2s_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_i2s_ctrl), 0xb4);
    }
    #[test]
    fn clk_i2s_div_int_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_i2s_div_int), 0xb8);
    }
    #[test]
    fn clk_i2s_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_i2s_sel), 0xc0);
    }
    #[test]
    fn clk_mipi0_cfg_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_mipi0_cfg_ctrl),
            0xc4
        );
    }
    #[test]
    fn clk_mipi0_cfg_div_int_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_mipi0_cfg_div_int),
            0xc8
        );
    }
    #[test]
    fn clk_mipi0_cfg_sel_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_mipi0_cfg_sel),
            0xd0
        );
    }
    #[test]
    fn clk_mipi1_cfg_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_mipi1_cfg_ctrl),
            0xd4
        );
    }
    #[test]
    fn clk_mipi1_cfg_div_int_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_mipi1_cfg_div_int),
            0xd8
        );
    }
    #[test]
    fn clk_mipi1_cfg_sel_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_mipi1_cfg_sel),
            0xe0
        );
    }
    #[test]
    fn clk_pcie_aux_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_pcie_aux_ctrl),
            0xe4
        );
    }
    #[test]
    fn clk_pcie_aux_div_int_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_pcie_aux_div_int),
            0xe8
        );
    }
    #[test]
    fn clk_pcie_aux_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_pcie_aux_sel), 0xf0);
    }
    #[test]
    fn clk_usbh0_microframe_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_usbh0_microframe_ctrl),
            0xf4
        );
    }
    #[test]
    fn clk_usbh0_microframe_div_int_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_usbh0_microframe_div_int),
            0xf8
        );
    }
    #[test]
    fn clk_usbh0_microframe_sel_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_usbh0_microframe_sel),
            0x100
        );
    }
    #[test]
    fn clk_usbh1_microframe_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_usbh1_microframe_ctrl),
            0x104
        );
    }
    #[test]
    fn clk_usbh1_microframe_div_int_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_usbh1_microframe_div_int),
            0x108
        );
    }
    #[test]
    fn clk_usbh1_microframe_sel_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_usbh1_microframe_sel),
            0x110
        );
    }
    #[test]
    fn clk_usbh0_suspend_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_usbh0_suspend_ctrl),
            0x114
        );
    }
    #[test]
    fn clk_usbh0_suspend_div_int_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_usbh0_suspend_div_int),
            0x118
        );
    }
    #[test]
    fn clk_usbh0_suspend_sel_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_usbh0_suspend_sel),
            0x120
        );
    }
    #[test]
    fn clk_usbh1_suspend_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_usbh1_suspend_ctrl),
            0x124
        );
    }
    #[test]
    fn clk_usbh1_suspend_div_int_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_usbh1_suspend_div_int),
            0x128
        );
    }
    #[test]
    fn clk_usbh1_suspend_sel_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_usbh1_suspend_sel),
            0x130
        );
    }
    #[test]
    fn clk_eth_tsu_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_eth_tsu_ctrl),
            0x134
        );
    }
    #[test]
    fn clk_eth_tsu_div_int_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_eth_tsu_div_int),
            0x138
        );
    }
    #[test]
    fn clk_eth_tsu_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_eth_tsu_sel), 0x140);
    }
    #[test]
    fn clk_adc_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_adc_ctrl), 0x144);
    }
    #[test]
    fn clk_adc_div_int_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_adc_div_int), 0x148);
    }
    #[test]
    fn clk_adc_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_adc_sel), 0x150);
    }
    #[test]
    fn clk_sdio_timer_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_sdio_timer_ctrl),
            0x154
        );
    }
    #[test]
    fn clk_sdio_timer_div_int_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_sdio_timer_div_int),
            0x158
        );
    }
    #[test]
    fn clk_sdio_timer_sel_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_sdio_timer_sel),
            0x160
        );
    }
    #[test]
    fn clk_sdio_alt_src_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_sdio_alt_src_ctrl),
            0x164
        );
    }
    #[test]
    fn clk_sdio_alt_src_div_int_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_sdio_alt_src_div_int),
            0x168
        );
    }
    #[test]
    fn clk_sdio_alt_src_sel_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_sdio_alt_src_sel),
            0x170
        );
    }
    #[test]
    fn clk_gp0_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp0_ctrl), 0x174);
    }
    #[test]
    fn clk_gp0_div_int_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp0_div_int), 0x178);
    }
    #[test]
    fn clk_gp0_div_frac_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_gp0_div_frac),
            0x17c
        );
    }
    #[test]
    fn clk_gp0_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp0_sel), 0x180);
    }
    #[test]
    fn clk_gp1_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp1_ctrl), 0x184);
    }
    #[test]
    fn clk_gp1_div_int_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp1_div_int), 0x188);
    }
    #[test]
    fn clk_gp1_div_frac_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_gp1_div_frac),
            0x18c
        );
    }
    #[test]
    fn clk_gp1_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp1_sel), 0x190);
    }
    #[test]
    fn clk_gp2_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp2_ctrl), 0x194);
    }
    #[test]
    fn clk_gp2_div_int_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp2_div_int), 0x198);
    }
    #[test]
    fn clk_gp2_div_frac_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_gp2_div_frac),
            0x19c
        );
    }
    #[test]
    fn clk_gp2_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp2_sel), 0x1a0);
    }
    #[test]
    fn clk_gp3_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp3_ctrl), 0x1a4);
    }
    #[test]
    fn clk_gp3_div_int_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp3_div_int), 0x1a8);
    }
    #[test]
    fn clk_gp3_div_frac_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_gp3_div_frac),
            0x1ac
        );
    }
    #[test]
    fn clk_gp3_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp3_sel), 0x1b0);
    }
    #[test]
    fn clk_gp4_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp4_ctrl), 0x1b4);
    }
    #[test]
    fn clk_gp4_div_int_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp4_div_int), 0x1b8);
    }
    #[test]
    fn clk_gp4_div_frac_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_gp4_div_frac),
            0x1bc
        );
    }
    #[test]
    fn clk_gp4_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp4_sel), 0x1c0);
    }
    #[test]
    fn clk_gp5_ctrl_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp5_ctrl), 0x1c4);
    }
    #[test]
    fn clk_gp5_div_int_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp5_div_int), 0x1c8);
    }
    #[test]
    fn clk_gp5_div_frac_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_gp5_div_frac),
            0x1cc
        );
    }
    #[test]
    fn clk_gp5_sel_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, clk_gp5_sel), 0x1d0);
    }
    #[test]
    fn clk_sys_resus_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_sys_resus_ctrl),
            0x20c
        );
    }
    #[test]
    fn clk_slow_sys_resus_ctrl_offset() {
        assert_eq!(
            core::mem::offset_of!(RegisterBlock, clk_slow_sys_resus_ctrl),
            0x214
        );
    }
    #[test]
    fn fc0_ref_khz_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, fc0_ref_khz), 0x21c);
    }
    #[test]
    fn fc0_min_khz_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, fc0_min_khz), 0x220);
    }
    #[test]
    fn fc0_max_khz_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, fc0_max_khz), 0x224);
    }
    #[test]
    fn fc0_delay_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, fc0_delay), 0x228);
    }
    #[test]
    fn fc0_interval_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, fc0_interval), 0x22c);
    }
    #[test]
    fn fc0_src_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, fc0_src), 0x230);
    }
    #[test]
    fn fc0_status_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, fc0_status), 0x234);
    }
    #[test]
    fn fc0_result_offset() {
        assert_eq!(core::mem::offset_of!(RegisterBlock, fc0_result), 0x238);
    }
    #[test]
    fn register_block_size() {
        assert_eq!(core::mem::size_of::<RegisterBlock>(), 0x23c);
    }
}
