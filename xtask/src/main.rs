//! Repository quality gates.

mod licenses;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match std::env::args().nth(1).as_deref() {
        Some("licenses") => licenses::check(),
        _ => Err("usage: cargo xtask licenses".into()),
    }
}
