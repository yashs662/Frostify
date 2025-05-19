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
    opacity: f32,               // Component opacity
    tint_intensity: f32,        // Tint intensity for the tint color
    border_color: vec4<f32>,    // Border color
    border_width: f32,          // Border thickness in pixels
    border_position: u32,       // Border position: 0=inside, 1=center, 2=outside
    _padding2: vec2<f32>,       // Padding for alignment
    inner_bounds: vec4<f32>,    // (inner_min.x, inner_min.y, inner_max.x, inner_max.y)
    outer_bounds: vec4<f32>,    // (outer_min.x, outer_min.y, outer_max.x, outer_max.y)
    corner_centers: vec4<f32>,  // (tl_center.x, tl_center.y, tr_center.x, tr_center.y)
    corner_centers2: vec4<f32>, // (bl_center.x, bl_center.y, br_center.x, br_center.y)
    corner_radii: vec4<f32>,    // (inner_tl_radius, inner_tr_radius, inner_bl_radius, inner_br_radius)
    corner_radii2: vec4<f32>,   // (outer_tl_radius, outer_tr_radius, outer_bl_radius, outer_br_radius)
    shadow_color: vec4<f32>,    // Shadow color
    shadow_offset: vec2<f32>,   // Shadow offset
    shadow_blur: f32,           // Shadow blur intensity
    shadow_opacity: f32,        // Shadow opacity
    clip_bounds: vec4<f32>,     // Clipping bounds (min_x, min_y, max_x, max_y)
    clip_border_radius: vec4<f32>, // Clipping border radius (top-left, top-right, bottom-left, bottom-right)
    clip_enabled: vec2<f32>,    // Whether clipping is enabled (x, y)
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
        
        // Use tint_intensity parameter to control tint strength
        var tinted = mix(
            background.rgb, 
            base_color.rgb, 
            base_color.a * component.tint_intensity * component.opacity
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

// Enhanced blur implementation with increased strength and no repeating patterns
fn gaussian_blur(tex: texture_2d<f32>, samp: sampler, uv: vec2<f32>, blur_radius: f32) -> vec4<f32> {
    // Early exit for minimal blur
    if (blur_radius < 0.05) {
        return textureSample(tex, samp, uv);
    }
    
    // Get texture dimensions
    let tex_size = vec2<f32>(textureDimensions(tex));
    let pixel_size = 1.0 / tex_size;
    
    // Allow for much stronger blur by removing the upper limit on effective radius
    let effective_radius = max(2.0, blur_radius);
    
    // For small blur values, use a simple box blur
    if (blur_radius < 3.0) {
        // Simple faster blur for small radii
        var result = vec4<f32>(0.0);
        let sample_count = 9.0;
        
        for (var y = -1; y <= 1; y++) {
            for (var x = -1; x <= 1; x++) {
                let offset = vec2<f32>(f32(x), f32(y)) * pixel_size * blur_radius;
                result += textureSample(tex, samp, uv + offset);
            }
        }
        
        return result / sample_count;
    }
    
    // For stronger blur, increase samples and sampling distance
    var result = vec4<f32>(0.0);
    var total_weight = 0.0;
    
    // Scale steps with blur radius to ensure strong blur
    // For very large blur values, cap the steps but increase the sampling distance
    let num_steps = min(20, max(8, i32(sqrt(effective_radius) * 1.5)));
    
    // Center sample has highest weight
    result += textureSample(tex, samp, uv) * 1.0;
    total_weight += 1.0;
    
    // Use a scaling factor to increase the sampling distance for stronger blur
    let distance_scale = max(1.0, effective_radius / 15.0);
    
    // Create a more natural pattern using a spiral with denser sampling
    for (var i = 1; i <= num_steps; i++) {
        // Calculate current ring weight - less aggressive falloff for stronger blur
        let ring_dist = f32(i) / f32(num_steps);
        let weight = exp(-ring_dist * ring_dist * 3.0); // Gentler falloff for stronger effect
        
        // Calculate ring radius with increased distance for stronger blur
        let ring_radius = f32(i) * (effective_radius / f32(num_steps)) * pixel_size * distance_scale;
        
        // Take samples in a circular pattern - use more directions for stronger blur
        let num_dirs = min(16, max(8, i32(blur_radius / 10.0) + 8));
        let rotation_offset = f32(i) * 0.2 + blur_radius * 0.01; // More variation for larger blur
        
        // Sample in multiple directions around the center
        for (var dir = 0; dir < num_dirs; dir++) {
            let angle = f32(dir) * (2.0 * 3.14159 / f32(num_dirs)) + rotation_offset;
            let offset = vec2<f32>(
                cos(angle) * ring_radius.x,
                sin(angle) * ring_radius.y
            );
            
            // For very strong blur, use multiple samples per direction at different distances
            if (blur_radius > 30.0 && i % 3 == 0) {
                // Add a few extra samples at different distances along the same direction
                for (var j = 1; j <= 2; j++) {
                    let inner_factor = 0.5 + f32(j) * 0.25; // Sample at 75% and 100% of the radius
                    let inner_offset = offset * inner_factor;
                    result += textureSample(tex, samp, uv + inner_offset) * weight * 0.5;
                    total_weight += weight * 0.5;
                }
            }
            
            // Add main sample
            result += textureSample(tex, samp, uv + offset) * weight;
            total_weight += weight;
        }
    }
    
    // Normalize the result
    return result / max(0.001, total_weight);
}

// Convert screen UVs to pixel coordinates 
fn uv_to_pixels(uv: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        uv.x * component.screen_size.x,
        (1.0 - uv.y) * component.screen_size.y
    );
}

