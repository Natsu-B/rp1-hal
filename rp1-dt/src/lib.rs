#![no_std]
//! Device-tree handling shared by the host tools and the BCM2712 loader (`rp1boot`):
//! an owned FDT tree, libfdt-compatible overlay application and the final-DTB checks.
extern crate alloc;

pub mod check;
pub mod fdt;
pub mod overlay;

pub type Result<T> = core::result::Result<T, alloc::string::String>;
