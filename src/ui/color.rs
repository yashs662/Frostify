use crate::ui::component::{BackgroundGradientConfig, GradientType};
use colorgrad::Gradient;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Color {
    AliceBlue,
    AntiqueWhite,
    Aquamarine,
    Azure,
    Beige,
    Bisque,
    Black,
    Blue,
    Crimson,
    Cyan,
    DarkGray,
    DarkGreen,
    Fuchsia,
    Gold,
    Gray,
    Green,
    Indigo,
    LimeGreen,
    Maroon,
    MidnightBlue,
    Navy,
    Olive,
    Orange,
    OrangeRed,
    Pink,
    Purple,
    Red,
    Salmon,
    SeaGreen,
    Silver,
    Teal,
    Tomato,
    Turquoise,
    Violet,
    White,
    Yellow,
    YellowGreen,
    Custom([f32; 4]),
    Transparent,
}

#[allow(dead_code)]
impl Color {
    pub fn value(&self) -> [f32; 4] {
        match *self {
            Color::AliceBlue => [0.94, 0.97, 1.0, 1.0],
            Color::AntiqueWhite => [0.98, 0.92, 0.84, 1.0],
            Color::Aquamarine => [0.49, 1.0, 0.83, 1.0],
            Color::Azure => [0.94, 1.0, 1.0, 1.0],
            Color::Beige => [0.96, 0.96, 0.86, 1.0],
            Color::Bisque => [1.0, 0.89, 0.77, 1.0],
            Color::Black => [0.0, 0.0, 0.0, 1.0],
            Color::Blue => [0.0, 0.0, 1.0, 1.0],
            Color::Crimson => [0.86, 0.08, 0.24, 1.0],
            Color::Cyan => [0.0, 1.0, 1.0, 1.0],
            Color::DarkGray => [0.25, 0.25, 0.25, 1.0],
            Color::DarkGreen => [0.0, 0.5, 0.0, 1.0],
            Color::Fuchsia => [1.0, 0.0, 1.0, 1.0],
            Color::Gold => [1.0, 0.84, 0.0, 1.0],
            Color::Gray => [0.5, 0.5, 0.5, 1.0],
            Color::Green => [0.0, 1.0, 0.0, 1.0],
            Color::Indigo => [0.29, 0.0, 0.51, 1.0],
            Color::LimeGreen => [0.2, 0.8, 0.2, 1.0],
            Color::Maroon => [0.5, 0.0, 0.0, 1.0],
            Color::MidnightBlue => [0.1, 0.1, 0.44, 1.0],
            Color::Navy => [0.0, 0.0, 0.5, 1.0],
            Color::Olive => [0.5, 0.5, 0.0, 1.0],
            Color::Orange => [1.0, 0.65, 0.0, 1.0],
            Color::OrangeRed => [1.0, 0.27, 0.0, 1.0],
            Color::Pink => [1.0, 0.08, 0.58, 1.0],
            Color::Purple => [0.5, 0.0, 0.5, 1.0],
            Color::Red => [1.0, 0.0, 0.0, 1.0],
            Color::Salmon => [0.98, 0.5, 0.45, 1.0],
            Color::SeaGreen => [0.18, 0.55, 0.34, 1.0],
            Color::Silver => [0.75, 0.75, 0.75, 1.0],
            Color::Teal => [0.0, 0.5, 0.5, 1.0],
            Color::Tomato => [1.0, 0.39, 0.28, 1.0],
            Color::Turquoise => [0.25, 0.88, 0.82, 1.0],
            Color::Violet => [0.93, 0.51, 0.93, 1.0],
            Color::White => [1.0, 1.0, 1.0, 1.0],
            Color::Yellow => [1.0, 1.0, 0.0, 1.0],
            Color::YellowGreen => [0.6, 0.8, 0.2, 1.0],
            Color::Custom(color) => color,
            Color::Transparent => [0.0, 0.0, 0.0, 0.0],
        }
    }

    pub fn with_alpha(&self, alpha: f32) -> Color {
        let [r, g, b, _] = self.value();
        Color::Custom([r, g, b, alpha])
    }

    pub fn darken(&self, factor: f32) -> Color {
        let factor = factor.clamp(0.0, 1.0);
        let [r, g, b, a] = self.value();
        Color::Custom([r * factor, g * factor, b * factor, a])
    }

    pub fn lighten(&self, factor: f32) -> Color {
        let factor = factor.clamp(0.0, 1.0);
        let [r, g, b, a] = self.value();
        Color::Custom([
            r + (1.0 - r) * factor,
            g + (1.0 - g) * factor,
            b + (1.0 - b) * factor,
            a,
        ])
    }

    pub fn to_rgb_0_255(self) -> [u8; 4] {
        let [r, g, b, a] = self.value();
        [
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
            (a * 255.0) as u8,
        ]
    }

