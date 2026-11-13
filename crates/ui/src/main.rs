//! Native desktop entry point.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let smoke = match args.as_slice() {
        [] => false,
        [argument] if argument == "--smoke-test" => true,
        _ => return Err("usage: bondline [--smoke-test]".into()),
    };
    eframe::run_native(
        "Bondline",
        bondline_ui::native_options()?,
        Box::new(move |cc| Ok(Box::new(bondline_ui::BondlineApp::new(cc, smoke)?))),
    )?;
    Ok(())
}
