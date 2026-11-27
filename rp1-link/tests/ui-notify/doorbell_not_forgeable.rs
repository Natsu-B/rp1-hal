// Safe code cannot forge a Doorbell.
use rp1_link::notify::Doorbell;
fn forged() -> Doorbell<27, 1> {
    Doorbell { _private: core::marker::PhantomData }
}
fn main() {}