    pub fn lerp(&self, other: &Color, t: f32) -> Color {
        //use colorgrad::Gradient;
        let gradient = colorgrad::GradientBuilder::new()
            .colors(&[self.to_colorgrad_color(), other.to_colorgrad_color()])
            .domain(&[0.0, 1.0])
            .build::<colorgrad::LinearGradient>()
            .unwrap();

        let color = gradient.at(t);
        Color::Custom([color.r, color.g, color.b, color.a])
    }

    pub fn to_glyphon_color(self) -> glyphon::Color {
        let [r, g, b, a] = self.to_rgb_0_255();
        glyphon::Color::rgba(r, g, b, a)
    }

    pub fn to_colorgrad_color(self) -> colorgrad::Color {
        let [r, g, b, a] = self.value();
        colorgrad::Color::from_linear_rgba(r, g, b, a)
    }

    // Method to create a 2D gradient texture for more complex gradients
    pub fn generate_2d_gradient_texture(
        gradient_config: BackgroundGradientConfig,
        width: u32,
        height: u32,
    ) -> image::RgbaImage {
        let mut colors = vec![];
        let mut positions = vec![];

        for stop in gradient_config.color_stops {
            colors.push(stop.color.to_colorgrad_color());
            positions.push(stop.position);
        }

        let g = colorgrad::GradientBuilder::new()
            .colors(&colors)
            .domain(&positions)
            .build::<colorgrad::LinearGradient>()
            .unwrap();

        match gradient_config.gradient_type {
            GradientType::Linear => {
                // Convert angle from degrees to radians
                let angle_rad = gradient_config.angle * std::f32::consts::PI / 180.0;

                // Calculate the gradient direction vector
                let dir_x = angle_rad.cos();
                let dir_y = angle_rad.sin();

                // Calculate the maximum possible distance in this direction
                let max_dist =
                    ((width as f32 * dir_x).abs() + (height as f32 * dir_y).abs()).max(1.0);

                // Create the image
                image::ImageBuffer::from_fn(width, height, |x, y| {
                    // Project the pixel coordinates onto the gradient direction vector
                    let px = x as f32 - width as f32 / 2.0;
                    let py = y as f32 - height as f32 / 2.0;

                    // Calculate the normalized position along the gradient (0 to 1)
                    let proj = (px * dir_x + py * dir_y) / max_dist + 0.5;
                    let clamped_proj = proj.clamp(0.0, 1.0);

                    // Sample the gradient at this position
                    let color = g.at(clamped_proj);
                    image::Rgba(color.to_rgba8())
                })
            }
            GradientType::Radial => {
                // Default center is middle of image
                let (center_x, center_y) = gradient_config.center.unwrap_or((0.5, 0.5));

                // Convert center from 0-1 range to pixel coordinates
                let center_x_px = center_x * width as f32;
                let center_y_px = center_y * height as f32;

                // Calculate the max distance from center to any corner
                let corner_distances = [
                    ((0.0 - center_x_px).powi(2) + (0.0 - center_y_px).powi(2)).sqrt(),
                    ((width as f32 - center_x_px).powi(2) + (0.0 - center_y_px).powi(2)).sqrt(),
                    ((0.0 - center_x_px).powi(2) + (height as f32 - center_y_px).powi(2)).sqrt(),
                    ((width as f32 - center_x_px).powi(2) + (height as f32 - center_y_px).powi(2))
                        .sqrt(),
                ];

                // Maximum distance to corner
                let max_dist = corner_distances.iter().cloned().fold(0.0, f32::max);

                // Use provided radius or default to max distance
                let gradient_radius = gradient_config.radius.unwrap_or(1.0) * max_dist;

                image::ImageBuffer::from_fn(width, height, |x, y| {
                    // Calculate distance from center
                    let dx = x as f32 - center_x_px;
                    let dy = y as f32 - center_y_px;
                    let distance = (dx * dx + dy * dy).sqrt();

                    // Normalize distance to 0-1 range
                    let normalized_dist = (distance / gradient_radius).clamp(0.0, 1.0);

                    // Sample the gradient at this normalized distance
                    let color = g.at(normalized_dist);
                    image::Rgba(color.to_rgba8())
                })
            }
        }
    }

    pub fn from_hex(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).unwrap()
        } else {
            255
        };

        Color::Custom([
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        ])
    }

    // Helper to convert a gradient to a WGPU texture
    pub fn create_gradient_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gradient_config: BackgroundGradientConfig,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        // Generate the gradient image
        let gradient_image = Self::generate_2d_gradient_texture(gradient_config, width, height);

        // Create a texture with the gradient
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Gradient Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Write the gradient data to the texture
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &gradient_image.into_raw(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        // Create a texture view
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        (texture, texture_view)
    }
}
