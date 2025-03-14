// A simple shader that renders a texture onto a full-screen quad

// Vertex shader generates a full-screen triangle
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    // Generate a triangle that covers the whole screen
    let x = f32(vertex_index & 1u) * 2.0;
    let y = f32((vertex_index >> 1u) & 1u) * 2.0;
    
    // When using 6 vertices (0-5) for 2 triangles:
    // vertices 0,1,2 form first triangle, 2,1,3 would form second
    // But we're using 0,1,2 and 3,4,5 for simplicity
    return vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
}

// Texture binding for the main render texture
@group(0) @binding(0) var t_main: texture_2d<f32>;
@group(0) @binding(1) var s_main: sampler;

// Fragment shader samples from the texture
@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    // Convert from pixel coordinates to UV coordinates
    let uv = vec2<f32>(pos.x, pos.y) / vec2<f32>(textureDimensions(t_main));
    
    // Sample the texture and return the color
    return textureSample(t_main, s_main, uv);
}
