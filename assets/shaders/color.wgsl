struct VertexInput {
    @location(0) position : vec3<f32>,
    @location(1) tex_coords : vec2<f32>,
};

struct ComponentUniform {
    color : vec4<f32>,
    position : vec2<f32>,       // Position in pixels (top-left corner)
    size : vec2<f32>,           // Size in pixels (width, height)
    border_radius : vec4<f32>,  // Corner radii in pixels (top-left, top-right, bottom-left, bottom-right)
    viewport_size : vec2<f32>,  // Viewport dimensions in pixels
    use_texture : u32,          // Flag: 0 for color, 1 for texture, 2 for frosted glass
    blur_radius: f32,           // Blur intensity for frosted glass
    opacity: f32,               // Overall opacity for frosted glass
}

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

// Improved Gaussian blur with better quality
fn sample_blur(tex: texture_2d<f32>, samp: sampler, uv: vec2<f32>, blur_amount: f32) -> vec4<f32> {
    if (blur_amount <= 0.0) {
        return textureSample(tex, samp, uv);
    }

    // Get texture dimensions for proper scaling of the blur
    let tex_size = vec2<f32>(textureDimensions(tex));
    let pixel_size = 1.0 / tex_size;
    
    // Scale blur based on a percentage of screen size for consistent results
    // Use higher blur intensity values (compared to previous version)
    let sigma = blur_amount * 0.05;
    
    var color = vec4<f32>(0.0);
    var total_weight = 0.0;
    
    // Increase sample count for higher quality blur
    let max_samples = 31; // Must be odd number
    let half_samples = max_samples / 2;
    
    // Use a two-pass separable Gaussian blur for better performance and quality
    // First horizontal pass
    var horizontal = vec4<f32>(0.0);
    var horizontal_weight = 0.0;
    
    for (var i = -half_samples; i <= half_samples; i++) {
        let offset = vec2<f32>(f32(i), 0.0) * pixel_size * sigma * 2.0;
        let sample_pos = uv + offset;
        
        // Use exponential falloff for weight calculation
        let weight = exp(-(f32(i*i) / (2.0 * sigma * sigma * 30.0)));
        horizontal += textureSample(tex, samp, sample_pos) * weight;
        horizontal_weight += weight;
    }
    horizontal /= horizontal_weight;
    
    // Second vertical pass
    for (var j = -half_samples; j <= half_samples; j++) {
        let offset = vec2<f32>(0.0, f32(j)) * pixel_size * sigma * 2.0;
        
        // Sample from horizontal result conceptually (we simulate this)
        let sample_pos = uv + offset;
        
        // Use exponential falloff for weight calculation
        let weight = exp(-(f32(j*j) / (2.0 * sigma * sigma * 30.0)));
        
        // Sample the source texture with both horizontal and vertical offsets
        var sample_color = vec4<f32>(0.0);
        var sample_weight = 0.0;
        
        for (var i = -half_samples; i <= half_samples; i++) {
            let h_offset = vec2<f32>(f32(i), 0.0) * pixel_size * sigma * 2.0;
            let h_weight = exp(-(f32(i*i) / (2.0 * sigma * sigma * 30.0)));
            sample_color += textureSample(tex, samp, sample_pos + h_offset) * h_weight;
            sample_weight += h_weight;
        }
        sample_color /= sample_weight;
        
        color += sample_color * weight;
        total_weight += weight;
    }
    
    return color / total_weight;
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
    // This is the critical part that handles clipping for all rendering modes
    if (pixel_coords.x < rect_min.x || pixel_coords.x > rect_max.x || 
        pixel_coords.y < rect_min.y || pixel_coords.y > rect_max.y) {
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
    
    // Calculate texture coordinates based on component type
    var tex_coords: vec2<f32>;

    if (component.use_texture == 2u) {
        // For frosted glass, calculate screen-space UV coordinates
        // Fix: Maintain consistent Y orientation between capture and sampling
        tex_coords = vec2<f32>(
            pixel_coords.x / component.viewport_size.x,
            pixel_coords.y / component.viewport_size.y  // Don't flip Y for frosted glass
        );
    } else {
        // For regular textures, use the normalized coordinates within the component
        tex_coords = vec2<f32>(
            (pixel_coords.x - rect_min.x) / component.size.x,
            (pixel_coords.y - rect_min.y) / component.size.y
        );
    }

    var final_color: vec4<f32>;

    if (component.use_texture == 1u) {
        // Regular texture mode
        final_color = textureSample(t_diffuse, s_diffuse, tex_coords);
    } 
    else if (component.use_texture == 2u) {
        // 1. Sample the background with blur
        var blurAmount = component.blur_radius * 1.2;
        var background = sample_blur(t_diffuse, s_diffuse, tex_coords, blurAmount);
        
        // 2. Apply tint with the component color
        var tinted = mix(
            background.rgb, 
            component.color.rgb, 
            component.color.a * 0.4 * component.opacity
        );
        
        // 3. Improved edge highlight for a cleaner glass-like effect
        // Remove the inset shadow look by using a more subtle highlight approach
        let local_tex_coords = vec2<f32>(
            (pixel_coords.x - rect_min.x) / component.size.x,
            (pixel_coords.y - rect_min.y) / component.size.y
        );
        
        // Create a subtle edge glow that doesn't look like an inset shadow
        let edge_distance_x = min(local_tex_coords.x, 1.0 - local_tex_coords.x) * 2.0;
        let edge_distance_y = min(local_tex_coords.y, 1.0 - local_tex_coords.y) * 2.0;
        let edge_distance = min(edge_distance_x, edge_distance_y);
        
        // Apply a very subtle, smooth highlight that increases slightly near edges
        // but doesn't create an obvious inset shadow effect
        let edge_highlight = smoothstep(0.0, 0.5, edge_distance) * 0.05;
        tinted += vec3<f32>(edge_highlight);
        
        // 4. Apply final opacity with slight gamma correction for more natural look
        final_color = vec4<f32>(pow(tinted, vec3<f32>(0.98)), component.opacity);
    }
    else {
        // Plain color mode
        final_color = in.color;
    }
    
    // Apply anti-aliasing at corner edges
    if (in_corner) {
        var aa_factor = smoothstep(corner_radius, corner_radius - aa_width, corner_dist);
        final_color.a *= aa_factor;
    }
    // Apply subtle anti-aliasing on straight edges as well
    else {
        // Find distance to nearest edge
        var edge_dist = min(min(dist_to_left, dist_to_right), min(dist_to_top, dist_to_bottom));
        
        // Only apply near edges (within aa_width pixels)
        if (edge_dist < aa_width) {
            final_color.a *= edge_dist / aa_width;
        }
    }
    
    return final_color;
}