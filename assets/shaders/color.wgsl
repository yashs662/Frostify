struct VertexInput {
    @location(0) position : vec3<f32>,
    @location(1) tex_coords : vec2<f32>,
};

struct ComponentUniform {
    color : vec4<f32>,
    position : vec2<f32>,       // Position in pixels (top-left corner)
    size : vec2<f32>,           // Size in pixels (width, height)
    border_radius : vec4<f32>,  // Corner radii in pixels (top-left, top-right, bottom-left, bottom-right)
    screen_size : vec2<f32>,    // Viewport dimensions in pixels
    use_texture : u32,          // Flag: 0 for color, 1 for texture, 2 for frosted glass
    blur_radius: f32,           // Blur intensity for frosted glass
    opacity: f32,               // Overall opacity for frosted glass
    _padding1: f32,             // Padding for alignment
    border_color: vec4<f32>,    // Border color
    border_width: f32,          // Border thickness in pixels
    border_position: u32,       // Border position: 0=inside, 1=center, 2=outside
    _padding2: vec2<f32>,       // Padding for alignment
    // Pre-computed values for optimization
    inner_bounds: vec4<f32>,    // (inner_min.x, inner_min.y, inner_max.x, inner_max.y)
    outer_bounds: vec4<f32>,    // (outer_min.x, outer_min.y, outer_max.x, outer_max.y)
    corner_centers: vec4<f32>,  // (tl_center.x, tl_center.y, tr_center.x, tr_center.y)
    corner_centers2: vec4<f32>, // (bl_center.x, bl_center.y, br_center.x, br_center.y)
    corner_radii: vec4<f32>,    // (inner_tl_radius, inner_tr_radius, inner_bl_radius, inner_br_radius)
    corner_radii2: vec4<f32>,   // (outer_tl_radius, outer_tr_radius, outer_bl_radius, outer_br_radius)
    shadow_color: vec4<f32>,     // Shadow color
    shadow_offset: vec2<f32>,    // Shadow offset
    shadow_blur: f32,            // Shadow blur intensity
    shadow_opacity: f32,         // Shadow opacity
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

// Optimized function to check if pixel is in a corner and get corner properties
fn check_corner(pixel_coords: vec2<f32>) -> vec4<f32> {
    // Pre-computed corner centers
    let tl_center = vec2<f32>(component.corner_centers.x, component.corner_centers.y);
    let tr_center = vec2<f32>(component.corner_centers.z, component.corner_centers.w);
    let bl_center = vec2<f32>(component.corner_centers2.x, component.corner_centers2.y);
    let br_center = vec2<f32>(component.corner_centers2.z, component.corner_centers2.w);
    
    // Pre-computed radii
    let inner_radii = component.corner_radii;
    let outer_radii = component.corner_radii2;
    
    // Use squared distance to avoid sqrt
    let tl_dist_sq = dot(pixel_coords - tl_center, pixel_coords - tl_center);
    let tr_dist_sq = dot(pixel_coords - tr_center, pixel_coords - tr_center);
    let bl_dist_sq = dot(pixel_coords - bl_center, pixel_coords - bl_center);
    let br_dist_sq = dot(pixel_coords - br_center, pixel_coords - br_center);
    
    // Check corners using squared distances
    if (pixel_coords.x <= tl_center.x && pixel_coords.y <= tl_center.y) {
        return vec4<f32>(1.0, tl_dist_sq, inner_radii.x, outer_radii.x);
    }
    if (pixel_coords.x >= tr_center.x && pixel_coords.y <= tr_center.y) {
        return vec4<f32>(1.0, tr_dist_sq, inner_radii.y, outer_radii.y);
    }
    if (pixel_coords.x <= bl_center.x && pixel_coords.y >= bl_center.y) {
        return vec4<f32>(1.0, bl_dist_sq, inner_radii.z, outer_radii.z);
    }
    if (pixel_coords.x >= br_center.x && pixel_coords.y >= br_center.y) {
        return vec4<f32>(1.0, br_dist_sq, inner_radii.w, outer_radii.w);
    }
    
    return vec4<f32>(0.0);
}

// Optimized function to check if we're in the border area
fn check_border(pixel_coords: vec2<f32>, corner_result: vec4<f32>) -> bool {
    if (component.border_width <= 0.0) {
        return false;
    }
    
    // Use pre-computed bounds
    let inner_min = vec2<f32>(component.inner_bounds.x, component.inner_bounds.y);
    let inner_max = vec2<f32>(component.inner_bounds.z, component.inner_bounds.w);
    let outer_min = vec2<f32>(component.outer_bounds.x, component.outer_bounds.y);
    let outer_max = vec2<f32>(component.outer_bounds.z, component.outer_bounds.w);
    
    // For corners, check if we're in the border ring using squared distances
    let in_corner = corner_result.x > 0.5;
    if (in_corner) {
        let corner_dist_sq = corner_result.y;
        let inner_radius_sq = corner_result.z * corner_result.z;
        let outer_radius_sq = corner_result.w * corner_result.w;
        return corner_dist_sq >= inner_radius_sq && corner_dist_sq <= outer_radius_sq;
    }
    
    // For straight edges, check if outside the inner content area but inside the outer bounds
    let outside_inner = pixel_coords.x <= inner_min.x || 
                       pixel_coords.x >= inner_max.x || 
                       pixel_coords.y <= inner_min.y || 
                       pixel_coords.y >= inner_max.y;
                       
    let inside_outer = pixel_coords.x >= outer_min.x &&
                      pixel_coords.x <= outer_max.x &&
                      pixel_coords.y >= outer_min.y &&
                      pixel_coords.y <= outer_max.y;
                      
    return outside_inner && inside_outer;
}

// Function to calculate texture coordinates based on pixel position
fn calculate_tex_coords(pixel_coords: vec2<f32>) -> vec2<f32> {
    let content_min = component.position;
    let content_max = component.position + component.size;
    
    if (component.use_texture == 2u) {
        // For frosted glass, use screen-space UV coordinates 
        return vec2<f32>(
            pixel_coords.x / component.screen_size.x,
            pixel_coords.y / component.screen_size.y  // Don't flip Y for frosted glass
        );
    } else {
        // For regular textures, use normalized coordinates within the component
        return vec2<f32>(
            (pixel_coords.x - content_min.x) / (content_max.x - content_min.x),
            (pixel_coords.y - content_min.y) / (content_max.y - content_min.y)
        );
    }
}

// Simple border color function without anti-aliasing
fn get_border_color(pixel_coords: vec2<f32>, in_corner: bool, corner_dist: f32, inner_radius: f32, outer_radius: f32) -> vec4<f32> {
    return component.border_color;
}

// Function to get content color (regular color, texture, or frosted glass)
fn get_content_color(pixel_coords: vec2<f32>, tex_coords: vec2<f32>, base_color: vec4<f32>) -> vec4<f32> {
    let content_min = component.position;
    let content_max = component.position + component.size;
    
    if (component.use_texture == 1u) {
        // Regular texture mode
        return textureSample(t_diffuse, s_diffuse, tex_coords);
    } 
    else if (component.use_texture == 2u) {
        // Improved frosted glass effect using high-quality Gaussian blur
        var blurAmount = component.blur_radius;
        var background = gaussian_blur(t_diffuse, s_diffuse, tex_coords, blurAmount);
        
        // Mix the background with the tint color using a more subtle approach
        var tinted = mix(
            background.rgb, 
            base_color.rgb, 
            base_color.a * 0.25 * component.opacity
        );
        
        // Apply subtle brightness and saturation adjustments for macOS-like appearance
        let luminance = dot(tinted, vec3<f32>(0.299, 0.587, 0.114));
        let saturation_adjust = mix(vec3<f32>(luminance), tinted, 1.05); // Slightly boost saturation
        
        // Final glass effect with correct opacity
        return vec4<f32>(saturation_adjust, component.opacity);
    }
    else {
        // Plain color mode
        return base_color;
    }
}

// Normal distribution function for Gaussian kernel
fn normpdf(x: f32, sigma: f32) -> f32 {
    return 0.39894 * exp(-0.5 * x * x / (sigma * sigma)) / sigma;
}

// High-quality Gaussian blur implementation with unlimited scaling
fn gaussian_blur(tex: texture_2d<f32>, samp: sampler, uv: vec2<f32>, blur_radius: f32) -> vec4<f32> {
    if (blur_radius <= 0.0) {
        return textureSample(tex, samp, uv);
    }

    // Remove the cap on blur radius to allow unlimited scaling
    let effective_blur = blur_radius * 2.5;
    
    let tex_size = vec2<f32>(textureDimensions(tex));
    let pixel_size = 1.0 / tex_size;
    
    // Scale sigma based on blur_radius with no upper limit
    // Use a logarithmic scale for very large values to maintain performance
    let sigma = max(2.0, min(log(1.0 + effective_blur) * 5.0, 50.0));
    
    // Dynamically adjust kernel size based on blur radius
    // For extreme blur values, cap the kernel size for performance but increase sampling distance
    let kernel_size = min(15, max(5, i32(min(sigma, 15.0) * 2.5) | 1)); // Ensure odd number
    let k_size = (kernel_size - 1) / 2;
    
    // Create the 1D kernel
    var kernel: array<f32, 15>; // Size capped at 15 for performance
    var z = 0.0;
    
    // Fill kernel with Gaussian values
    for (var j = 0; j <= k_size; j++) {
        let value = normpdf(f32(j), sigma);
        if (j < 15) { // Safety check
            kernel[k_size + j] = value;
            if (j > 0 && (k_size - j) >= 0) {
                kernel[k_size - j] = value;
            }
        }
        if (j > 0) {
            z += 2.0 * value;
        } else {
            z += value;
        }
    }
    
    // Normalize kernel
    for (var j = 0; j < min(kernel_size, 15); j++) {
        kernel[j] /= z;
    }
    
    // Calculate a dynamic sampling scale that increases with blur radius
    // This allows for unlimited blur effect even with limited kernel size
    let sampling_scale = max(1.5, min(effective_blur / 10.0, 20.0));
    
    // Two-pass blur with dynamically scaled sampling offsets for stronger effect
    // First horizontal pass
    var horizontal = vec4<f32>(0.0);
    for (var i = -k_size; i <= k_size; i++) {
        // Scale offset dynamically based on blur radius
        let offset = vec2<f32>(f32(i), 0.0) * pixel_size * sampling_scale;
        var factor: f32 = 0.0;
        if (i < 15 && i >= -k_size) {
            factor = kernel[k_size + i];
        } else {
            factor = 0.0;
        }
        horizontal += textureSample(tex, samp, uv + offset) * factor;
    }
    
    // Second vertical pass with increased sampling radius
    var final_color = vec4<f32>(0.0);
    for (var j = -k_size; j <= k_size; j++) {
        // Increased sampling distance for vertical pass too
        let vertical_uv = uv + vec2<f32>(0.0, f32(j)) * pixel_size * sampling_scale;
        
        // Sample directly from texture for better performance
        var h_sample = vec4<f32>(0.0);
        for (var i = -k_size; i <= k_size; i++) {
            let sample_uv = vertical_uv + vec2<f32>(f32(i), 0.0) * pixel_size * sampling_scale;
            var factor: f32 = 0.0;
            if (i < 15 && i >= -k_size) {
                factor = kernel[k_size + i];
            } else {
                factor = 0.0;
            }
            h_sample += textureSample(tex, samp, sample_uv) * factor;
        }
        
        var kernel_factor: f32 = 0.0;
        if (j < 15 && j >= -k_size) {
            kernel_factor = kernel[k_size + j];
        } else {
            kernel_factor = 0.0;
        }
        final_color += h_sample * kernel_factor;
    }
    
    return final_color;
}

// Convert screen UVs to pixel coordinates 
fn uv_to_pixels(uv: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        uv.x * component.screen_size.x,
        (1.0 - uv.y) * component.screen_size.y
    );
}

