//! Collect kernel diagnostics emitted during this invocation through the userspace kmsg ABI.
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom},
    os::unix::fs::OpenOptionsExt,
};

pub fn start() -> io::Result<File> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open("/dev/kmsg")?;
    file.seek(SeekFrom::End(0))?;
    Ok(file)
}

#[derive(Default)]
struct Errors {
    scmi_timeouts: usize,
    oops: usize,
    link_errors: usize,
}

impl Errors {
    fn observe(&mut self, message: &str) -> bool {
        let message = message.to_ascii_lowercase();
        let timeout = message
            .split_once("scmi")
            .is_some_and(|(_, suffix)| suffix.contains("timed out"));
        let oops = ["oops", "internal error", "kernel panic"]
            .iter()
            .any(|s| message.contains(s));
        let link = ["bar 0 not claimed", "link down", "link is down"]
            .iter()
            .any(|s| message.contains(s));
        self.scmi_timeouts += usize::from(timeout);
        self.oops += usize::from(oops);
        self.link_errors += usize::from(link);
        timeout || oops || link
    }
}

fn collect(tag: &str, mut file: File) -> io::Result<Errors> {
    let mut errors = Errors::default();
    let mut buffer = [0; 8192];
    loop {
        match file.read(&mut buffer) {
            Ok(n) if n != 0 => {
                let record = String::from_utf8_lossy(&buffer[..n]);
                let (_, message) = record.split_once(';').ok_or(io::ErrorKind::InvalidData)?;
                let message = message.lines().next().unwrap_or("");
                if errors.observe(message) {
                    println!("FAULT_KERNEL {tag} {message}");
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(errors),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
            Ok(_) => return Err(io::ErrorKind::UnexpectedEof.into()),
        }
    }
}

pub fn report(tag: &str, file: io::Result<File>) {
    match file.and_then(|file| collect(tag, file)) {
        Ok(Errors {
            scmi_timeouts,
            oops,
            link_errors,
        }) => println!(
            "FAULT_DMESG {tag} scmi_timeouts={scmi_timeouts} oops={oops} link_errors={link_errors}"
        ),
        Err(error) => println!("FAULT_ERROR {tag} step=kmsg error={error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinguishes_boot_banner_from_runtime_errors() {
        let mut errors = Errors::default();
        assert!(!errors.observe("arm-scmi: SCMI max-rx-timeout: 30ms"));
        assert!(errors.observe("arm-scmi: message timed out"));
        assert!(errors.observe("Internal error: Oops: 00000000"));
        assert!(errors.observe("brcm-pcie: link is down"));
        assert_eq!(
            (errors.scmi_timeouts, errors.oops, errors.link_errors),
            (1, 1, 1)
        );
    }
}