// Optimized shadow calculation that uses fewer operations
fn simple_shadow(pixel_pos: vec2<f32>, shadow_pos: vec2<f32>, shadow_size: vec2<f32>, radius: vec4<f32>, blur: f32) -> f32 {
    // Bail early if pixel is far outside shadow region
    let shadow_min = shadow_pos - vec2<f32>(blur * 2.0);
    let shadow_max = shadow_pos + shadow_size + vec2<f32>(blur * 2.0);
    
    if (pixel_pos.x < shadow_min.x || pixel_pos.x > shadow_max.x || 
        pixel_pos.y < shadow_min.y || pixel_pos.y > shadow_max.y) {
        return 0.0;
    }
    
    // Get the shape bounds
    let shape_min = shadow_pos;
    let shape_max = shadow_pos + shadow_size;
    
    // Adjust radius to not exceed half the shape size
    let max_radius = min(shadow_size.x, shadow_size.y) * 0.5;
    let tl_radius = min(radius.x, max_radius);
    let tr_radius = min(radius.y, max_radius);
    let bl_radius = min(radius.z, max_radius);
    let br_radius = min(radius.w, max_radius);
    
    // Get corner centers
    let tl_center = vec2<f32>(shape_min.x + tl_radius, shape_min.y + tl_radius);
    let tr_center = vec2<f32>(shape_max.x - tr_radius, shape_min.y + tr_radius);
    let bl_center = vec2<f32>(shape_min.x + bl_radius, shape_max.y - bl_radius);
    let br_center = vec2<f32>(shape_max.x - br_radius, shape_max.y - br_radius);
    
    // Distance to the shape
    var dist_to_shape = 1000.0;
    
    // Optimized corner checks - only calculate distance if we're actually in the corner region
    if (pixel_pos.x <= tl_center.x && pixel_pos.y <= tl_center.y) {
        dist_to_shape = distance(pixel_pos, tl_center) - tl_radius;
    }
    else if (pixel_pos.x >= tr_center.x && pixel_pos.y <= tr_center.y) {
        dist_to_shape = distance(pixel_pos, tr_center) - tr_radius;
    }
    else if (pixel_pos.x <= bl_center.x && pixel_pos.y >= bl_center.y) {
        dist_to_shape = distance(pixel_pos, bl_center) - bl_radius;
    }
    else if (pixel_pos.x >= br_center.x && pixel_pos.y >= br_center.y) {
        dist_to_shape = distance(pixel_pos, br_center) - br_radius;
    }
    // Inside main rectangle but outside corners
    else if (pixel_pos.x >= shape_min.x && pixel_pos.x <= shape_max.x && 
             pixel_pos.y >= shape_min.y && pixel_pos.y <= shape_max.y) {
        // Inside the shape (no shadow)
        return 0.0;
    }
    else {
        // Nearest edge distance
        dist_to_shape = min(
            min(abs(pixel_pos.x - shape_min.x), abs(pixel_pos.x - shape_max.x)),
            min(abs(pixel_pos.y - shape_min.y), abs(pixel_pos.y - shape_max.y))
        );
    }
    
    // Create more efficient falloff calculation
    return clamp(1.0 - dist_to_shape / max(0.001, blur), 0.0, 1.0);
}