// Better shadow calculation that properly handles pill shapes with large border radii
fn simple_shadow(pixel_pos: vec2<f32>, shadow_pos: vec2<f32>, shadow_size: vec2<f32>, radius: vec4<f32>, blur: f32) -> f32 {
    // For pill shape, we need to handle the case where border radius is equal to half the height
    // Get the shape bounds
    let shape_min = shadow_pos;
    let shape_max = shadow_pos + shadow_size;
    
    // Adjust radius to not exceed half the shape size
    let max_radius_x = shadow_size.x * 0.5;
    let max_radius_y = shadow_size.y * 0.5;
    let max_radius = min(max_radius_x, max_radius_y);
    
    let tl_radius = min(radius.x, max_radius);
    let tr_radius = min(radius.y, max_radius);
    let bl_radius = min(radius.z, max_radius);
    let br_radius = min(radius.w, max_radius);
    
    // Get corner centers (where the actual rounded corners are centered)
    let tl_center = vec2<f32>(shape_min.x + tl_radius, shape_min.y + tl_radius);
    let tr_center = vec2<f32>(shape_max.x - tr_radius, shape_min.y + tr_radius);
    let bl_center = vec2<f32>(shape_min.x + bl_radius, shape_max.y - bl_radius);
    let br_center = vec2<f32>(shape_max.x - br_radius, shape_max.y - br_radius);
    
    // Distance to the shape (initialized to a high value)
    var dist_to_shape = 100000.0;
    
    // Check distance to each corner
    // Top-left corner
    if (pixel_pos.x <= tl_center.x && pixel_pos.y <= tl_center.y) {
        dist_to_shape = max(0.0, distance(pixel_pos, tl_center) - tl_radius);
    }
    // Top-right corner
    else if (pixel_pos.x >= tr_center.x && pixel_pos.y <= tr_center.y) {
        dist_to_shape = max(0.0, distance(pixel_pos, tr_center) - tr_radius);
    }
    // Bottom-left corner
    else if (pixel_pos.x <= bl_center.x && pixel_pos.y >= bl_center.y) {
        dist_to_shape = max(0.0, distance(pixel_pos, bl_center) - bl_radius);
    }
    // Bottom-right corner
    else if (pixel_pos.x >= br_center.x && pixel_pos.y >= br_center.y) {
        dist_to_shape = max(0.0, distance(pixel_pos, br_center) - br_radius);
    }
    // Left edge
    else if (pixel_pos.x <= shape_min.x) {
        dist_to_shape = shape_min.x - pixel_pos.x;
    }
    // Right edge
    else if (pixel_pos.x >= shape_max.x) {
        dist_to_shape = pixel_pos.x - shape_max.x;
    }
    // Top edge
    else if (pixel_pos.y <= shape_min.y) {
        dist_to_shape = shape_min.y - pixel_pos.y;
    }
    // Bottom edge
    else if (pixel_pos.y >= shape_max.y) {
        dist_to_shape = pixel_pos.y - shape_max.y;
    }
    // Inside the shape (no shadow)
    else {
        return 0.0;
    }
    
    // Create softer falloff for the shadow
    let falloff_factor = 1.2;
    return smoothstep(blur * falloff_factor, 0.0, dist_to_shape);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_coords = uv_to_pixels(in.uv);
    
    // ======== SHADOW CALCULATION ========
    var shadow_color = vec4<f32>(0.0);
    
    if (component.shadow_blur > 0.0 && component.shadow_opacity > 0.0) {
        // Create shadow at offset position
        let shadow_position = component.position + component.shadow_offset;
        let shadow_size = component.size;
        
        // Calculate shadow intensity using our simplified function
        let shadow_intensity = simple_shadow(
            pixel_coords,
            shadow_position,
            shadow_size,
            component.border_radius,
            component.shadow_blur
        );
        
        shadow_color = vec4<f32>(
            component.shadow_color.rgb,
            component.shadow_color.a * shadow_intensity * component.shadow_opacity
        );
    }
    
    // ======== COMPONENT RENDERING ========
    // Check if pixel is outside component bounds
    if (pixel_coords.x < component.outer_bounds.x || pixel_coords.x > component.outer_bounds.z || 
        pixel_coords.y < component.outer_bounds.y || pixel_coords.y > component.outer_bounds.w) {
        // Only show shadow if outside component bounds
        return shadow_color;
    }

    // Check if in corner and outside radius
    let corner_result = check_corner(pixel_coords);
    let in_corner = corner_result.x > 0.5;
    let corner_dist_sq = corner_result.y;
    let outer_radius_sq = corner_result.w * corner_result.w;
    
    if (in_corner && corner_dist_sq > outer_radius_sq) {
        // Only show shadow if outside component bounds
        return shadow_color;
    }

    // Inside component - render normally
    let tex_coords = calculate_tex_coords(pixel_coords);
    let in_border = check_border(pixel_coords, corner_result);
    
    if (in_border) {
        return component.border_color;
    } else {
        let content_color = get_content_color(pixel_coords, tex_coords, in.color);
        return content_color;
    }
}