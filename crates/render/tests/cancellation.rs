use bondline_render::{Cancellation, Error, Scene, cpu_scene_with_cancel, gpu_scene_with_cancel};

#[test]
fn cancelled_render_is_rejected_before_allocating_a_frame_or_opening_a_gpu() {
    let cancelled = Cancellation::default();
    let clone = cancelled.clone();
    cancelled.cancel();
    assert!(clone.is_cancelled());
    let scene = Scene {
        width: 2048,
        height: 2048,
        background: [255; 4],
        primitives: vec![],
    };
    assert!(matches!(
        cpu_scene_with_cancel(&scene, &clone),
        Err(Error::Cancelled)
    ));
    assert!(matches!(
        pollster::block_on(gpu_scene_with_cancel(&scene, &clone)),
        Err(Error::Cancelled)
    ));
}
