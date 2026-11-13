//! Repository quality gates.

mod corpus;
mod layers;
mod licenses;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let operation = args.next();
    match operation.as_deref() {
        Some("licenses") if args.next().is_none() => licenses::check(),
        Some("layers") if args.next().is_none() => layers::check(),
        Some("corpus") => {
            let offline = match args.next().as_deref() {
                None => false,
                Some("--offline") if args.next().is_none() => true,
                _ => return Err("usage: cargo xtask corpus [--offline]".into()),
            };
            corpus::check(offline)
        }
        _ => Err("usage: cargo xtask <licenses|layers|corpus [--offline]>".into()),
    }
}
