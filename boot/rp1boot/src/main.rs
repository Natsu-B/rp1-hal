//! rp1boot: BCM2712 third-stage loader (docs/loader.md). Built only for
//! `aarch64-unknown-none-softfloat`; on the host this file is an empty program so the
//! workspace builds and the library tests run.
#![cfg_attr(target_os = "none", no_std, no_main)]

#[cfg(not(target_os = "none"))]
fn main() {}

#[cfg(target_os = "none")]
extern crate alloc;
#[cfg(target_os = "none")]
mod hw;
