// Safe code cannot make a Doorbell (only the core that declares `doorbell = 27` owns it).
use rp1_link::notify::Doorbell;
fn second() -> Doorbell<27, 1> {
    Doorbell::new()
}
fn main() {}
