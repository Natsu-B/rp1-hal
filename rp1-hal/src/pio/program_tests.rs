use super::*;
use crate::pio::tests::block;

fn words<const L: usize>(len: usize) -> Program<L> {
    Program {
        code: (0..len).map(|_| 0xa042).collect(),
        origin: None,
        wrap: pio_core::Wrap {
            source: len.saturating_sub(1) as u8,
            target: 0,
        },
        side_set: SideSet::default(),
        version: PioVersion::V0,
    }
}

#[test]
fn placement_handles_boundaries_holes_and_fixed_origins() {
    let pio = block();
    let install = |program: &Program<32>| pio.install_with(program, |_, _| {});
    assert_eq!(install(&words(0)).err(), Some(InstallError::InvalidProgram));
    assert_eq!(
        validate(&words::<33>(33)).err(),
        Some(InstallError::InvalidProgram)
    );
    assert_eq!(
        install(&words(2).set_origin(Some(31))).err(),
        Some(InstallError::InvalidProgram)
    );
    let full = install(&words(32)).unwrap();
    assert_eq!(full.offset(), 0);
    assert_eq!(install(&words(1)).err(), Some(InstallError::NoSpace));
    full.uninstall().unwrap();

    let upper = install(&words(3)).unwrap();
    let hole = install(&words(4)).unwrap();
    let lower = install(&words(3)).unwrap();
    assert_eq!(
        (upper.offset(), hole.offset(), lower.offset()),
        (29, 25, 22)
    );
    assert_eq!(
        install(&words(4).set_origin(Some(26))).err(),
        Some(InstallError::OriginOccupied)
    );
    hole.uninstall().unwrap();
    let replacement = install(&words(4)).unwrap();
    assert_eq!(replacement.offset(), 25);
}

#[test]
fn relocates_jmp_and_wrap_at_a_fixed_origin() {
    let pio = block();
    let mut p = words::<32>(4)
        .set_origin(Some(24))
        .set_wrap(pio_core::Wrap {
            source: 3,
            target: 1,
        });
    p.code[1] = 0x0563;
    let mut mem = [0; 32];
    let installed = pio
        .install_with(&p, |index, word| mem[index] = word)
        .unwrap();
    assert_eq!(installed.offset(), 24);
    assert_eq!((installed.wrap_bottom, installed.wrap_top), (25, 27));
    assert_eq!(&mem[24..28], &[0xa042, 0x057b, 0xa042, 0xa042]);
}

#[test]
fn invalid_programs_never_write_instruction_memory() {
    let pio = block();
    let reject = |p: &Program<32>| {
        pio.install_with(p, |_, _| panic!("wrote before validation"))
            .err()
            .unwrap()
    };
    for word in [
        0x6080 | (3 << 5),
        0xa0a1,
        0xa081,
        0x8018,
        0x2000 | (3 << 5),
        0xc008,
    ] {
        let mut p = words(1);
        p.code[0] = word;
        assert_eq!(
            reject(&p),
            InstallError::UnsupportedInstruction,
            "{word:#06x}"
        );
    }
    let mut p = words(2);
    p.code[0] = 2;
    assert_eq!(reject(&p), InstallError::InvalidProgram);
    p.code[0] = 0xa042;
    p.wrap.source = 0;
    assert_eq!(reject(&p), InstallError::InvalidProgram);
    p.wrap.source = 1;
    p.side_set = SideSet::new(true, 5, false);
    assert_eq!(reject(&p), InstallError::InvalidProgram);
    p.side_set = SideSet::default();
    p.version = PioVersion::V1;
    assert_eq!(reject(&p), InstallError::UnsupportedInstruction);
}

#[test]
fn reused_and_forgotten_leases_cannot_be_overwritten() {
    let pio = block();
    let program = words::<32>(1).set_origin(Some(31));
    let install = || pio.install_with(&program, |_, _| {});
    let original = install().unwrap();
    fn preserved(p: InstalledProgram<'_>) -> InstalledProgram<'_> {
        let failed = p.uninstall().unwrap_err();
        assert_eq!(*failed.error(), InstallError::InUse);
        failed.into_driver()
    }
    pio.active[0].set(original.mask);
    pio.active[1].set(original.mask);
    let original = preserved(original);
    assert_eq!(install().err(), Some(InstallError::OriginOccupied));
    pio.active[0].set(0);
    let original = preserved(original);
    assert_eq!(install().err(), Some(InstallError::OriginOccupied));
    pio.active[1].set(0);
    original.uninstall().unwrap();
    // InstalledProgram has a hardware lease even when the Rust value is gone.
    let _forgotten = core::mem::ManuallyDrop::new(install().unwrap());
    assert_eq!(install().err(), Some(InstallError::OriginOccupied));
}
