#[rp1_rt::interrupt(proc0)]
fn UNKNOWN_IRQ() {}
fn main() {}

mod rp1 {
    pub mod __irq_proc0 {
        pub trait Handler {}
        #[allow(non_camel_case_types)]
        pub enum UNKNOWN_IRQ {}
    }
}
