struct VertexInput {
    @location(0) position : vec3<f32>,
    @location(1) tex_coords : vec2<f32>,
};

struct ComponentUniform {
    color : vec4<f32>,
    location : vec2<f32>,
    size : vec2<f32>,
};

@group(0) @binding(0)
var<uniform> component : ComponentUniform;

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) color : vec4<f32>,
    @location(1) uv : vec2<f32>, // Screen-space UV coordinates
};

// Vertex shader that creates a full-screen quad
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Generate a full-screen triangle (efficiently covers the screen with 3 vertices)
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    var uv = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(2.0, 0.0),
        vec2<f32>(0.0, 2.0)
    );
    var out: VertexOutput;
    out.clip_position = vec4<f32>(pos[vertex_index], 0.0, 1.0);
    out.uv = uv[vertex_index];
    out.color = component.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert screen UV coordinates to NDC coordinates (-1 to 1)
    var ndc = in.uv * 2.0 - 1.0;
    var half_size = component.size * 0.5;
    var rectangle_center = vec2<f32>(
        component.location.x + half_size.x,
        component.location.y - half_size.y
    );
    
    // Coordinates relative to the rectangle center
    var rel_pos = abs(ndc - rectangle_center);
   
    // Rounded rectangle parameters
    let corner_radius = 0.01; // Should be getting this from the component in the future
    let smoothing = 0.002;    // Anti-aliasing parameter
   
    // Calculate SDF with rounded corners
    var x0 = rel_pos.x - half_size.x + corner_radius;
    var y0 = rel_pos.y - half_size.y + corner_radius;
   
    // Clamp to 0 for inside calculations
    var x1 = max(x0, 0.0);
    var y1 = max(y0, 0.0);
   
    // Calculate distance using exponent
    let exponent = 2.0;
   
    var d_pos = pow(pow(x1, exponent) + pow(y1, exponent), 1.0/exponent);
    var d_neg = min(max(x0, y0), 0.0);
    var sdf = d_pos + d_neg - corner_radius;
   
    // Apply anti-aliasing
    var alpha = 1.0 - smoothstep(-smoothing, smoothing, sdf);
   
    // Discard pixels outside the rectangle
    if (alpha < 0.001) {
        discard;
    }
   
    // Return color with adjusted alpha for edges
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}