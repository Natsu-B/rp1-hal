//! Host tests of the shared-memory protocol, one file per part of `shm`.
extern crate std;

mod block;
pub(super) mod model;
mod notify;
mod overwrite;
mod ring;
#[cfg(feature = "unstable-ring-dma")]
mod staged;
mod wait;

#[repr(C, align(8))]
struct Mem<const N: usize>([u8; N]);

fn mem<const N: usize>() -> std::boxed::Box<Mem<N>> {
    std::boxed::Box::new(Mem([0; N]))
}
