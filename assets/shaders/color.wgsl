struct VertexInput {
    @location(0) position: vec2<f32>,  // Vertex position in clip space
    @location(1) uv: vec2<f32>,        // UV coordinates (0-1 range)
}

struct ComponentUniform {
    color : vec4<f32>,
    position : vec2<f32>,           // Position in pixels (top-left corner)
    size : vec2<f32>,               // Size in pixels (width, height)
    border_radius : vec4<f32>,      // Corner radii in pixels (top-left, top-right, bottom-left, bottom-right)
    screen_size : vec2<f32>,        // Viewport dimensions in pixels
    use_texture : u32,              // Flag: 0 for color, 1 for texture, 2 for frosted glass
    blur_radius: f32,               // Blur intensity for frosted glass
    opacity: f32,                   // Component opacity
    tint_intensity: f32,            // Tint intensity for the tint color
    border_width: f32,              // Border thickness in pixels
    border_position: u32,           // Border position: 0=inside, 1=center, 2=outside
    border_color: vec4<f32>,        // Border color
    inner_bounds: vec4<f32>,        // (inner_min.x, inner_min.y, inner_max.x, inner_max.y)
    outer_bounds: vec4<f32>,        // (outer_min.x, outer_min.y, outer_max.x, outer_max.y)
    corner_centers: vec4<f32>,      // (tl_center.x, tl_center.y, tr_center.x, tr_center.y)
    corner_centers2: vec4<f32>,     // (bl_center.x, bl_center.y, br_center.x, br_center.y)
    corner_radii: vec4<f32>,        // (inner_tl_radius, inner_tr_radius, inner_bl_radius, inner_br_radius)
    corner_radii2: vec4<f32>,       // (outer_tl_radius, outer_tr_radius, outer_bl_radius, outer_br_radius)
    shadow_color: vec4<f32>,        // Shadow color
    shadow_offset: vec2<f32>,       // Shadow offset
    shadow_blur: f32,               // Shadow blur intensity
    shadow_opacity: f32,            // Shadow opacity
    clip_bounds: vec4<f32>,         // Clipping bounds (min_x, min_y, max_x, max_y)
    clip_border_radius: vec4<f32>,  // Clipping border radius (top-left, top-right, bottom-left, bottom-right)
    clip_enabled: vec2<f32>,        // Whether clipping is enabled (x, y)
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
    @location(1) uv : vec2<f32>,           // UV coordinates within the quad (0-1)
    @location(2) world_pos : vec2<f32>,    // World position in pixels
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    
    // Calculate world position from UV and component bounds
    // Expand bounds to include shadow if present
    let shadow_expansion = vec2<f32>(
        abs(component.shadow_offset.x) + component.shadow_blur * 2.0,
        abs(component.shadow_offset.y) + component.shadow_blur * 2.0
    );
    
    let expanded_min = vec2<f32>(
        component.outer_bounds.x - shadow_expansion.x,
        component.outer_bounds.y - shadow_expansion.y
    );
    let expanded_max = vec2<f32>(
        component.outer_bounds.z + shadow_expansion.x,
        component.outer_bounds.w + shadow_expansion.y
    );
    
    out.world_pos = mix(expanded_min, expanded_max, vertex.uv);
    out.color = component.color;
    return out;
}

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

// High-quality Gaussian blur function
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

// SDF for rectangle with half-size xy
fn sd_rectangle(p: vec2<f32>, xy: vec2<f32>) -> f32 {
    let d = abs(p) - max(xy, vec2<f32>(0.0));
    let outer = length(max(d, vec2<f32>(0.0)));
    let inner = min(max(d.x, d.y), 0.0);
    return outer + inner;
}

// SDF for rounded rectangle
fn sd_rounded_rectangle(p: vec2<f32>, xy: vec2<f32>, r: vec4<f32>) -> f32 {
    // Select appropriate radius based on quadrant
    let quadrant_x = select(0u, 1u, p.x > 0.0);
    let quadrant_y = select(0u, 2u, p.y < 0.0);
    let radius_index = quadrant_x + quadrant_y;
    
    var s: f32;
    switch radius_index {
        case 0u: { s = r.x; }      // top-left
        case 1u: { s = r.y; }      // top-right  
        case 2u: { s = r.z; }      // bottom-left
        default: { s = r.w; }      // bottom-right
    }
    
    // Clamp radius to prevent overlap
    s = min(s, min(xy.x, xy.y));
    
    return sd_rectangle(p, xy - vec2<f32>(s)) - s;
}

// Get SDF distance for component shape (handles border positioning)
fn get_component_sdf(pixel_coords: vec2<f32>) -> f32 {
    let component_center = component.position + component.size * 0.5;
    let p = pixel_coords - component_center;
    
    // Adjust component size based on border position
    var outer_half_size = component.size * 0.5;
    if (component.border_width > 0.0 && component.border_position == 2u) {
        // Outside border: component grows outward
        outer_half_size = outer_half_size + vec2<f32>(component.border_width);
    }
    
    return sd_rounded_rectangle(p, outer_half_size, component.border_radius);
}

// Get SDF distance for shadow
fn get_shadow_sdf(pixel_coords: vec2<f32>) -> f32 {
    let shadow_position = component.position + component.shadow_offset;
    let shadow_center = shadow_position + component.size * 0.5;
    let p = pixel_coords - shadow_center;
    let half_size = component.size * 0.5;
    
    return sd_rounded_rectangle(p, half_size, component.border_radius);
}

