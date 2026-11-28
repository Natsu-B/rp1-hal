use rp1_build::{
    board::{Gpiomem, OnConflict},
    metadata::{Declaration, Instance},
};

fn parse(body: &str) -> Declaration {
    toml::from_str(&format!("schema = 2\n{body}")).unwrap()
}
fn error(body: &str, expected: &[&str]) {
    let result = toml::from_str::<Declaration>(&format!("schema = 2\n{body}"))
        .map_err(|e| e.to_string())
        .and_then(|d| d.validate().map_err(|e| e.to_string()));
    let message = result.unwrap_err();
    for text in expected {
        assert!(message.contains(text), "expected {text:?} in {message}");
    }
}

#[test]
fn requires_schema_2() {
    for body in ["schema = 1\n[proc0]\npins = [22]", "schema = 3", ""] {
        let message = toml::from_str::<Declaration>(body).unwrap_err().to_string();
        assert!(
            message.contains("E0: manifest requires schema = 2"),
            "{body:?}: {message}"
        );
    }
}

#[test]
fn pin_twice_across_cores() {
    error(
        "[proc0]\ngpio=[10]\n[proc1]\nspi0={sck=11,mosi=10,miso=9}",
        &["E1", "GPIO 10", "proc0.gpio", "proc1.spi0.mosi"],
    );
}
#[test]
fn pin_twice_function_and_gpio() {
    error(
        "[proc0]\ngpio=[4]\nuart2={tx=4,rx=5}",
        &["E1", "proc0.gpio", "proc0.uart2.tx"],
    );
}
#[test]
fn pin_twice_across_functions() {
    error(
        "[proc0]\nuart2={tx=4,rx=5}\ni2c2={sda=4,scl=5}",
        &["E1", "GPIO 4", "proc0.uart2.tx", "proc0.i2c2.sda"],
    );
}
#[test]
fn pin_vs_log() {
    error(
        "log={sink='uart0',baudrate=115200}\n[proc0]\ni2c3={sda=14,scl=15}",
        &["E1", "log.uart0.tx", "proc0.i2c3.sda"],
    );
}
#[test]
fn pin_vs_linux_keep() {
    error(
        "[linux]\nkeep={uart1={tx=0,rx=1},i2c1={sda=2,scl=3}}\n[proc0]\ngpio=[2]",
        &["E1", "linux.keep.i2c1.sda", "proc0.gpio"],
    );
}
#[test]
fn wrong_signal_pin() {
    error(
        "[proc0]\nuart2={tx=6,rx=5}",
        &["E2", "proc0.uart2.tx = 6", "UART2_CTS", "alt2", "GPIO4"],
    );
}
#[test]
fn pin_without_that_block() {
    error(
        "[proc0]\nspi0={sck=4,mosi=10,miso=9}",
        &["E2", "proc0.spi0.sck = 4", "GPIO11"],
    );
}
#[test]
fn uart5_has_no_bank0_pins() {
    error(
        "[proc0]\nuart5={tx=4,rx=5}",
        &["E2", "UART5 has no bank-0 pins"],
    );
}
#[test]
fn gpio_out_of_range() {
    error("[proc0]\ngpio=[54]", &["E2", "GPIO 54 is out of range"]);
}
#[test]
fn instance_twice() {
    error(
        "[proc0]\ni2c1={sda=2,scl=3}\n[proc1]\ni2c1={sda=10,scl=11}",
        &["E3", "I2C1", "proc0", "proc1"],
    );
}
#[test]
fn log_and_core_uart0() {
    error(
        "log={sink='uart0',baudrate=115200}\n[proc0]\nuart0={tx=14,rx=15}",
        &["E3", "UART0", "log sink", "proc0"],
    );
}
#[test]
fn instance_vs_linux_keep() {
    error(
        "[proc0]\nuart1={tx=0,rx=1}",
        &["E3", "UART1", "Linux by board rpi-cm5 (default linux.keep)"],
    );
}
#[test]
fn explicit_linux_keep_is_not_attributed_to_the_board() {
    error(
        "[linux]\nkeep={i2c1={sda=2,scl=3}}\n[proc0]\ni2c1={sda=2,scl=3}",
        &["E3", "I2C1", "Linux by linux.keep", "proc0"],
    );
    let message = toml::from_str::<Declaration>(
        "schema = 2\n[linux]\nkeep={uart1={tx=0,rx=1}}\n[proc0]\ngpio=[0]",
    )
    .unwrap()
    .validate()
    .unwrap_err()
    .to_string();
    assert!(
        message.contains("E1: GPIO 0 is claimed by both linux.keep.uart1.tx"),
        "{message}"
    );
    assert!(!message.contains("board"), "{message}");
}
#[test]
fn board_line_pin() {
    error(
        "[proc0]\ngpio=[32]",
        &["E4", "GPIO 32", "ETH_RST_N", "Linux on rpi-cm5"],
    );
    for (pin, line) in [
        (42, "USB_VBUS_EN"),
        (43, "board GPIO"),
        (44, "RP1_STAT_LED"),
    ] {
        error(&format!("[proc0]\ngpio=[{pin}]"), &["E4", line]);
    }
    for pin in 28..=53 {
        error(&format!("[proc0]\ngpio=[{pin}]"), &["E4", "owned by Linux"]);
    }
}
#[test]
fn uart1_policy_pins() {
    for pin in [0, 1] {
        error(
            &format!("[proc0]\ngpio=[{pin}]"),
            &["E4", "kept for Linux", "uart1.", "board rpi-cm5"],
        );
    }
}
#[test]
fn linux_required_block() {
    for block in ["eth", "usbhost0", "usbhost1", "pcie"] {
        error(
            &format!("[proc0]\n{block}={{}}"),
            &["E4", "required by Linux on rpi-cm5"],
        );
    }
}
#[test]
fn spi_slave_only() {
    for n in [4, 7] {
        error(&format!("[proc0]\nspi{n}={{}}"), &["E4", "slave-only"]);
    }
}
#[test]
fn spi2_conflicts_with_the_default_linux_uart1_pins() {
    let spi2 = "[proc0]\nspi2={sck=3,mosi=2,miso=1}";
    error(spi2, &["E4", "kept for Linux", "uart1.rx"]);
    parse(&format!("[linux]\nkeep={{}}\n{spi2}"))
        .validate()
        .unwrap();
}
#[test]
fn serial_irq_declarations_require_exclusive_ownership() {
    for (instance, pins) in [
        ("uart0", "tx=14,rx=15"),
        ("uart2", "tx=4,rx=5"),
        ("uart3", "tx=8,rx=9"),
        ("uart4", "tx=12,rx=13"),
        ("spi1", "sck=21,mosi=20,miso=19"),
        ("spi2", "sck=3,mosi=2,miso=1"),
        ("spi3", "sck=7,mosi=6,miso=5"),
        ("spi5", "sck=15,mosi=14,miso=13"),
        ("i2c0", "sda=8,scl=9"),
        ("i2c2", "sda=4,scl=5"),
        ("i2c3", "sda=22,scl=23"),
    ] {
        error(
            &format!(
                "[linux]\nkeep={{}}\n[proc0]\nwait_timer='ALARM0'\n{instance}={{{pins}}}\n[proc1]\nwait_timer='ALARM1'\n{instance}={{{pins}}}"
            ),
            &["E3", &instance.to_ascii_uppercase(), "proc0", "proc1"],
        );
        let custom = format!("[proc0]\ncustom_{instance}_handler=true");
        if matches!(instance, "spi2" | "spi5") {
            error(
                &custom,
                &["E0", &format!("unknown key 'custom_{instance}_handler'")],
            );
        } else {
            error(
                &custom,
                &["E5", &format!("custom_{instance}_handler needs {instance}")],
            );
        }
    }
}

