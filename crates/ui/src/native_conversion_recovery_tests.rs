use super::*;

#[test]
fn successful_explicit_retry_clears_the_previous_conversion_error_banner() -> TestResult {
    let mut fixture = Fixture::new(1160.0, 800.0, 1.0)?;
    let current = dirty_native_drawing(&mut fixture)?;
    let current_bytes = fs::read(&current)?;
    let before = fixture.state()?;
    let input = fixture.directory.path("recovery-input-λ.smi");
    let existing = fixture.directory.path("existing-recovery-λ.cdxml");
    let output = fixture.directory.path("recovered-output-λ.cdxml");
    fs::write(&input, b"CC\n")?;
    fs::write(&existing, b"Keep this existing destination exactly.\n")?;
    open_form(&mut fixture)?;
    paths(&mut fixture, &input, &existing)?;
    fixture.choose("Convert")?;
    wait_for(&mut fixture, |fixture, shown| {
        has_text(shown, "Conversion failed")
            && !fixture.app.files.is_busy()
            && !fixture.app.executor.has_active_work()
    })?;
    assert_eq!(
        fs::read(&existing)?,
        b"Keep this existing destination exactly.\n"
    );
    assert_preserved(&fixture, &before)?;
    set_field(
        &mut fixture,
        "Output file",
        output.to_str().ok_or("output path")?,
    )?;
    fixture.choose("Convert")?;
    wait_for(&mut fixture, |fixture, shown| {
        has_text(shown, "Conversion complete")
            && output.exists()
            && !fixture.app.files.is_busy()
            && !fixture.app.executor.has_active_work()
    })?;
    actual_output(&output, false)?;
    assert_preserved(&fixture, &before)?;
    assert_eq!(fs::read(&input)?, b"CC\n");
    assert_eq!(fs::read(&current)?, current_bytes);
    assert!(
        fixture.app.error.is_none(),
        "A completed retry must not display the previous conversion as a current failure: {:?}",
        fixture.app.error
    );
    Ok(())
}
