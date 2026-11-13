struct Rectangle {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: u32,
};

struct Scene {
    width: u32,
    height: u32,
    rectangles: array<Rectangle>,
};

@group(0) @binding(0) var<storage, read> scene: Scene;

struct Vertex {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) color: vec4<f32>,
};

@vertex fn vertex(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance: u32) -> Vertex {
    var corners = array<vec2<f32>, 6>(
        vec2(0.0,0.0), vec2(1.0,0.0), vec2(0.0,1.0),
        vec2(0.0,1.0), vec2(1.0,0.0), vec2(1.0,1.0)
    );
    let rectangle = scene.rectangles[instance];
    let pixel = vec2(f32(rectangle.x), f32(rectangle.y)) + corners[vertex_index] * vec2(f32(rectangle.width), f32(rectangle.height));
    var output: Vertex;
    output.position = vec4(pixel.x / f32(scene.width) * 2.0 - 1.0, 1.0 - pixel.y / f32(scene.height) * 2.0, 0.0, 1.0);
    output.color = unpack4x8unorm(rectangle.color);
    return output;
}

@fragment fn fragment(input: Vertex) -> @location(0) vec4<f32> { return input.color; }
