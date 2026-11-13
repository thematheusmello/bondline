struct Primitive {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    begin_x: f32,
    begin_y: f32,
    direction_x: f32,
    direction_y: f32,
    length: f32,
    stroke_width: f32,
    color: u32,
    kind: u32,
};

struct Scene {
    width: u32,
    height: u32,
    primitives: array<Primitive>,
};

@group(0) @binding(0) var<storage, read> scene: Scene;

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
        let offset = input.position.xy - vec2(primitive.begin_x, primitive.begin_y);
        let direction = vec2(primitive.direction_x, primitive.direction_y);
        let along = dot(offset, direction);
        let across = offset.x * direction.y - offset.y * direction.x;
        if along < 0.0 || along > primitive.length || abs(across) > primitive.stroke_width * 0.5 { discard; }
    }
    return unpack4x8unorm(primitive.color);
}
