@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x == 0u {
        var sum: f32 = 0.0;
        for (var i = 0u; i < 1000000u; i = i + 1u) {
            sum = sum + input[i] * input[i];
        }
        output[0] = sum;
    }
}