#[test]
fn missing_signal() {
    error(
        "[proc1]\nspi0={sck=11,mosi=10}",
        &["E5", "proc1.spi0 needs sck, mosi, miso"],
    );
    error("[proc0]\nuart2={tx=4}", &["E5", "needs tx, rx"]);
    error("[proc0]\ni2c1={sda=2}", &["E5", "needs sda, scl"]);
}
#[test]
fn gpio_irq_not_gpio() {
    error(
        "[proc1]\ngpio_irq=[27]",
        &["E5", "gpio_irq 27 is not in proc1.gpio"],
    );
    error(
        "[proc0]\nuart2={tx=4,rx=5}\ngpio_irq=[4]",
        &["E5", "not in proc0.gpio"],
    );
}
#[test]
fn gpio_irq_pin_twice() {
    error(
        "[proc1]\ngpio=[27]\ngpio_irq=[27,27]",
        &[
            "E1",
            "GPIO 27 is claimed by both proc1.gpio_irq and proc1.gpio_irq",
        ],
    );
}
#[test]
fn doorbell_owns_its_pin() {
    let d = parse("[proc0]\ndoorbell=26\n[proc1]\ndoorbell=27");
    d.validate().unwrap();
    assert_eq!((d.proc0.doorbell, d.proc1.doorbell), (Some(26), Some(27)));
    error(
        "[proc1]\ngpio=[27]\ndoorbell=27",
        &[
            "E1",
            "GPIO 27 is claimed by both proc1.gpio and proc1.doorbell",
        ],
    );
    error(
        "[proc0]\ndoorbell=27\n[proc1]\ndoorbell=27",
        &[
            "E1",
            "GPIO 27 is claimed by both proc0.doorbell and proc1.doorbell",
        ],
    );
    error(
        "[proc1]\ndoorbell=27\ngpio_irq=[27]",
        &["E5", "gpio_irq 27 is not in proc1.gpio"],
    );
    error("[proc1]\ndoorbell=54", &["E2", "GPIO 54 is out of range"]);
}
#[test]
fn unknown_keys_and_hardware_cs_are_rejected() {
    error(
        "[proc1]\nuart9={tx=4,rx=5}",
        &["E0", "unknown key 'uart9' in proc1", "instances: uart0, "],
    );
    error(
        "[proc0]\nspi0={sck=11,mosi=10,miso=9,cs0=8}",
        &["E4", "hardware CS is unsupported"],
    );
    error(
        "[proc0]\nadc={pin=22}",
        &["E0", "proc0.adc must be an empty table"],
    );
    error("[proc0]\ngpio=22", &["E0", "proc0.gpio"]);
    error(
        "log={sink='uart1',baudrate=115200}",
        &["E0", "invalid UART0 log"],
    );
    for body in [
        "[proc0]\nuart2={tx=4,rx=5,typo=6}",
        "[proc0]\ni2c1={sda=2,scl=3,typo=4}",
        "[proc0]\nadc={pin=22}",
        "[linux]\non_conflict='ignore'",
        "[linux]\ngpiomem='sometimes'",
        "[linux]\nkeeep={}",
        "[proc0]\npins=[22]",
        "[proc0]\nperipherals=['ADC']",
    ] {
        error(body, &[]);
    }
}
#[test]
fn board_defaults_and_explicit_replacement() {
    let d = parse("");
    assert_eq!(d.linux.on_conflict, OnConflict::Disable);
    assert_eq!(d.linux.gpiomem, Gpiomem::Disable);
    assert_eq!(d.linux.keep, None);
    assert_eq!(d.linux.kept().len(), 1);
    assert!(d.warnings().is_empty());
    let with_linux = parse("[linux]\nuio_group='gpio'");
    assert_eq!(with_linux.linux.keep, d.linux.keep);
    let d =
        parse("[linux]\nkeep={}\non_conflict='refuse'\ngpiomem='keep'\n[proc0]\ngpio=[0,1,2,3]");
    d.validate().unwrap();
    assert_eq!(d.linux.on_conflict, OnConflict::Refuse);
    assert_eq!(d.linux.gpiomem, Gpiomem::Keep);
    assert!(d.warnings().iter().any(|s| s.contains("GPIO 0/1")));
    let d = parse("[linux]\nkeep={}\n[proc0]\nuart1={tx=0,rx=1}");
    d.validate().unwrap();
    let d = parse("[linux]\nkeep={i2c1={sda=2,scl=3}}\n[proc0]\ngpio=[0,1]");
    d.validate().unwrap();
    assert!(!d.linux.kept().contains_key("uart1"));
}

