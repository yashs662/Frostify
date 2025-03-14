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
    border_color: vec4<f32>,    // Border color
    border_width: f32,          // Border thickness in pixels
    border_position: u32,       // Border position: 0=inside, 1=center, 2=outside
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

// Function to calculate rect bounds based on border position
fn calculate_bounds() -> vec4<f32> {
    // Original component bounds (content area)
    let content_min = component.position;
    let content_max = component.position + component.size;

    // Adjust visible bounds based on border position
    var visible_rect_min = content_min;
    var visible_rect_max = content_max;
    
    // For Outside and Center border positions, expand the visible bounds
    if (component.border_width > 0.0) {
        if (component.border_position == 2u) {
            // Outside: visible area extends beyond content by border width
            visible_rect_min -= vec2<f32>(component.border_width);
            visible_rect_max += vec2<f32>(component.border_width);
        } else if (component.border_position == 1u) {
            // Center: expand visible area by half the border width
            visible_rect_min -= vec2<f32>(component.border_width * 0.5);
            visible_rect_max += vec2<f32>(component.border_width * 0.5);
        }
    }
    
    return vec4<f32>(visible_rect_min.x, visible_rect_min.y, visible_rect_max.x, visible_rect_max.y);
}

// Function to calculate corner centers and radii
fn calculate_corner_properties() -> array<vec4<f32>, 4> {
    let content_min = component.position;
    let content_max = component.position + component.size;
    
    // Calculate max radius to prevent overlap
    let max_radius_x = min(component.size.x * 0.5, 10000.0);
    let max_radius_y = min(component.size.y * 0.5, 10000.0);
    let max_radius = min(max_radius_x, max_radius_y);

    // Clamp all radii to max
    let tl_radius = min(component.border_radius.x, max_radius);
    let tr_radius = min(component.border_radius.y, max_radius);
    let bl_radius = min(component.border_radius.z, max_radius);
    let br_radius = min(component.border_radius.w, max_radius);

    // Calculate outer radii based on border position
    var outer_tl_radius = tl_radius;
    var outer_tr_radius = tr_radius;
    var outer_bl_radius = bl_radius;
    var outer_br_radius = br_radius;

    // Calculate inner radii based on border position
    var inner_tl_radius = tl_radius;
    var inner_tr_radius = tr_radius;
    var inner_bl_radius = bl_radius;
    var inner_br_radius = br_radius;

    if (component.border_position == 0u) {
        // Inside borders - reduce inner radius
        inner_tl_radius = max(tl_radius - component.border_width, 0.0);
        inner_tr_radius = max(tr_radius - component.border_width, 0.0);
        inner_bl_radius = max(bl_radius - component.border_width, 0.0);
        inner_br_radius = max(br_radius - component.border_width, 0.0);
    } else if (component.border_position == 1u) {
        // Center borders - add half border width to outer radius and subtract half from inner
        // For center borders, ensure we have continuous corners
        let half_border = component.border_width * 0.5;
        outer_tl_radius = tl_radius + half_border;
        outer_tr_radius = tr_radius + half_border;
        outer_bl_radius = bl_radius + half_border;
        outer_br_radius = br_radius + half_border;
        
        // Ensure inner radius doesn't go negative which causes gaps
        inner_tl_radius = max(tl_radius - half_border, 0.0);
        inner_tr_radius = max(tr_radius - half_border, 0.0);
        inner_bl_radius = max(bl_radius - half_border, 0.0);
        inner_br_radius = max(br_radius - half_border, 0.0);
    } else if (component.border_position == 2u) {
        // Outside borders - add full border width to outer radius
        outer_tl_radius = tl_radius + component.border_width;
        outer_tr_radius = tr_radius + component.border_width;
        outer_bl_radius = bl_radius + component.border_width;
        outer_br_radius = br_radius + component.border_width;
    }

    // Calculate corner centers (for the content box)
    let tl_center = vec2<f32>(content_min.x + tl_radius, content_min.y + tl_radius);
    let tr_center = vec2<f32>(content_max.x - tr_radius, content_min.y + tr_radius);
    let bl_center = vec2<f32>(content_min.x + bl_radius, content_max.y - bl_radius);
    let br_center = vec2<f32>(content_max.x - br_radius, content_max.y - br_radius);
    
    // Return array of corner properties:
    // For each vec4: (center.x, center.y, inner_radius, outer_radius)
    return array<vec4<f32>, 4>(
        vec4<f32>(tl_center.x, tl_center.y, inner_tl_radius, outer_tl_radius),
        vec4<f32>(tr_center.x, tr_center.y, inner_tr_radius, outer_tr_radius),
        vec4<f32>(bl_center.x, bl_center.y, inner_bl_radius, outer_bl_radius),
        vec4<f32>(br_center.x, br_center.y, inner_br_radius, outer_br_radius)
    );
}