// Get SDF distance for clipping bounds
fn get_clip_sdf(pixel_coords: vec2<f32>) -> f32 {
    let clip_center = vec2<f32>(
        (component.clip_bounds.x + component.clip_bounds.z) * 0.5,
        (component.clip_bounds.y + component.clip_bounds.w) * 0.5
    );
    let clip_half_size = vec2<f32>(
        (component.clip_bounds.z - component.clip_bounds.x) * 0.5,
        (component.clip_bounds.w - component.clip_bounds.y) * 0.5
    );
    let p = pixel_coords - clip_center;
    
    return sd_rounded_rectangle(p, clip_half_size, component.clip_border_radius);
}

// Get SDF distance for inner content area (excluding border)
fn get_inner_sdf(pixel_coords: vec2<f32>) -> f32 {
    if (component.border_width <= 0.0) {
        return get_component_sdf(pixel_coords);
    }
    
    let component_center = component.position + component.size * 0.5;
    let p = pixel_coords - component_center;
    
    // For border positioning:
    // - Inside: border grows inward, reducing content area
    // - Center: border grows both ways, content area reduced by half border width
    // - Outside: border grows outward, content area unchanged (but we still need inner bounds)
    var border_inset: f32;
    switch component.border_position {
        case 0u: { border_inset = component.border_width; }      // inside
        case 1u: { border_inset = component.border_width * 0.5; } // center
        default: { border_inset = 0.0; }                         // outside (no content reduction)
    }
    
    let inner_half_size = component.size * 0.5 - vec2<f32>(border_inset);
    
    // Calculate inner radii (border reduces corner radius by the inset amount)
    let inner_radii = max(component.border_radius - vec4<f32>(border_inset), vec4<f32>(0.0));
    
    return sd_rounded_rectangle(p, inner_half_size, inner_radii);
}

// Simple shadow calculation using SDF
fn calculate_shadow_intensity(pixel_coords: vec2<f32>) -> f32 {
    if (component.shadow_blur <= 0.0 || component.shadow_opacity <= 0.0) {
        return 0.0;
    }
    
    let shadow_dist = get_shadow_sdf(pixel_coords);
    
    // Create soft shadow falloff
    let shadow_edge = component.shadow_blur;
    return smoothstep(shadow_edge, -shadow_edge, shadow_dist);
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_coords = in.world_pos;
   
    // Early clip test using SDF
    if (component.clip_enabled.x > 0.5 || component.clip_enabled.y > 0.5) {
        let clip_dist = get_clip_sdf(pixel_coords);
        if (clip_dist > 0.5) {  // Small threshold for anti-aliasing
            discard;
        }
    }
   
    // Get distances
    let component_dist = get_component_sdf(pixel_coords);
    let inner_dist = get_inner_sdf(pixel_coords);
    
    // Calculate anti-aliasing factor based on screen-space derivatives
    let fwidth_dist = fwidth(component_dist);
    let aa_factor = clamp(fwidth_dist, 0.5, 2.0);
    
    // Shadow calculation
    var shadow_color = vec4<f32>(0.0);
    if (component.shadow_blur > 0.0 && component.shadow_opacity > 0.0) {
        let shadow_intensity = calculate_shadow_intensity(pixel_coords);
        shadow_color = vec4<f32>(
            component.shadow_color.rgb,
            component.shadow_color.a * shadow_intensity * component.shadow_opacity
        );
    }
    
    // Component alpha with precise anti-aliasing
    let component_alpha = smoothstep(aa_factor, -aa_factor, component_dist);
    
    if (component_alpha <= 0.001) {
        // Completely outside component
        return vec4<f32>(shadow_color.rgb, shadow_color.a * component.opacity);
    }
    
    // Check if we have a border
    if (component.border_width > 0.0) {
        // Calculate inner content alpha
        let inner_alpha = smoothstep(aa_factor, -aa_factor, inner_dist);
        
        // If we're inside the inner area, render content
        if (inner_dist <= 0.0) {
            // In content area
            let tex_coords = calculate_tex_coords(pixel_coords);
            
            var content_color: vec4<f32>;
            if (component.use_texture == 0u) {
                content_color = vec4<f32>(in.color.rgb, in.color.a * inner_alpha * component_alpha);
            } else {
                let base_content = get_content_color(pixel_coords, tex_coords, in.color);
                content_color = vec4<f32>(base_content.rgb, base_content.a * inner_alpha * component_alpha);
            }
            
            // Mix border color at the edges for anti-aliasing
            let border_mix_factor = 1.0 - inner_alpha;
            let border_color_with_alpha = vec4<f32>(
                component.border_color.rgb,
                component.border_color.a * component_alpha
            );
            
            let mixed_color = mix(content_color, border_color_with_alpha, border_mix_factor * border_color_with_alpha.a);
            let final_color = mix(shadow_color, mixed_color, mixed_color.a);
            return vec4<f32>(final_color.rgb, final_color.a * component.opacity);
        } else {
            // In border area (between outer and inner boundaries)
            let border_color_with_alpha = vec4<f32>(
                component.border_color.rgb,
                component.border_color.a * component_alpha
            );
            
            let final_color = mix(shadow_color, border_color_with_alpha, border_color_with_alpha.a);
            return vec4<f32>(final_color.rgb, final_color.a * component.opacity);
        }
    } else {
        // No border - render content directly
        let tex_coords = calculate_tex_coords(pixel_coords);
        
        var content_color: vec4<f32>;
        if (component.use_texture == 0u) {
            content_color = vec4<f32>(in.color.rgb, in.color.a * component_alpha);
        } else {
            let base_content = get_content_color(pixel_coords, tex_coords, in.color);
            content_color = vec4<f32>(base_content.rgb, base_content.a * component_alpha);
        }
        
        let final_color = mix(shadow_color, content_color, content_color.a);
        return vec4<f32>(final_color.rgb, final_color.a * component.opacity);
    }
}