#[test]
fn optional_signals_and_linux_keep_are_validated() {
    // Review 12 (Claude N1): RP1 has no flow-control driver, so CTS/RTS cannot be declared.
    for key in ["cts=6", "rts=7", "cts=7"] {
        error(
            &format!("[proc0]\nuart2={{tx=4,rx=5,{key}}}"),
            &[
                "E4",
                &format!("proc0.uart2.{}", &key[..3]),
                "UART flow control is not supported yet",
            ],
        );
    }
    // Linux keeps its own flow-control pins; they are still checked against the pin table.
    parse("[linux]\nkeep={uart1={tx=0,rx=1,cts=2,rts=3}}")
        .validate()
        .unwrap();
    error(
        "[linux]\nkeep={uart2={tx=4,rx=5,cts=7}}",
        &["E2", "linux.keep.uart2.cts = 7", "UART2_RTS", "UART2_CTS"],
    );
    error(
        "[linux]\nkeep={i2c1={sda=2}}",
        &["E5", "linux.keep.i2c1 needs sda, scl"],
    );
    error(
        "[linux]\nkeep={uart1={tx=1,rx=0}}",
        &["E2", "linux.keep.uart1.tx"],
    );
}

#[test]
fn public_model_edits_cannot_bypass_instance_validation() {
    let mut d = parse("[proc0]\nuart2={tx=4,rx=5}");
    let uart = d.proc0.instances.remove("uart2").unwrap();
    d.proc0.instances.insert("i2c1".into(), uart);
    assert!(d.validate().is_err());
    d.proc0.instances.clear();
    d.proc0.instances.insert("uart9".into(), Instance::Adc {});
    assert!(d.validate().is_err());
}
#[test]
fn schema2_typed_instances_and_roundtrip() {
    let d = parse(
        "log={sink='uart0',baudrate=115200}\n[proc0]\ngpio=[22]\ntimer=['ALARM0']\ni2c1={sda=2,scl=3}\n[proc1]\ngpio=[8,27]\ngpio_irq=[27]\ntimer=['ALARM1']\nadc={}\nspi0={sck=11,mosi=10,miso=9}\nuart2={tx=4,rx=5}",
    );
    d.validate().unwrap();
    assert!(matches!(d.proc1.instances["uart2"], Instance::Uart(_)));
    assert_eq!(
        d.proc1.instances["uart2"].pins("uart2"),
        [("tx", 4), ("rx", 5)]
    );
    let serialized = toml::to_string(&d).unwrap();
    let roundtrip: Declaration = toml::from_str(&serialized).unwrap();
    roundtrip.validate().unwrap();
    assert_eq!(
        d.declaration_hash().unwrap(),
        roundtrip.declaration_hash().unwrap()
    );
}

