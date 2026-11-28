use rp1_pac::{Interrupt, RegisterSpec, TypedAtomicClear, TypedAtomicSet, TypedAtomicXor};

#[allow(dead_code)]
fn compile_svdlite_style_access(resets: &rp1_pac::RESETS) {
    let _ = resets.ctrl1().read();
    resets.ctrl1().modify(|_r, w| w.uart0().set_bit());
    resets.ctrl1().atomic_set(|w| w.uart0().set());
    resets.ctrl1().atomic_clear(|w| w.uart0().clear());
    resets.ctrl1().atomic_xor(|w| w.uart0().toggle());
    unsafe {
        resets.ctrl1().atomic_set_bits_unchecked(1 << 26);
    }
}

#[allow(dead_code)]
fn compile_specialized_operations(timer: &rp1_pac::TIMER, watchdog: &rp1_pac::WATCHDOG) {
    timer.intr().clear(|w| w.alarm().clear_bits(1));
    watchdog.load().command(|w| w.value().bits(1_000));
    let _ = watchdog.boot_magic().read().value().bits();
    let _ = watchdog.scratch(0).read().value().bits();
}

#[test]
fn logical_base_addresses_and_interrupts_match_regdb() {
    assert_eq!(rp1_pac::TIMER::PTR as usize, 0x400a_c000);
    assert_eq!(rp1_pac::WATCHDOG::PTR as usize, 0x4015_4000);
    assert_eq!(rp1_pac::RESETS::PTR as usize, 0x4001_4000);
    assert_eq!(rp1_pac::UART0::PTR as usize, 0x4003_0000);
    assert_eq!(Interrupt::UART0 as u16, 25);
    assert_eq!(Interrupt::UART5 as u16, 46);
    assert_eq!(rp1_pac::NVIC_PRIO_BITS, 3);
    assert_eq!(rp1_pac::NVIC_INTERRUPT_COUNT, 64);
    assert_eq!(rp1_pac::DEFINED_INTERRUPT_COUNT, 61);
    assert_eq!(rp1_pac::Interrupt::IO_BANK0 as u16, 0);
    assert_eq!(rp1_pac::Interrupt::VBUSCTRL as u16, 60);
}

#[test]
fn core_capability_metadata_matches_regdb() {
    use rp1_pac::metadata::{self, ObservationResult};

    assert_eq!(metadata::CORE_CPUID, 0x412f_c231);
    assert_eq!(metadata::PROCESSOR_COUNT, 2);
    assert_eq!(metadata::NVIC_EXTERNAL_SLOTS, 64);
    assert_eq!(metadata::NVIC_ASSIGNED_INTERRUPTS, 61);
    assert_eq!(metadata::NVIC_PRIORITY_BITS, 3);
    assert_eq!(metadata::MPU_DATA_REGIONS, 8);
    assert_eq!(metadata::MPU_INSTRUCTION_REGIONS, 0);
    assert_eq!(metadata::SYSTICK_CALIB, 0x1387);
    assert_eq!(metadata::DWT_COMPARATORS, 4);
    assert_eq!(metadata::FPB_CODE_COMPARATORS, 6);
    assert_eq!(metadata::FPB_LITERAL_COMPARATORS, 2);
    assert_eq!(
        metadata::EXCLUSIVE_STORE.result,
        ObservationResult::NotObserved
    );
    assert_eq!(
        metadata::BOOT_ROM_PUBLIC_LOOKUP_TABLE.result,
        ObservationResult::NotObserved
    );
}

#[test]
fn register_layout_uses_32_bit_cells() {
    assert_eq!(
        core::mem::size_of::<rp1_pac::generated::timer::RAW_HIGH>(),
        4
    );
    assert_eq!(
        core::mem::size_of::<rp1_pac::generated::watchdog::LOAD>(),
        4
    );
    assert_eq!(core::mem::size_of::<rp1_pac::generated::resets::CTRL1>(), 4);
}