// Function to check if pixel is in a corner and get corner properties
fn check_corner(pixel_coords: vec2<f32>) -> vec4<f32> {
    let content_min = component.position;
    let content_max = component.position + component.size;
    
    // Get corner properties
    let corners = calculate_corner_properties();
    
    // Return values: (is_in_corner, distance, inner_radius, outer_radius)
    // Default to not in corner
    var result = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    
    // Check top-left corner region
    let tl_center = vec2<f32>(corners[0].x, corners[0].y);
    let tl_dist = distance(pixel_coords, tl_center);
    
    // Check if we're in the corner's detection region
    if (pixel_coords.x <= tl_center.x && pixel_coords.y <= tl_center.y) {
        return vec4<f32>(1.0, tl_dist, corners[0].z, corners[0].w);
    }
    
    // Check top-right corner region
    let tr_center = vec2<f32>(corners[1].x, corners[1].y);
    let tr_dist = distance(pixel_coords, tr_center);
    
    if (pixel_coords.x >= tr_center.x && pixel_coords.y <= tr_center.y) {
        return vec4<f32>(1.0, tr_dist, corners[1].z, corners[1].w);
    }
    
    // Check bottom-left corner region
    let bl_center = vec2<f32>(corners[2].x, corners[2].y);
    let bl_dist = distance(pixel_coords, bl_center);
    
    if (pixel_coords.x <= bl_center.x && pixel_coords.y >= bl_center.y) {
        return vec4<f32>(1.0, bl_dist, corners[2].z, corners[2].w);
    }
    
    // Check bottom-right corner region
    let br_center = vec2<f32>(corners[3].x, corners[3].y);
    let br_dist = distance(pixel_coords, br_center);
    
    if (pixel_coords.x >= br_center.x && pixel_coords.y >= br_center.y) {
        return vec4<f32>(1.0, br_dist, corners[3].z, corners[3].w);
    }
    
    // Not in any corner
    return result;
}

