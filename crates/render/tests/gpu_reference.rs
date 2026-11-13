//! Independent expected pixels and CPU/GPU comparison, without any GUI dependency.

#[test]
fn native_gpu_clear_matches_cpu_and_independent_rgba_reference()
-> Result<(), Box<dyn std::error::Error>> {
    let (gpu, info) = pollster::block_on(bondline_render::gpu_solid(2, 2, [12, 34, 56, 255]))?;
    println!(
        "GPU reference: {} {:?} {:?}",
        info.name, info.backend, info.device_type
    );
    assert_eq!(gpu.rgba, [12, 34, 56, 255].repeat(4));
    assert_eq!(gpu, bondline_render::cpu_solid(2, 2, [12, 34, 56, 255])?);
    assert!(pollster::block_on(bondline_render::gpu_solid(0, 1, [0; 4])).is_err());
    Ok(())
}
