struct Primitive {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: u32,
    kind: u32,
    row_offset: u32,
};

struct Scene {
    width: u32,
    height: u32,
    primitives: array<Primitive>,
};

@group(0) @binding(0) var<storage, read> scene: Scene;
@group(0) @binding(1) var<storage, read> stroke_rows: array<vec2<u32>>;

struct Vertex {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) instance: u32,
};

@vertex fn vertex(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance: u32) -> Vertex {
    var corners = array<vec2<f32>, 6>(
        vec2(0.0,0.0), vec2(1.0,0.0), vec2(0.0,1.0),
        vec2(0.0,1.0), vec2(1.0,0.0), vec2(1.0,1.0)
    );
    let primitive = scene.primitives[instance];
    let pixel = vec2(f32(primitive.x), f32(primitive.y)) + corners[vertex_index] * vec2(f32(primitive.width), f32(primitive.height));
    var output: Vertex;
    output.position = vec4(pixel.x / f32(scene.width) * 2.0 - 1.0, 1.0 - pixel.y / f32(scene.height) * 2.0, 0.0, 1.0);
    output.instance = instance;
    return output;
}

@fragment fn fragment(input: Vertex) -> @location(0) vec4<f32> {
    let primitive = scene.primitives[input.instance];
    if primitive.kind == 1u {
        // Host prepares bounded analytic row geometry, never a finished CPU image.
        // Integer coverage preserves the shared closed-cap predicate without f32 ties.
        let pixel = vec2<u32>(input.position.xy);
        if pixel.y < primitive.y || pixel.y >= primitive.y + primitive.height { discard; }
        let range = stroke_rows[primitive.row_offset + pixel.y - primitive.y];
        if pixel.x < range.x || pixel.x >= range.y { discard; }
    }
    return unpack4x8unorm(primitive.color);
}