#[test]
fn canonical_claim_order() {
    let d = parse(
        "[proc0]\ngpio=[22,7,6]\ngpio_irq=[22]\ntimer=['ALARM1','ALARM0']\n\
         uart2={tx=4,rx=5}\ni2c1={sda=10,scl=11}\nadc={}",
    );
    d.validate().unwrap();
    assert_eq!(d.proc0.gpio, [22, 7, 6]);
    assert_eq!(d.proc0.pins(), [4, 5, 6, 7, 10, 11, 22]);
    assert_eq!(
        d.proc0.peripherals(),
        ["TIMER.ALARM1", "TIMER.ALARM0", "ADC", "I2C1", "UART2"]
    );
}

#[test]
fn declaration_hash_tracks_edits_after_parsing() {
    let mut d = parse("[proc0]\ngpio=[22]");
    let hash = d.declaration_hash().unwrap();
    d.proc0.gpio.push(23);
    assert_ne!(d.declaration_hash().unwrap(), hash);
}

#[test]
fn linux_keep_conflicts_with_itself_and_log() {
    error(
        "[linux]\nkeep={uart1={tx=0,rx=1},i2c0={sda=0,scl=1}}",
        &["E1", "linux.keep."],
    );
    error(
        "log={sink='uart0',baudrate=115200}\n[linux]\nkeep={uart0={tx=14,rx=15}}",
        &["E3", "UART0", "log sink"],
    );
}
#[test]
fn repeated_gpio_and_alarm() {
    error("[proc0]\ngpio=[22,22]", &["E1", "GPIO 22"]);
    error(
        "[proc0]\ntimer=['ALARM0','ALARM0']",
        &["E3", "TIMER.ALARM0"],
    );
}

#[test]
fn all_supported_instances_accept_pin_table_routes() {
    use rp1_pinmux::{I2cSignal, Signal, SpiSignal, UartSignal, candidate_pins};
    let pin = |s| candidate_pins(s).next().unwrap();
    for n in 0..=4 {
        let d = parse(&format!(
            "[linux]\nkeep={{}}\n[proc0]\nuart{n}={{tx={},rx={}}}",
            pin(Signal::Uart(n, UartSignal::Tx)),
            pin(Signal::Uart(n, UartSignal::Rx)),
        ));
        d.validate().unwrap();
        let d = parse(&format!(
            "[proc0]\ngpio=[]\n[linux]\nkeep={{uart{n}={{tx={},rx={},cts={},rts={}}}}}",
            pin(Signal::Uart(n, UartSignal::Tx)),
            pin(Signal::Uart(n, UartSignal::Rx)),
            pin(Signal::Uart(n, UartSignal::Cts)),
            pin(Signal::Uart(n, UartSignal::Rts)),
        ));
        d.validate().unwrap();
    }
    for n in 0..=3 {
        parse(&format!(
            "[linux]\nkeep={{}}\n[proc0]\ni2c{n}={{sda={},scl={}}}",
            pin(Signal::I2c(n, I2cSignal::Sda)),
            pin(Signal::I2c(n, I2cSignal::Scl))
        ))
        .validate()
        .unwrap();
    }
    for n in [0, 1, 3, 5] {
        parse(&format!(
            "[proc0]\nspi{n}={{sck={},mosi={},miso={}}}",
            pin(Signal::Spi(n, SpiSignal::Sclk)),
            pin(Signal::Spi(n, SpiSignal::Mosi)),
            pin(Signal::Spi(n, SpiSignal::Miso))
        ))
        .validate()
        .unwrap();
    }
}

