use super::*;
use crate::{
    metadata::{Declaration, TypeInfo},
    resources, wire,
};
use std::{fs, path::Path};

fn fixture() -> Declaration {
    let mut d: Declaration = toml::from_str(
        r#"
schema = 2
[proc0]
host_irq = true
[proc1]
doorbell = 27

[[shm]]
name = "type"
kind = "ring"
core = "proc0"
direction = "rp1-to-linux"
notify = "irq"
overflow = "overwrite"
message = "Word"
depth = 32

[[shm]]
name = "down"
kind = "ring"
core = "proc1"
direction = "linux-to-rp1"
notify = "irq"
message = "crate::types::Word"
depth = 256

[[shm]]
name = "trace"
kind = "block"
core = "proc1"
type = "core::primitive::u32"
writer = "linux"
"#,
    )
    .unwrap();
    d.types.extend([
        ("Word".into(), TypeInfo::of::<[u32; 4]>()),
        ("crate::types::Word".into(), TypeInfo::of::<[u32; 4]>()),
        ("core::primitive::u32".into(), TypeInfo::of::<u32>()),
    ]);
    d.validate().unwrap();
    d
}

#[test]
fn generated_resources_only_reference_their_link_features() {
    let plain = Declaration::default();
    let source = resources(&plain, &[], &plain.shm_layout().unwrap()).unwrap();
    assert!(!source.contains("rp1_link"));
    assert!(!source.contains("pub mod link"));

    let mut d = fixture();
    let full = resources(&d, &[], &d.shm_layout().unwrap()).unwrap();
    assert!(full.contains("pub link: link::Proc0LinkInit"));
    assert!(full.contains("pub link: link::Proc1LinkInit"));
    assert!(full.contains("fn doorbell_proc1()"));
    assert!(!full.contains("fn doorbell_proc0()"));
    assert!(full.contains(r#"export_name = "__rp1_p1_IO_BANK0""#));
    assert_eq!(
        full.matches("impl sealed::Admitted for IO_BANK0").count(),
        0
    );
    d.proc1.gpio = vec![22];
    d.proc1.gpio_irq = vec![22];
    assert!(
        d.validate()
            .unwrap_err()
            .to_string()
            .contains("shares IO_BANK0")
    );
    d.proc1.custom_doorbell_handler = true;
    let custom = resources(&d, &[], &d.shm_layout().unwrap()).unwrap();
    assert!(!custom.contains("fn doorbell_proc1()"));
    assert_eq!(
        custom.matches("impl sealed::Admitted for IO_BANK0").count(),
        1
    );
    assert!(d.validate().is_ok());
    d.proc1.custom_doorbell_handler = false;
    d.proc1.gpio_irq.clear();
    d.proc1.gpio.clear();

    d.shm.remove(0);
    let source = resources(&d, &[], &d.shm_layout().unwrap()).unwrap();
    assert!(!source.contains("Proc0Link"));
    assert!(source.contains("pub link: link::Proc1LinkInit"));
    assert!(source.contains("pub host_irq: ::rp1_link::notify::HostIrq"));

    d.shm.clear();
    let irq_only = resources(&d, &[], &d.shm_layout().unwrap()).unwrap();
    assert!(!irq_only.contains("pub mod link"));
    assert!(irq_only.contains("pub host_irq: ::rp1_link::notify::HostIrq"));
    // A doorbell exists without shm too: pre-Ready users answer it.
    assert!(irq_only.contains("pub type Doorbell1 = ::rp1_link::notify::Doorbell<27, 1>;"));
    assert!(irq_only.contains("pub doorbell: Doorbell1,"));
    assert!(irq_only.contains("doorbell: unsafe { Doorbell1::new() },"));
    assert!(!irq_only.contains("Doorbell0"));

    d.proc1.doorbell = None;
    d.proc0.host_irq = false;
    let no_notifier = resources(&d, &[], &d.shm_layout().unwrap()).unwrap();
    assert!(!no_notifier.contains("rp1_link"));
}

#[test]
fn every_region_has_an_alias_that_names_its_notifier() {
    let d = fixture();
    let (body, _) = fragment(&d);
    for alias in [
        "pub type Type = transport::RingTx<crate::Word, transport::notify::HostIrq>;",
        "pub type Down = transport::RingRx<crate::types::Word, super::Doorbell1>;",
        "pub type Trace = transport::Block<::core::primitive::u32>;",
        "pub r#type: Type,",
        "pub down: Down,",
        "r#type: unsafe { generated::ring_tx(REGIONS[0]) }?,",
        "down: unsafe { generated::ring_rx(REGIONS[1]) }?,",
    ] {
        assert!(body.contains(alias), "{alias}\n{body}");
    }
    let mut poll = d.clone();
    for r in &mut poll.shm {
        r.notify = None;
    }
    poll.proc0.host_irq = false;
    let (body, _) = fragment(&poll);
    assert!(
        body.contains("pub type Type = transport::RingTx<crate::Word>;"),
        "{body}"
    );
    assert!(
        body.contains("pub type Down = transport::RingRx<crate::types::Word>;"),
        "{body}"
    );
}

#[test]
fn aliases_must_be_distinct_and_free() {
    let mut d = fixture();
    d.shm[1].name = "ty_pe".into();
    d.shm[0].name = "ty__pe".into();
    let error = generate(&d.shm_layout().unwrap(), &[], false).unwrap_err();
    assert_eq!(
        error.to_string(),
        "SHM names \"ty__pe\" and \"ty_pe\" both give the type alias TyPe; rename one"
    );
    let mut d = fixture();
    d.shm[1].name = "proc1_link".into();
    let error = generate(&d.shm_layout().unwrap(), &[], false).unwrap_err();
    assert!(
        error.to_string().contains("type alias Proc1Link"),
        "{error}"
    );
}

#[test]
fn unrepresentable_field_names_point_to_the_declaration() {
    let mut d = fixture();
    d.shm[0].name = "self".into();
    let error = generate(&d.shm_layout().unwrap(), &[], false).unwrap_err();
    assert_eq!(
        error.to_string(),
        "SHM name \"self\" cannot be a Rust field; rename this region"
    );
}

#[test]
fn reserved_field_names_become_raw_identifiers() {
    for name in [
        "abstract", "become", "box", "final", "macro", "priv", "try", "yield",
    ] {
        assert_eq!(field_name(name).unwrap(), format!("r#{name}"));
    }
    assert_eq!(field_name("control").unwrap(), "control");
}

fn fragment(d: &Declaration) -> (String, Vec<u8>) {
    let shm = d.shm_layout().unwrap();
    let note =
        wire::descriptor_with_layout("fixture", "0.1.0", d, [0x45; 32], [0x67; 20], &shm).unwrap();
    (generate(&shm, &note, false).unwrap(), note)
}

fn write_case(dir: &Path, name: &str, source: String) -> std::path::PathBuf {
    let path = dir.join(format!("{name}.rs"));
    fs::write(
        &path,
        format!(
            "type Word = [u32; 4];\nmod types {{ pub type Word = super::Word; }}\n\
             type Doorbell1 = ::rp1_link::notify::Doorbell<27, 1>;\n{source}"
        ),
    )
    .unwrap();
    let snapshot = Path::new("tests/ui-link").join(format!("{name}.stderr"));
    if name != "pass" {
        let expected = fs::read_to_string(snapshot).unwrap();
        fs::write(
            path.with_extension("stderr"),
            expected.replace("$CASE", path.to_str().unwrap()),
        )
        .unwrap();
    }
    path
}

#[test]
fn generated_types_and_layout() {
    let d = fixture();
    let (body, note) = fragment(&d);
    let dir = tempfile::tempdir().unwrap();
    let t = trybuild::TestCases::new();
    let pass = include_str!("../../tests/ui-link/pass.inc");
    t.pass(write_case(
        dir.path(),
        "pass",
        format!("mod link {{\n{body}\nconst NOTE: &[u8] = &{note:?};\n{pass}"),
    ));

    let mut wrong_size = d.clone();
    wrong_size.types.get_mut("Word").unwrap().size = 24;
    let mut wrong_id = d.clone();
    wrong_id.types.get_mut("Word").unwrap().id ^= 1;
    for (name, declaration) in [("wrong_size", wrong_size), ("wrong_id", wrong_id)] {
        let (body, _) = fragment(&declaration);
        t.compile_fail(write_case(
            dir.path(),
            name,
            format!("mod link {{\n{body}}}\nfn main() {{}}"),
        ));
    }

    t.compile_fail(write_case(
        dir.path(),
        "open_twice",
        format!(
            "mod link {{\n{body}}}\n\
         fn reopen(init: link::Proc1LinkInit) {{\n\
             let _ = init.try_open();\n\
             let _ = init.try_open();\n\
         }}\n\
         fn main() {{}}",
        ),
    ));
}

const PING: &str = r#"
[[shm]]
name = "ping"
kind = "rpc"
core = "proc1"
message = "Word"
reply = "Pong"
depth = 1
notify = { dn = "irq", up = "poll" }
"#;

fn named(body: &str) -> Declaration {
    let mut d: Declaration =
        toml::from_str(&format!("schema = 2\n[proc1]\ndoorbell = 27\n{body}")).unwrap();
    d.types.extend([
        ("Word".into(), TypeInfo::of::<[u32; 4]>()),
        ("Pong".into(), TypeInfo::of::<u32>()),
    ]);
    d
}

#[test]
fn named_endpoints_share_one_layout() {
    use rp1_proto::note::{Note, RPC, SHM_REGION, rpc_name};

    let events = PING
        .replace("ping", "events")
        .replace("rpc", "channel")
        .replace("depth = 1", "depth = { dn = 2, up = 4 }");
    let status = PING
        .replace("ping", "status")
        .replace("depth = 1", "depth = 8");
    let d = named(&format!("{PING}{events}{status}"));
    d.validate().unwrap();
    let (body, desc) = fragment(&d);
    let note = Note::parse(&desc).unwrap();
    let names: Vec<_> = note
        .records()
        .map(std::result::Result::unwrap)
        .filter(|record| record.kind == RPC)
        .map(|record| {
            assert_eq!(record.flags, 0);
            rpc_name(record.value).unwrap()
        })
        .collect();
    assert_eq!(names, ["ping", "status"]);
    let regions: Vec<_> = note
        .records()
        .map(std::result::Result::unwrap)
        .filter(|r| r.kind == SHM_REGION)
        .map(|r| ShmRegion::parse(r.value).unwrap())
        .collect();
    let expected = [
        ("ping-dn", ShmKind::RingToRp1, 1),
        ("ping-up", ShmKind::RingToLinux, 1),
        ("events-dn", ShmKind::RingToRp1, 2),
        ("events-up", ShmKind::RingToLinux, 4),
        ("status-dn", ShmKind::RingToRp1, 8),
        ("status-up", ShmKind::RingToLinux, 8),
    ];
    assert_eq!(
        regions
            .iter()
            .map(|r| (r.name, r.kind, r.depth))
            .collect::<Vec<_>>(),
        expected
    );
    let out = tempfile::tempdir().unwrap();
    let dts = out.path().join("named.dts");
    let dtbo = out.path().join("named.dtbo");
    fs::write(&dts, crate::overlay::dts(&d, &desc).unwrap()).unwrap();
    crate::overlay::compile(&dts, &dtbo).unwrap();
    let overlay = fs::read(dtbo).unwrap();
    for (r, index) in regions.iter().zip(0..) {
        assert_eq!(r.notify_irq, index % 2 == 0);
        assert!(!r.overwrite);
        let name = format!("rp1:{}\0", r.name);
        assert!(
            overlay
                .windows(name.len())
                .any(|bytes| bytes == name.as_bytes())
        );
    }

    let t = trybuild::TestCases::new();
    t.pass(write_case(out.path(), "pass", format!(
        "type Pong = u32;\nmod link {{\n{body}}}\n\
         fn main() {{\n\
             let _: fn(link::Proc1Link) -> rp1_link::Server<Word, Pong, rp1_link::notify::Poll, Doorbell1> = |l| l.ping;\n\
             let _: fn(link::Proc1Link) -> rp1_link::Channel<Pong, Word, rp1_link::notify::Poll, Doorbell1> = |l| l.events;\n\
         }}"
    )));
}

#[test]
fn named_endpoint_limits() {
    for bad in [
        PING.replace("depth = 1", "depth = { dn = 1, up = 1 }"),
        PING.replace("ping", "abcdefghijklmnopqrstuv"),
        PING.replace("ping", "header"),
        format!("{PING}{PING}"),
    ]
    .into_iter()
    .chain(
        ["direction", "overflow", "writer", "type", "placement"]
            .map(|key| format!("{PING}{key} = \"extra\"\n")),
    ) {
        assert!(named(&bad).validate().is_err(), "{bad}");
    }
    named(&PING.replace("ping", "abcdefghijklmnopqrstu"))
        .validate()
        .unwrap();

    let d = named(&format!(
        "{}{}",
        PING.replace("ping", "pi_ng"),
        PING.replace("ping", "pi__ng")
    ));
    assert!(
        generate(&d.shm_layout().unwrap(), &[], false)
            .unwrap_err()
            .to_string()
            .contains("PiNg")
    );

    let mut d = named(
        &(0..32)
            .map(|i| PING.replace("ping", &format!("ping{i}")))
            .collect::<String>(),
    );
    d.validate().unwrap();
    let (_, desc) = fragment(&d);
    assert!(rp1_proto::note::Note::parse(&desc).is_ok());
    d.shm.push(
        named(
            &PING
                .replace("ping", "more")
                .replace("rpc", "ring")
                .replace("reply = \"Pong\"", "direction = \"linux-to-rp1\"")
                .replace(
                    "notify = { dn = \"irq\", up = \"poll\" }",
                    "notify = \"irq\"",
                ),
        )
        .shm
        .remove(0),
    );
    assert!(d.validate().unwrap_err().to_string().contains("at most 64"));
}

#[test]
fn generated_ownership_is_checked_by_rust() {
    let dir = tempfile::tempdir().unwrap();
    let mut d = fixture();
    d.proc0.gpio = vec![22];
    d.proc0.gpio_irq = vec![22];
    d.proc1.timer = vec!["ALARM0".into()];
    d.proc1.instances.insert(
        "dma".into(),
        crate::metadata::Instance::Dma(crate::metadata::DmaChannels {
            channels: Some(vec![0, 2]),
        }),
    );
    d.validate().unwrap();
    let generated = resources(&d, &[], &d.shm_layout().unwrap())
        .unwrap()
        .replace(
            r#"include_bytes!(concat!(env!("OUT_DIR"),"/rp1-app.dtbo"))"#,
            r#"b"""#,
        );
    fs::write(dir.path().join("resources.inc"), generated).unwrap();
    let harness = r#"
type Word = [u32; 4];
mod types { pub type Word = super::Word; }
include!("resources.inc");
"#;
    let t = trybuild::TestCases::new();
    let pass = dir.path().join("ownership_pass.rs");
    fs::write(
        &pass,
        format!(
            "{harness}
#[rp1_rt::interrupt(proc1)]
fn TIMER_0() {{}}
fn channels(dma: rp1::Proc1DmaChannels) {{
    let _: rp1_hal::dma::Channel<0> = dma.ch0;
    let _: rp1_hal::dma::Channel<2> = dma.ch2;
}}
fn gpio(p: rp1::Proc0Resources) -> rp1_hal::gpio::Pin<22> {{ p.pins.gpio22 }}
#[rp1_rt::interrupt(proc0)]
fn IO_BANK0() {{}}
fn alarm(irq: rp1::proc1_irq::TIMER_0) -> rp1_rt::interrupt::Binding<rp1_rt::interrupt::AlarmIrq<0>, 26, 1> {{ irq }}
{}",
            include_str!("../../tests/ui-link/ownership_core_local.inc"),
        ),
    )
    .unwrap();
    t.pass(pass);
    for (name, body) in [
        (
            "ownership_admission",
            r#"fn use_pin(_: rp1_hal::gpio::Pin<22>) {}
fn duplicate_pin(p: rp1::Proc0Resources) {
    use_pin(p.pins.gpio22);
    use_pin(p.pins.gpio22);
}
fn wrong_route(p: rp1::Proc0Resources) {
    let _ = rp1_hal::uart::Pins::<1>::builder().tx(p.pins.gpio22);
}
fn host_alarm(p: rp1::Proc0Resources) {
    let _ = p.TIMER_ALARM3;
}
fn unlisted_channel(dma: rp1::Proc1DmaChannels) {
    let _ = dma.ch1;
}
fn main() {}
"#,
        ),
        (
            "ownership_private",
            r#"fn private_resources(p: rp1::Proc0Resources) {
    let _ = rp1::Proc0Pins { ..p.pins };
}
fn main() {}
"#,
        ),
        (
            "ownership_irq_sealed",
            r#"use rp1::__irq_proc0::{Declared, TIMER_0};
impl Declared for TIMER_0 {}
fn main() {}
"#,
        ),
        (
            "ownership_irq",
            r#"#[rp1_rt::interrupt(proc0)]
fn TIMER_0() {}
#[rp1_rt::interrupt(proc1)]
fn IO_BANK0() {}
#[rp1_rt::interrupt(proc1)]
fn TIMER_0() {}
#[rp1_rt::interrupt(proc1)]
fn SPI5() {}
mod duplicate {
    #[rp1_rt::interrupt(proc1)]
    fn TIMER_0() {}
}
fn main() {}
"#,
        ),
    ] {
        let path = dir.path().join(format!("{name}.rs"));
        fs::write(&path, format!("{harness}{body}")).unwrap();
        if let Ok(expected) = fs::read_to_string(format!("tests/ui-link/{name}.stderr")) {
            fs::write(
                path.with_extension("stderr"),
                expected.replace("$CASE", path.to_str().unwrap()).replace(
                    "$GENERATED",
                    dir.path().join("resources.inc").to_str().unwrap(),
                ),
            )
            .unwrap();
        }
        t.compile_fail(path);
    }
}