#[test]
fn atomic_offsets_are_attached_to_register_specs() {
    type Ctrl1 = rp1_pac::generated::resets::ctrl1::CTRL1_SPEC;
    type Msix = rp1_pac::generated::pcie_apbs::msix_cfg::MSIX_CFG_SPEC;

    assert_eq!(<Ctrl1 as TypedAtomicXor>::OFFSET, 0x1000);
    assert_eq!(<Ctrl1 as TypedAtomicSet>::OFFSET, 0x2000);
    assert_eq!(<Ctrl1 as TypedAtomicClear>::OFFSET, 0x3000);
    assert_eq!(<Msix as TypedAtomicSet>::OFFSET, 0x0800);
    assert_eq!(<Msix as TypedAtomicClear>::OFFSET, 0x0c00);
    assert_eq!(core::mem::size_of::<<Ctrl1 as RegisterSpec>::Ux>(), 4);
}

#[test]
fn singleton_take_rejects_second_owner() {
    let first = unsafe { rp1_pac::Peripherals::take() };
    assert!(first.is_some());
    assert!(unsafe { rp1_pac::Peripherals::take() }.is_none());
}

#[test]
fn unsafe_atomic_api_documents_its_safety_contract() {
    let generic = include_str!("../src/generic.rs");
    assert!(generic.contains("# Safety"));
    assert!(generic.contains("reserved, inferred, command, or otherwise unsafe bits"));
    assert!(generic.contains("atomic_set_bits_unchecked"));
    assert!(!generic.contains("pub fn atomic_set_bits("));
}

#[test]
fn pio_windows_have_distinct_fifo_offsets_and_interleaved_control_registers() {
    use rp1_pac::generated::{pio, pio_m3};
    // The test holds ordinary zero-filled storage. It never reads or writes MMIO.
    let apb: pio::RegisterBlock = unsafe { core::mem::zeroed() };
    let local: pio_m3::RegisterBlock = unsafe { core::mem::zeroed() };
    let a = &raw const apb as usize;
    let m = &raw const local as usize;
    let offset = |reg: &_| reg as *const _ as usize - m;
    assert_eq!(rp1_pac::PIO::PTR as usize, 0x4017_8000);
    assert_eq!(rp1_pac::PIO_M3::PTR as usize, 0xf000_0000);
    for sm in 0..4 {
        assert_eq!(apb.txf(sm) as *const _ as usize - a, 4 * sm);
        assert_eq!(apb.rxf(sm) as *const _ as usize - a, 0x10 + 4 * sm);
        assert_eq!(local.txf(sm) as *const _ as usize - m, 0x14 + 4 * sm);
        assert_eq!(local.rxf(sm) as *const _ as usize - m, 0x24 + 4 * sm);
        let regs = [
            local.sm_clkdiv(sm) as *const _ as usize,
            local.sm_execctrl(sm) as *const _ as usize,
            local.sm_shiftctrl(sm) as *const _ as usize,
            local.sm_addr(sm) as *const _ as usize,
            local.sm_instr(sm) as *const _ as usize,
            local.sm_pinctrl(sm) as *const _ as usize,
            local.sm_dmatx(sm) as *const _ as usize,
            local.sm_dmarx(sm) as *const _ as usize,
        ];
        for (i, reg) in regs.into_iter().enumerate() {
            assert_eq!(reg - m, 0xcc + 0x20 * sm + 4 * i);
        }
    }
    assert_eq!(local.instr_mem_array().len(), 32);
    for n in 0..32 {
        assert_eq!(offset(local.instr_mem(n)), 0x4c + 4 * n);
    }
    assert_eq!(local.intr() as *const _ as usize - m, 0x14c);
    assert_eq!(local.irq1_ints() as *const _ as usize - m, 0x164);
    assert_eq!(core::mem::size_of_val(&local), 0x168);
}

#[allow(dead_code)]
fn pio_access(apb: &rp1_pac::PIO) {
    apb.txf(0).push(1);
    let _ = apb.rxf(0).pop();
    let pio = apb.pio_m3();
    pio.txf(0).push(1);
    let _ = pio.rxf(0).pop();
    let _ = pio.sm_addr(0).read().bits();
    unsafe { pio.instr_mem(31).write_bits_unchecked(0) };
}
