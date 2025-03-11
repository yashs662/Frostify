struct VertexInput {
    @location(0) position : vec3<f32>,
    @location(1) tex_coords : vec2<f32>,
};

struct ComponentUniform {
    color : vec4<f32>,
    position : vec2<f32>,    // Position in pixels (top-left corner)
    size : vec2<f32>,        // Size in pixels (width, height)
    border_radius : vec4<f32>, // Corner radii in pixels (top-left, top-right, bottom-left, bottom-right)
    viewport_size : vec2<f32>, // Viewport dimensions in pixels
    use_texture : u32,         // Flag: 1 if using texture, 0 if using color
};

@group(0) @binding(0)
var<uniform> component : ComponentUniform;

// Texture bindings (optional)
@group(0) @binding(1)
var t_diffuse : texture_2d<f32>;
@group(0) @binding(2)
var s_diffuse : sampler;

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) color : vec4<f32>,
    @location(1) uv : vec2<f32>, // Screen-space UV coordinates
};

// Vertex shader that creates a full-screen triangle
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
    // Convert UV to pixel coordinates (flip Y to match typical top-left origin)
    var pixel_coords = vec2<f32>(
        in.uv.x * component.viewport_size.x,
        (1.0 - in.uv.y) * component.viewport_size.y
    );
    
    // Rectangle in pixel coordinates
    var rect_min = component.position;
    var rect_max = component.position + component.size;
    
    // Edge distances
    var dist_to_left = pixel_coords.x - rect_min.x;
    var dist_to_right = rect_max.x - pixel_coords.x;
    var dist_to_top = pixel_coords.y - rect_min.y;
    var dist_to_bottom = rect_max.y - pixel_coords.y;
    
    // Check if we're outside the main rectangle bounds
    if (dist_to_left < 0.0 || dist_to_right < 0.0 || 
        dist_to_top < 0.0 || dist_to_bottom < 0.0) {
        discard;
    }
    
    // Limit corner radii to avoid overlap
    var max_radius_x = min(component.size.x * 0.5, 10000.0);
    var max_radius_y = min(component.size.y * 0.5, 10000.0);
    var max_radius = min(max_radius_x, max_radius_y);
    
    // Clamp all radii
    var tl_radius = min(component.border_radius.x, max_radius);
    var tr_radius = min(component.border_radius.y, max_radius);
    var bl_radius = min(component.border_radius.z, max_radius);
    var br_radius = min(component.border_radius.w, max_radius);
    
    // Calculate corner centers
    var tl_center = vec2<f32>(rect_min.x + tl_radius, rect_min.y + tl_radius);
    var tr_center = vec2<f32>(rect_max.x - tr_radius, rect_min.y + tr_radius);
    var bl_center = vec2<f32>(rect_min.x + bl_radius, rect_max.y - bl_radius);
    var br_center = vec2<f32>(rect_max.x - br_radius, rect_max.y - br_radius);
    
    // Check if in corner regions and calculate distance
    var in_corner = false;
    var corner_dist: f32 = 0.0;
    var corner_radius: f32 = 0.0;
    
    // Top-left corner
    if (pixel_coords.x <= tl_center.x && pixel_coords.y <= tl_center.y && tl_radius > 0.0) {
        corner_dist = distance(pixel_coords, tl_center);
        corner_radius = tl_radius;
        in_corner = true;
    }
    // Top-right corner
    else if (pixel_coords.x >= tr_center.x && pixel_coords.y <= tr_center.y && tr_radius > 0.0) {
        corner_dist = distance(pixel_coords, tr_center);
        corner_radius = tr_radius;
        in_corner = true;
    }
    // Bottom-left corner
    else if (pixel_coords.x <= bl_center.x && pixel_coords.y >= bl_center.y && bl_radius > 0.0) {
        corner_dist = distance(pixel_coords, bl_center);
        corner_radius = bl_radius;
        in_corner = true;
    }
    // Bottom-right corner
    else if (pixel_coords.x >= br_center.x && pixel_coords.y >= br_center.y && br_radius > 0.0) {
        corner_dist = distance(pixel_coords, br_center);
        corner_radius = br_radius;
        in_corner = true;
    }
    
    // If in a corner and outside the radius, discard
    if (in_corner && corner_dist > corner_radius) {
        discard;
    }
    
    // Anti-aliasing width for both corners and edges
    var aa_width = 1.5;
    
    // Calculate texture coordinates if needed
    var tex_color = in.color;
    if (component.use_texture == 1u) {
        // Map pixel to texture coordinates
        var tex_coords = vec2<f32>(
            (pixel_coords.x - rect_min.x) / component.size.x,
            (pixel_coords.y - rect_min.y) / component.size.y
        );
        tex_color = textureSample(t_diffuse, s_diffuse, tex_coords);
    }
    
    // Start with texture/color alpha
    var alpha = tex_color.a;
    
    // Apply anti-aliasing at corner edges
    if (in_corner) {
        var aa_factor = smoothstep(corner_radius, corner_radius - aa_width, corner_dist);
        alpha *= aa_factor;
    }
    // Apply subtle anti-aliasing on straight edges as well
    else {
        // Find distance to nearest edge
        var edge_dist = min(min(dist_to_left, dist_to_right), min(dist_to_top, dist_to_bottom));
        
        // Only apply near edges (within aa_width pixels)
        if (edge_dist < aa_width) {
            alpha *= edge_dist / aa_width;
        }
    }
    
    return vec4<f32>(tex_color.rgb, alpha);
}