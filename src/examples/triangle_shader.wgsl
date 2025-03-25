struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>, // gl_Position in OpenGL
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}

//@vertex
//fn vs_main(@builtin(vertex_index) i : u32) -> @builtin(position) vec4f {
//   let positions = array<vec2f, 3>(
//        vec2f(0.0,  0.5),
//        vec2f(-0.5, -0.5),
//        vec2f(0.5, -0.5)
//   );
//   return vec4f(positions[i], 0.0, 1.0);
//}
//
//@fragment
//fn fs_main() -> @location(0) vec4f {
//    return vec4f(0.0, 0.15, 1.0, 1.0);
//}

//
//
//
//@vertex
//fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
//    return vec4<f32>(position, 0.0, 1.0);
//}
//
//@fragment
//fn fs_main() -> @location(0) vec4<f32> {
//    return vec4<f32>(0.0, 1.0, 0.0, 1.0); // Зелёный цвет
//}