fn is_inside_rounded_rect(pos: vec2<f32>, rect_min: vec2<f32>, rect_max: vec2<f32>, radii: vec4<f32>) -> bool {
    // Early exit for points clearly inside the non-rounded part
    if (pos.x >= rect_min.x + radii.x && pos.x <= rect_max.x - radii.y &&
        pos.y >= rect_min.y + radii.x && pos.y <= rect_max.y - radii.z) {
        return true;
    }
    
    // Get corner centers
    let tl_center = vec2<f32>(rect_min.x + radii.x, rect_min.y + radii.x);
    let tr_center = vec2<f32>(rect_max.x - radii.y, rect_min.y + radii.y);
    let bl_center = vec2<f32>(rect_min.x + radii.z, rect_max.y - radii.z);
    let br_center = vec2<f32>(rect_max.x - radii.w, rect_max.y - radii.w);
    
    // Check corners using squared distances
    if (pos.x <= tl_center.x && pos.y <= tl_center.y) {
        let dist_sq = dot(pos - tl_center, pos - tl_center);
        return dist_sq <= radii.x * radii.x;
    }
    if (pos.x >= tr_center.x && pos.y <= tr_center.y) {
        let dist_sq = dot(pos - tr_center, pos - tr_center);
        return dist_sq <= radii.y * radii.y;
    }
    if (pos.x <= bl_center.x && pos.y >= bl_center.y) {
        let dist_sq = dot(pos - bl_center, pos - bl_center);
        return dist_sq <= radii.z * radii.z;
    }
    if (pos.x >= br_center.x && pos.y >= br_center.y) {
        let dist_sq = dot(pos - br_center, pos - br_center);
        return dist_sq <= radii.w * radii.w;
    }
    
    // In the non-corner regions, check against rect bounds
    return pos.x >= rect_min.x && pos.x <= rect_max.x && 
           pos.y >= rect_min.y && pos.y <= rect_max.y;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_coords = uv_to_pixels(in.uv);
    
    // Early clip test - avoid all calculations if outside clip region
    if (component.clip_enabled.x > 0.5 || component.clip_enabled.y > 0.5) {
        // Check if we need to use rounded clipping
        let use_rounded_clip = component.clip_border_radius.x > 0.0 || 
                               component.clip_border_radius.y > 0.0 || 
                               component.clip_border_radius.z > 0.0 || 
                               component.clip_border_radius.w > 0.0;
                               
        if (use_rounded_clip) {
            // Use rounded rectangle clipping
            let inside_clip = is_inside_rounded_rect(
                pixel_coords,
                vec2<f32>(component.clip_bounds.x, component.clip_bounds.y),
                vec2<f32>(component.clip_bounds.z, component.clip_bounds.w),
                component.clip_border_radius
            );
            
            if (!inside_clip) {
                discard;
            }
        } else {
            // Use original rectangular clipping
            if ((component.clip_enabled.x > 0.5 && (pixel_coords.x < component.clip_bounds.x || pixel_coords.x > component.clip_bounds.z)) ||
                (component.clip_enabled.y > 0.5 && (pixel_coords.y < component.clip_bounds.y || pixel_coords.y > component.clip_bounds.w))) {
                discard;
            }
        }
    }
    
    // Early exit for pixels outside outer bounds - no shadow calculation needed
    if (pixel_coords.x < component.outer_bounds.x - component.shadow_blur * 2.0 || 
        pixel_coords.x > component.outer_bounds.z + component.shadow_blur * 2.0 || 
        pixel_coords.y < component.outer_bounds.y - component.shadow_blur * 2.0 || 
        pixel_coords.y > component.outer_bounds.w + component.shadow_blur * 2.0) {
        // Completely outside any possible rendering area
        discard;
    }
    
    // Shadow calculation - only perform if shadow is actually visible
    var shadow_color = vec4<f32>(0.0);
    if (component.shadow_blur > 0.0 && component.shadow_opacity > 0.0) {
        let shadow_position = component.position + component.shadow_offset;
        let shadow_size = component.size;
        
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
    
    // Check if pixel is outside component bounds
    if (pixel_coords.x < component.outer_bounds.x || pixel_coords.x > component.outer_bounds.z || 
        pixel_coords.y < component.outer_bounds.y || pixel_coords.y > component.outer_bounds.w) {
        // Only show shadow if outside component bounds
        return vec4<f32>(shadow_color.rgb, shadow_color.a * component.opacity);
    }

    // Check corner - optimized to avoid sqrt when possible
    let corner_result = check_corner(pixel_coords);
    let in_corner = corner_result.x > 0.5;
    if (in_corner) {
        let corner_dist_sq = corner_result.y;
        let outer_radius_sq = corner_result.w * corner_result.w;
        
        if (corner_dist_sq > outer_radius_sq) {
            // Only show shadow if outside component bounds
            return vec4<f32>(shadow_color.rgb, shadow_color.a * component.opacity);
        }
    }

    // Inside component - render normally
    let tex_coords = calculate_tex_coords(pixel_coords);
    
    // Border check
    if (component.border_width > 0.0) {
        let in_border = check_border(pixel_coords, corner_result);
        if (in_border) {
            return vec4<f32>(component.border_color.rgb, component.border_color.a * component.opacity);
        }
    }
    
    // Content color with early exit for simple case
    if (component.use_texture == 0u) {
        // Plain color mode - fastest path
        return vec4<f32>(in.color.rgb, in.color.a * component.opacity);
    } else {
        // More complex texture or frosted glass
        let content_color = get_content_color(pixel_coords, tex_coords, in.color);
        return vec4<f32>(content_color.rgb, content_color.a * component.opacity);
    }
}