#[test]
fn instance_declarations_require_tables() {
    for body in [
        "[proc0]\nuart2=[4,5,6,7]",
        "[proc0]\ni2c1=[2,3]",
        "[proc0]\nspi0=[11,10,9]",
        "[linux]\nkeep={uart1=[0,1]}",
    ] {
        error(body, &["E0", "must be a pin table"]);
    }
}

#[test]
fn pwm_ownership_is_whole_block_even_with_disjoint_channel_pins() {
    error(
        "[proc0]\npwm0={channel0=12}\n[proc1]\npwm0={channel1=13}",
        &["E3", "PWM0", "proc0", "proc1"],
    );
    error(
        "[linux]\nkeep={pwm0={channel1=13}}\n[proc0]\npwm0={channel0=12}",
        &["E3", "PWM0", "Linux by linux.keep"],
    );
    error(
        "[proc0]\ngpio=[12]\npwm0={channel0=12}",
        &["E1", "GPIO 12", "pwm0.channel0"],
    );
    error(
        "log={sink='uart0',baudrate=115200}\n[proc0]\npwm0={channel2=14}",
        &["E1", "GPIO 14", "log.uart0.tx"],
    );
    error(
        "[proc0]\npwm0={channel0=13}",
        &["E2", "pwm0.channel0", "GPIO12"],
    );
    error("[proc0]\npwm1={channel0=12}", &["E2", "PWM1"]);
    error("[proc0]\npwm1={channel0=45}", &["E4", "Linux"]);
    error("[proc0]\npwm0={channel4=12}", &["E0", "channel4"]);
    let d =
        parse("[proc0]\npwm0={channel0=12,channel1=13,channel2=18,channel3=19}\n[proc1]\npwm1={}");
    d.validate().unwrap();
    assert_eq!(d.proc0.pins(), [12, 13, 18, 19]);
    assert_eq!(d.proc0.peripherals(), ["PWM0"]);
    assert_eq!(d.proc1.peripherals(), ["PWM1"]);
}

#[test]
fn dma_is_one_pinless_controller_owned_by_one_core() {
    error(
        "[proc0]\ndma={}\n[proc1]\ndma={}",
        &["E3", "DMA", "proc0", "proc1"],
    );
    error("[proc0]\ndma={channel=1}", &["E0", "proc0.dma"]);
    error("[proc0]\ndma=1", &["E0", "proc0.dma"]);
    for channels in ["[2,2]", "[8]", "[-1]"] {
        error(
            &format!("[proc1]\ndma={{channels={channels}}}"),
            &["E0", "channels"],
        );
    }
    error("[linux]\nkeep={dma={channels=[0]}}", &["E4", "whole DMA"]);
    error(
        "[linux]\nkeep={dma={}}\n[proc0]\ndma={channels=[0,2]}",
        &["E3", "DMA", "linux.keep"],
    );
    for channels in ["[]", "[2,0]"] {
        parse(&format!("[proc1]\ndma={{channels={channels}}}"))
            .validate()
            .unwrap();
    }
    let d = parse("[proc1]\ndma={}");
    d.validate().unwrap();
    assert!(d.proc1.pins().is_empty());
    assert_eq!(d.proc1.peripherals(), ["DMA"]);
    // No longer a board Linux block: the declaration may own it.
    assert!(
        !rp1_proto::board::LINUX_BLOCKS
            .iter()
            .any(|(b, _)| *b == "DMA")
    );
}

