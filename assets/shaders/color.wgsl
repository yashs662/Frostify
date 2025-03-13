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
    
    // Limit corner radii to avoid overlap
    var max_radius_x = min(component.size.x * 0.5, 10000.0);
    var max_radius_y = min(component.size.y * 0.5, 10000.0);
    var max_radius = min(max_radius_x, max_radius_y);

    // Clamp all radii
    var tl_radius = min(component.border_radius.x, max_radius);
    var tr_radius = min(component.border_radius.y, max_radius);
    var bl_radius = min(component.border_radius.z, max_radius);
    var br_radius = min(component.border_radius.w, max_radius);

    // Calculate outer radii based on border position
    var outer_tl_radius = tl_radius;
    var outer_tr_radius = tr_radius;
    var outer_bl_radius = bl_radius;
    var outer_br_radius = br_radius;

    if (component.border_position == 1u) {
        // Center borders - add half border width to outer radius
        outer_tl_radius = tl_radius + component.border_width * 0.5;
        outer_tr_radius = tr_radius + component.border_width * 0.5;
        outer_bl_radius = bl_radius + component.border_width * 0.5;
        outer_br_radius = br_radius + component.border_width * 0.5;
    } else if (component.border_position == 2u) {
        // Outside borders - add full border width to outer radius
        outer_tl_radius = tl_radius + component.border_width;
        outer_tr_radius = tr_radius + component.border_width;
        outer_bl_radius = bl_radius + component.border_width;
        outer_br_radius = br_radius + component.border_width;
    }

    // Calculate corner centers
    let tl_center = vec2<f32>(content_min.x + tl_radius, content_min.y + tl_radius);
    let tr_center = vec2<f32>(content_max.x - tr_radius, content_min.y + tr_radius);
    let bl_center = vec2<f32>(content_min.x + bl_radius, content_max.y - bl_radius);
    let br_center = vec2<f32>(content_max.x - br_radius, content_max.y - br_radius);
    
    // Return array of corner properties:
    // For each vec4: (center.x, center.y, inner_radius, outer_radius)
    return array<vec4<f32>, 4>(
        vec4<f32>(tl_center.x, tl_center.y, tl_radius, outer_tl_radius),
        vec4<f32>(tr_center.x, tr_center.y, tr_radius, outer_tr_radius),
        vec4<f32>(bl_center.x, bl_center.y, bl_radius, outer_bl_radius),
        vec4<f32>(br_center.x, br_center.y, br_radius, outer_br_radius)
    );
}

