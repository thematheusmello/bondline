//! Native desktop entry point.

#[derive(Default)]
struct LaunchOptions {
    smoke: bool,
    control_port: Option<u16>,
}

fn parse_args(arguments: impl IntoIterator<Item = String>) -> std::io::Result<LaunchOptions> {
    let mut options = LaunchOptions::default();
    let mut arguments = arguments.into_iter();
    let invalid = || {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "usage: bondline [--smoke-test] [--control-port PORT]; PORT is 0..65535",
        )
    };
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--smoke-test" if !options.smoke => options.smoke = true,
            "--control-port" if options.control_port.is_none() => {
                options.control_port = Some(
                    arguments
                        .next()
                        .and_then(|port| port.parse().ok())
                        .ok_or_else(invalid)?,
                );
            }
            _ => return Err(invalid()),
        }
    }
    Ok(options)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_args(std::env::args().skip(1))?;
    eframe::run_native(
        "Bondline",
        bondline_ui::native_options()?,
        Box::new(move |cc| {
            Ok(Box::new(bondline_ui::BondlineApp::new(
                cc,
                options.smoke,
                options.control_port,
            )?))
        }),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn launch_flags_are_explicit_and_validated_before_opening_a_window()
    -> Result<(), Box<dyn std::error::Error>> {
        let options = super::parse_args(
            ["--smoke-test", "--control-port", "0"]
                .into_iter()
                .map(String::from),
        )?;
        assert!(options.smoke);
        assert_eq!(options.control_port, Some(0));
        let options = super::parse_args(std::iter::empty())?;
        assert!(!options.smoke);
        assert_eq!(options.control_port, None);
        for flags in [
            vec!["--control-port"],
            vec!["--control-port", "65536"],
            vec!["--control-port", "-1"],
            vec!["--smoke-test", "--smoke-test"],
            vec!["--unknown"],
            vec!["--control-port", "0", "--control-port", "0"],
        ] {
            assert!(super::parse_args(flags.into_iter().map(String::from)).is_err());
        }
        Ok(())
    }
}