#[test]
fn dma_pool_requires_aligned_nonempty_slots_and_fits_shared_sram() {
    for (slots, bytes) in [(0, 16), (2, 0), (2, 17), (4, 16384), (u32::MAX, u32::MAX)] {
        error(
            &format!("[proc1]\ndma={{pool={{slots={slots},bytes={bytes}}}}}"),
            &["E0", "proc1.dma.pool"],
        );
    }
    error(
        "[linux]\nkeep={dma={pool={slots=2,bytes=16}}}",
        &["E4", "whole DMA"],
    );
    let d = parse("[proc1]\ndma={channels=[0,2],pool={slots=2,bytes=4096}}");
    d.validate().unwrap();
}

#[test]
fn heartbeat_period() {
    let d = parse("[proc0]\nheartbeat={period_ms=100}");
    d.validate().unwrap();
    assert_eq!(d.proc0.heartbeat.map(|h| h.period_ms), Some(100));
    assert_eq!(d.proc1.heartbeat, None);
    error(
        "[proc0]\nheartbeat={period_ms=0}",
        &["E0", "proc0.heartbeat.period_ms must be at least 1"],
    );
    error(
        "[proc1]\nheartbeat={period=100}",
        &["E0", "proc1.heartbeat"],
    );
    error(
        "[proc1]\nheartbeat={period_ms=70000}",
        &["E0", "proc1.heartbeat"],
    );
}

#[test]
fn proc0_stack_always_holds_the_platform_margin() {
    use rp1_build::metadata::{DEFAULT_STACK, PLATFORM_STACK_MARGIN};
    let margin = rp1_build::metadata::Declaration::stack_margin;
    assert_eq!((margin(0), margin(1)), (PLATFORM_STACK_MARGIN, 0));
    let d = parse("");
    assert_eq!(
        d.stack_size(0).unwrap(),
        DEFAULT_STACK + PLATFORM_STACK_MARGIN
    );
    assert_eq!(d.stack_size(1).unwrap(), DEFAULT_STACK);
    // Declaring one core keeps the other's default.
    let d = parse("stack = { proc1 = '6KiB' }");
    assert_eq!(
        d.stack_size(0).unwrap(),
        DEFAULT_STACK + PLATFORM_STACK_MARGIN
    );
    assert_eq!(d.stack_size(1).unwrap(), 6144);
    // A declared proc0 stack is the user's part; the margin comes on top.
    let d = parse("stack = { proc0 = '1KiB' }");
    assert_eq!(d.stack_size(0).unwrap(), 1024 + PLATFORM_STACK_MARGIN);
    // The margin counts against the 8 KiB limit on proc0 only.
    assert!(parse("stack = { proc0 = '7KiB' }").stack_size(0).is_ok());
    assert!(parse("stack = { proc0 = '8KiB' }").stack_size(0).is_err());
    assert_eq!(
        parse("stack = { proc1 = '8KiB' }").stack_size(1).unwrap(),
        8192
    );
}

#[test]
fn pio_ownership_rejections() {
    for (body, expected) in [
        (
            "[proc0]\npio={sms=[0]}\n[proc1]\npio={sms=[1]}",
            "declared by both proc0 and proc1",
        ),
        ("[proc1]\npio={sms=[0,0]}", "state machine 0 is repeated"),
        (
            "[proc1]\npio={sms=[4]}",
            "state machine 4 is repeated or outside",
        ),
        ("[proc1]\npio={sms=[]}", "at least one state machine"),
        ("[proc1]\npio={}", "missing field"),
        ("[proc1]\npio={sms=[0],pins=[28]}", "GPIO 28 is board line"),
        (
            "[proc1]\npio={sms=[0],pins=[54]}",
            "GPIO 54 is out of range",
        ),
        (
            "[proc1]\npio={sms=[0],pins=[12,12]}",
            "GPIO 12 is claimed by both",
        ),
        (
            "[proc0]\ngpio=[12]\n[proc1]\npio={sms=[0],pins=[12]}",
            "GPIO 12 is claimed by both",
        ),
        (
            "clock={authority='linux'}\n[proc1]\npio={sms=[0]}",
            "PIO requires clock.authority = rp1",
        ),
        (
            "[linux]\nkeep={pio={sms=[0]}}",
            "reserved by the stock profile",
        ),
        (
            "[linux]\ngpiomem='keep'\n[proc1]\npio={sms=[0]}",
            "PIO requires linux.gpiomem = disable",
        ),
        ("[proc1]\npio={sms=[0],dma=true}", "unknown field"),
    ] {
        error(body, &[expected]);
    }
}