// Function to check if pixel is in a corner and get corner properties
fn check_corner(pixel_coords: vec2<f32>, bounds: vec4<f32>, corners: array<vec4<f32>, 4>) -> vec4<f32> {
    let visible_rect_min = vec2<f32>(bounds.x, bounds.y);
    let visible_rect_max = vec2<f32>(bounds.z, bounds.w);
    let content_min = component.position;
    let content_max = component.position + component.size;
    
    // Return values: (is_in_corner, distance, corner_radius, outer_radius)
    // Default to not in corner
    var result = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    
    // Check top-left corner
    var corner_min_x = visible_rect_min.x;
    var corner_max_x = content_min.x + corners[0].z; // tl_radius
    var corner_min_y = visible_rect_min.y;
    var corner_max_y = content_min.y + corners[0].z; // tl_radius
    
    if (pixel_coords.x >= corner_min_x && pixel_coords.x <= corner_max_x && 
        pixel_coords.y >= corner_min_y && pixel_coords.y <= corner_max_y) {
        let corner_center = vec2<f32>(corners[0].x, corners[0].y);
        let dist = distance(pixel_coords, corner_center);
        return vec4<f32>(1.0, dist, corners[0].z, corners[0].w); // (in_corner, dist, radius, outer_radius)
    }
    
    // Check top-right corner
    corner_min_x = content_max.x - corners[1].z; // tr_radius
    corner_max_x = visible_rect_max.x;
    corner_min_y = visible_rect_min.y;
    corner_max_y = content_min.y + corners[1].z; // tr_radius
    
    if (pixel_coords.x >= corner_min_x && pixel_coords.x <= corner_max_x && 
        pixel_coords.y >= corner_min_y && pixel_coords.y <= corner_max_y) {
        let corner_center = vec2<f32>(corners[1].x, corners[1].y);
        let dist = distance(pixel_coords, corner_center);
        return vec4<f32>(1.0, dist, corners[1].z, corners[1].w); // (in_corner, dist, radius, outer_radius)
    }
    
    // Check bottom-left corner
    corner_min_x = visible_rect_min.x;
    corner_max_x = content_min.x + corners[2].z; // bl_radius
    corner_min_y = content_max.y - corners[2].z; // bl_radius
    corner_max_y = visible_rect_max.y;
    
    if (pixel_coords.x >= corner_min_x && pixel_coords.x <= corner_max_x && 
        pixel_coords.y >= corner_min_y && pixel_coords.y <= corner_max_y) {
        let corner_center = vec2<f32>(corners[2].x, corners[2].y);
        let dist = distance(pixel_coords, corner_center);
        return vec4<f32>(1.0, dist, corners[2].z, corners[2].w); // (in_corner, dist, radius, outer_radius)
    }
    
    // Check bottom-right corner
    corner_min_x = content_max.x - corners[3].z; // br_radius
    corner_max_x = visible_rect_max.x;
    corner_min_y = content_max.y - corners[3].z; // br_radius
    corner_max_y = visible_rect_max.y;
    
    if (pixel_coords.x >= corner_min_x && pixel_coords.x <= corner_max_x && 
        pixel_coords.y >= corner_min_y && pixel_coords.y <= corner_max_y) {
        let corner_center = vec2<f32>(corners[3].x, corners[3].y);
        let dist = distance(pixel_coords, corner_center);
        return vec4<f32>(1.0, dist, corners[3].z, corners[3].w); // (in_corner, dist, radius, outer_radius)
    }
    
    // Not in any corner
    return result;
}

