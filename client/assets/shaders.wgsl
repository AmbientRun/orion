struct Camera {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
}

struct ObjectData {
    model: mat4x4<f32>,
    color: vec4<f32>,
}

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(0) @binding(1)
//var<uniform> object_data: array<ObjectData, 64>;
var<storage> object_data: array<ObjectData>;

@group(0) @binding(2)
var diffuse: texture_2d<f32>;
@group(0) @binding(3)
var diffuse_sampler: sampler;

@vertex
fn vs_main(
    in: VertexInput,
    @builtin(instance_index) instance: u32,
) -> VertexOutput {
    let object = object_data[instance];
    var out: VertexOutput;

    let mvp = camera.proj * camera.view * object.model;
    let clip_pos = mvp * vec4(in.pos, 1.0);

    out.clip_position = clip_pos;
    out.vert_pos = clip_pos.xyz;
    out.tex_coords = in.tex_coords;
    out.color = object.color;

    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color * textureSample(diffuse, diffuse_sampler, in.tex_coords);
    if color.a <= 0.001 {
        discard;
    }

    color = vec4(pow(color.rgb, vec3(1.0 / 2.2)), color.a);

    return color;
}
