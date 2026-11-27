#[rp1_rt::interrupt(proc0)]
fn UNKNOWN_IRQ() {}
fn main() {}

mod rp1 {
    pub mod __irq_proc0 {
        pub trait Handler {} pub trait Declared {}
        #[allow(non_camel_case_types)]
        pub enum UNKNOWN_IRQ {} impl Declared for UNKNOWN_IRQ {}
    }
}