// Function to check if we're in the border area
fn check_border(pixel_coords: vec2<f32>, bounds: vec4<f32>, corner_result: vec4<f32>) -> bool {
    if (component.border_width <= 0.0) {
        return false;
    }
    
    let content_min = component.position;
    let content_max = component.position + component.size;
    
    // Calculate the inner content bounds based on border position
    var inner_min = content_min;
    var inner_max = content_max;
    
    if (component.border_position == 0u) {
        // Inside border
        inner_min += vec2<f32>(component.border_width);
        inner_max -= vec2<f32>(component.border_width);
    } else if (component.border_position == 1u) {
        // Center border
        inner_min += vec2<f32>(component.border_width * 0.5);
        inner_max -= vec2<f32>(component.border_width * 0.5);
    }
    // For outside border, inner_min/max == content_min/max (no change)
    
    // For corners, check if we're in the border ring
    let in_corner = corner_result.x > 0.5;
    if (in_corner) {
        let corner_dist = corner_result.y;
        let inner_radius = corner_result.z;
        let outer_radius = corner_result.w;
        
        // Special handling for center borders to prevent gaps
        if (component.border_position == 1u) {
            // Ensure we include the entire border area with a small epsilon for center borders
            return corner_dist >= max(inner_radius - 0.5, 0.0) && corner_dist <= (outer_radius + 0.5);
        } else {
            // In border if between inner and outer radius
            return corner_dist >= inner_radius && corner_dist <= outer_radius;
        }
    } else {
        // For straight edges, check if outside the inner content area but inside the outer bounds
        return (pixel_coords.x < inner_min.x || 
                pixel_coords.x > inner_max.x || 
                pixel_coords.y < inner_min.y || 
                pixel_coords.y > inner_max.y) &&
               (pixel_coords.x >= bounds.x &&
                pixel_coords.x <= bounds.z &&
                pixel_coords.y >= bounds.y &&
                pixel_coords.y <= bounds.w);
    }
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

// Updated function to get anti-aliased color for borders
fn get_border_color(pixel_coords: vec2<f32>, in_corner: bool, corner_dist: f32, inner_radius: f32, outer_radius: f32) -> vec4<f32> {
    let aa_width = 1.0;
    var color = component.border_color;
    
    if (in_corner) {
        // For center borders, we need special handling for the corner transitions
        if (component.border_position == 1u) {
            // Extend inner radius slightly to ensure no gaps
            let effective_inner = max(inner_radius - 0.5, 0.0);
            let inner_aa = smoothstep(effective_inner - aa_width, effective_inner, corner_dist);
            let outer_aa = smoothstep(outer_radius + 0.5, outer_radius + 0.5 - aa_width, corner_dist);
            color.a *= inner_aa * outer_aa;
        } else {
            let inner_aa = smoothstep(inner_radius - aa_width, inner_radius, corner_dist);
            let outer_aa = smoothstep(outer_radius, outer_radius - aa_width, corner_dist);
            color.a *= inner_aa * outer_aa;
        }
    }
    
    return color;
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

// Apply anti-aliasing to edges
fn apply_edge_aa(color: vec4<f32>, pixel_coords: vec2<f32>, corner_result: vec4<f32>, in_border: bool) -> vec4<f32> {
    let aa_width = 1.0;
    var result = color;
    
    // If we're in a corner, apply anti-aliasing to the rounded edges
    if (corner_result.x > 0.5) {
        let corner_dist = corner_result.y;
        let inner_radius = corner_result.z;
        let outer_radius = corner_result.w;
        
        if (in_border) {
            // For center border case, ensure we don't create gaps
            if (component.border_position == 1u) {
                let effective_inner = max(inner_radius - 0.5, 0.0);
                let inner_fade = smoothstep(effective_inner - aa_width, effective_inner, corner_dist);
                let outer_fade = smoothstep(outer_radius + 0.5, outer_radius + 0.5 - aa_width, corner_dist);
                result.a *= inner_fade * outer_fade;
            } else {
                let inner_fade = smoothstep(inner_radius - aa_width, inner_radius, corner_dist);
                let outer_fade = smoothstep(outer_radius, outer_radius - aa_width, corner_dist);
                result.a *= inner_fade * outer_fade;
            }
        } else {
            // Apply AA to content edge when not in border
            if (component.border_width <= 0.0) {
                result.a *= smoothstep(inner_radius, inner_radius - aa_width, corner_dist);
            } else {
                // Additional check to improve content area anti-aliasing
                if (component.border_position == 1u) {
                    // For center border, use modified inner radius calculation
                    let effective_inner = max(inner_radius - 0.5, 0.0);
                    result.a *= smoothstep(effective_inner, effective_inner - aa_width, corner_dist);
                } else {
                    result.a *= smoothstep(inner_radius, inner_radius - aa_width, corner_dist);
                }
            }
        }
    } else {
        // For straight edges, apply anti-aliasing
        let content_min = component.position;
        let content_max = component.position + component.size;
        
        // Calculate the inner content bounds based on border position
        var inner_min = content_min;
        var inner_max = content_max;
        
        if (component.border_position == 0u) {
            inner_min += vec2<f32>(component.border_width);
            inner_max -= vec2<f32>(component.border_width);
        } else if (component.border_position == 1u) {
            inner_min += vec2<f32>(component.border_width * 0.5);
            inner_max -= vec2<f32>(component.border_width * 0.5);
        }
        
        // Apply anti-aliasing to straight edges for borders
        if (in_border) {
            // For center borders, ensure continuous edges
            if (component.border_position == 1u) {
                // Distance to nearest inner edge
                let dist_to_inner_edge_x = min(
                    abs(pixel_coords.x - inner_min.x),
                    abs(pixel_coords.x - inner_max.x)
                );
                let dist_to_inner_edge_y = min(
                    abs(pixel_coords.y - inner_min.y),
                    abs(pixel_coords.y - inner_max.y)
                );
                
                // Distance to nearest outer edge
                let dist_to_outer_edge_x = min(
                    abs(pixel_coords.x - content_min.x),
                    abs(pixel_coords.x - content_max.x)
                );
                let dist_to_outer_edge_y = min(
                    abs(pixel_coords.y - content_min.y),
                    abs(pixel_coords.y - content_max.y)
                );
                
                // Apply anti-aliasing fade based on the nearest edge
                let inner_fade = smoothstep(0.0, aa_width, min(dist_to_inner_edge_x, dist_to_inner_edge_y));
                let outer_fade = smoothstep(0.0, aa_width, min(dist_to_outer_edge_x, dist_to_outer_edge_y));
                
                result.a *= inner_fade * outer_fade;
            }
        }
    }
    
    return result;
}

// Convert screen UVs to pixel coordinates 
fn uv_to_pixels(uv: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        uv.x * component.screen_size.x,
        (1.0 - uv.y) * component.screen_size.y
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert UV to pixel coordinates
    let pixel_coords = uv_to_pixels(in.uv);

    // Calculate bounds based on border position
    let bounds = calculate_bounds();
    
    // Check if we're outside the expanded visible bounds
    if (pixel_coords.x < bounds.x || pixel_coords.x > bounds.z || 
        pixel_coords.y < bounds.y || pixel_coords.y > bounds.w) {
        discard;
    }

    // Check if in corner regions and get properties
    let corner_result = check_corner(pixel_coords);
    let in_corner = corner_result.x > 0.5;
    let corner_dist = corner_result.y;
    let inner_radius = corner_result.z;
    let outer_radius = corner_result.w;
    
    // If in a corner, check for clipping
    if (in_corner && corner_dist > outer_radius) {
        discard;
    }

    // Check if we're in the border area
    let in_border = check_border(pixel_coords, bounds, corner_result);
    
    // Calculate texture coordinates
    let tex_coords = calculate_tex_coords(pixel_coords);
    
    var final_color: vec4<f32>;

    // Determine the final color based on whether we're in border or not
    if (in_border && component.border_width > 0.0) {
        // Get border color with proper anti-aliasing
        final_color = get_border_color(pixel_coords, in_corner, corner_dist, inner_radius, outer_radius);
    } else {
        final_color = get_content_color(pixel_coords, tex_coords, in.color);
    }
    
    // Apply anti-aliasing
    final_color = apply_edge_aa(final_color, pixel_coords, corner_result, in_border);
    
    return final_color;
}