// Function to check if we're in the border area
fn check_border(pixel_coords: vec2<f32>, bounds: vec4<f32>, corner_result: vec4<f32>) -> bool {
    let visible_rect_min = vec2<f32>(bounds.x, bounds.y);
    let visible_rect_max = vec2<f32>(bounds.z, bounds.w);
    let content_min = component.position;
    let content_max = component.position + component.size;
    
    if (component.border_width <= 0.0) {
        return false;
    }
    
    // Calculate the inner content bounds based on border position
    var inner_min = content_min;
    var inner_max = content_max;
    
    if (component.border_position == 0u) {
        // Inside border
        inner_min += vec2<f32>(component.border_width);
        inner_max -= vec2<f32>(component.border_width);
    } else if (component.border_position == 1u) {
        // Center border - inner bounds should be inset by half the border
        inner_min += vec2<f32>(component.border_width * 0.5);
        inner_max -= vec2<f32>(component.border_width * 0.5);
    }
    // For outside border, inner_min/max == content_min/max (no change)
    
    // For corners, check if we're in the border ring
    let in_corner = corner_result.x > 0.5;
    if (in_corner) {
        let corner_dist = corner_result.y;
        let corner_radius = corner_result.z;
        let outer_radius = corner_result.w;
        
        var inner_radius = 0.0;
        
        if (component.border_position == 0u) {
            // Inside border: inner radius is corner radius - full border width
            inner_radius = max(corner_radius - component.border_width, 0.0);
        } else if (component.border_position == 1u) {
            // Center border: inner radius is corner radius - half border width
            inner_radius = max(corner_radius - component.border_width * 0.5, 0.0);
        } else {
            // Outside border: inner radius is the same as corner radius
            inner_radius = corner_radius;
        }
        
        // In border if between inner and outer radius
        var in_border = corner_dist >= inner_radius && corner_dist <= outer_radius;
        
        // For center and outside borders, restrict to the expanded visible area
        if (component.border_position != 0u) {
            in_border = in_border && 
               pixel_coords.x >= visible_rect_min.x && 
               pixel_coords.x <= visible_rect_max.x &&
               pixel_coords.y >= visible_rect_min.y && 
               pixel_coords.y <= visible_rect_max.y;
        }
        
        return in_border;
    } else {
        // For straight edges, check if outside the inner content area but inside visible bounds
        return (pixel_coords.x < inner_min.x || 
               pixel_coords.x > inner_max.x || 
               pixel_coords.y < inner_min.y || 
               pixel_coords.y > inner_max.y) &&
              (pixel_coords.x >= visible_rect_min.x &&
               pixel_coords.x <= visible_rect_max.x &&
               pixel_coords.y >= visible_rect_min.y &&
               pixel_coords.y <= visible_rect_max.y);
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

// Function to get anti-aliased color for borders
fn get_border_color(pixel_coords: vec2<f32>, in_corner: bool, corner_dist: f32, corner_radius: f32, outer_radius: f32) -> vec4<f32> {
    let aa_width = 1.5;
    var color = component.border_color;
    
    if (in_corner) {
        var inner_radius = 0.0;
        
        if (component.border_position == 0u) {
            inner_radius = max(corner_radius - component.border_width, 0.0);
        } else if (component.border_position == 1u) {
            inner_radius = max(corner_radius - component.border_width * 0.5, 0.0);
        } else {
            inner_radius = corner_radius;
        }
        
        // Fade in at inner edge and fade out at outer edge
        let inner_aa = smoothstep(inner_radius - aa_width, inner_radius, corner_dist);
        let outer_aa = smoothstep(outer_radius, outer_radius - aa_width, corner_dist);
        color.a *= inner_aa * outer_aa;
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
        // Frosted glass effect
        var blurAmount = component.blur_radius * 1.2;
        var background = sample_blur(t_diffuse, s_diffuse, tex_coords, blurAmount);
        
        // Mix the background with the tint color, respecting opacity
        var tinted = mix(
            background.rgb, 
            base_color.rgb, 
            base_color.a * 0.4 * component.opacity
        );
        
        // Calculate local coordinates for edge effects
        let local_tex_coords = vec2<f32>(
            (pixel_coords.x - content_min.x) / (content_max.x - content_min.x),
            (pixel_coords.y - content_min.y) / (content_max.y - content_min.y)
        );
        
        // Add subtle edge highlighting
        let edge_distance_x = min(local_tex_coords.x, 1.0 - local_tex_coords.x) * 2.0;
        let edge_distance_y = min(local_tex_coords.y, 1.0 - local_tex_coords.y) * 2.0;
        let edge_distance = min(edge_distance_x, edge_distance_y);
        
        let edge_highlight = smoothstep(0.0, 0.5, edge_distance) * 0.05;
        tinted += vec3<f32>(edge_highlight);
        
        return vec4<f32>(pow(tinted, vec3<f32>(0.98)), component.opacity);
    }
    else {
        // Plain color mode
        return base_color;
    }
}

// Apply anti-aliasing to edges
fn apply_edge_aa(color: vec4<f32>, pixel_coords: vec2<f32>, in_corner: bool, in_border: bool) -> vec4<f32> {
    let content_min = component.position;
    let content_max = component.position + component.size;
    let aa_width = 1.5;
    
    var result = color;
    
    if (!in_corner && !in_border) {
        // If we're at content edges (not in corner or border), apply AA
        let edge_dist = min(min(
            pixel_coords.x - content_min.x, 
            content_max.x - pixel_coords.x
        ), min(
            pixel_coords.y - content_min.y, 
            content_max.y - pixel_coords.y
        ));
        
        if (edge_dist < aa_width) {
            result.a *= edge_dist / aa_width;
        }
    }
    
    return result;
}

// Improved Gaussian blur with better performance and quality
fn sample_blur(tex: texture_2d<f32>, samp: sampler, uv: vec2<f32>, blur_amount: f32) -> vec4<f32> {
    if (blur_amount <= 0.0) {
        return textureSample(tex, samp, uv);
    }

    // Get texture dimensions for proper scaling of the blur
    let tex_size = vec2<f32>(textureDimensions(tex));
    let pixel_size = 1.0 / tex_size;
    
    // Scale blur based on a percentage of screen size for consistent results
    let sigma = blur_amount * 0.05;
    
    var color = vec4<f32>(0.0);
    var total_weight = 0.0;
    
    // Use a more optimized 1-pass blur with fewer samples for better performance
    let max_samples = 13; // Smaller sample count for better performance
    let half_samples = max_samples / 2;
    
    // Single-pass blur that approximates a two-pass Gaussian
    for (var i = -half_samples; i <= half_samples; i++) {
        for (var j = -half_samples; j <= half_samples; j++) {
            let offset = vec2<f32>(f32(i), f32(j)) * pixel_size * sigma * 3.0;
            let sample_pos = uv + offset;
            
            // Use a single-pass circular Gaussian weight function
            let dist_squared = f32(i*i + j*j);
            let weight = exp(-dist_squared / (2.0 * sigma * sigma * 10.0));
            
            color += textureSample(tex, samp, sample_pos) * weight;
            total_weight += weight;
        }
    }
    
    return color / total_weight;
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
    var pixel_coords = uv_to_pixels(in.uv);

    // Calculate bounds based on border position
    let bounds = calculate_bounds();
    let visible_rect_min = vec2<f32>(bounds.x, bounds.y);
    let visible_rect_max = vec2<f32>(bounds.z, bounds.w);
    let content_min = component.position;
    let content_max = component.position + component.size;

    // Check if we're outside the expanded visible bounds
    if (pixel_coords.x < visible_rect_min.x || pixel_coords.x > visible_rect_max.x || 
        pixel_coords.y < visible_rect_min.y || pixel_coords.y > visible_rect_max.y) {
        discard;
    }

    // Get corner properties (centers and radii)
    let corners = calculate_corner_properties();
    
    // Check if in corner regions and get properties
    let corner_props = check_corner(pixel_coords, bounds, corners);
    let in_corner = corner_props.x > 0.5;
    let corner_dist = corner_props.y;
    let corner_radius = corner_props.z;
    let outer_radius = corner_props.w;
    
    // If in a corner, we need to check for outer radius clipping
    if (in_corner) {
        var max_corner_dist: f32;
        
        if (component.border_position == 0u) {
            // Inside: clip at the original corner radius
            max_corner_dist = corner_radius;
            // For inside borders, also ensure we're within content bounds
            if (pixel_coords.x < content_min.x || 
                pixel_coords.x > content_max.x || 
                pixel_coords.y < content_min.y || 
                pixel_coords.y > content_max.y) {
                discard;
            }
        } else if (component.border_position == 1u) {
            // Center: clip at corner radius + half border width
            max_corner_dist = corner_radius + component.border_width * 0.5;
        } else {
            // Outside: clip at outer radius
            max_corner_dist = outer_radius;
        }
        
        // Discard pixels beyond the maximum allowed corner distance
        if (corner_dist > max_corner_dist) {
            discard;
        }
    }

    // Calculate texture coordinates
    let tex_coords = calculate_tex_coords(pixel_coords);

    // Check if we're in the border area
    let in_border = check_border(pixel_coords, bounds, corner_props);
    
    var final_color: vec4<f32>;

    // Determine the final color based on whether we're in border or not
    if (in_border && component.border_width > 0.0) {
        // Use border color with anti-aliasing
        final_color = get_border_color(pixel_coords, in_corner, corner_dist, corner_radius, outer_radius);
    } else {
        // Not in border, use regular content coloring
        final_color = get_content_color(pixel_coords, tex_coords, in.color);
    }
    
    // Apply anti-aliasing to the edges
    final_color = apply_edge_aa(final_color, pixel_coords, in_corner, in_border);
    
    return final_color;
}