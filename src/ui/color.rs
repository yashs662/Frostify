use crate::ui::ecs::{GradientType, builders::background::BackgroundGradientConfig};
use palette::{Alpha, Darken, Lighten, Mix, Srgb, Srgba, named};

macro_rules! define_colors {
    ($(($variant:ident, $named_variant:ident)),* $(,)?) => {
        #[allow(dead_code)]
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum Color {
            /// A custom color with RGBA values
            Custom(Srgba<f32>),
            /// Fully transparent color
            Transparent,
            $(
                #[doc = concat!("<div style=\"display: inline-block; width: 3em; height: 1em; border: 1px solid black; background: ", stringify!($named_variant), ";\"></div>")]
                $variant,
            )*
        }

        impl Color {
            pub fn to_srgba_f32(self) -> Srgba<f32> {
                match self {
                    $(Color::$variant => {
                        let srgb = Srgb::<f32>::from_format(named::$named_variant).into_linear();
                        Srgba::new(srgb.red, srgb.green, srgb.blue, 1.0)
                    },)*
                    Color::Custom(color) => color,
                    Color::Transparent => Srgba::new(0.0, 0.0, 0.0, 0.0),
                }
            }
        }
    };
}

define_colors! {
    (AliceBlue, ALICEBLUE),
    (AntiqueWhite, ANTIQUEWHITE),
    (Aqua, AQUA),
    (Aquamarine, AQUAMARINE),
    (Azure, AZURE),
    (Beige, BEIGE),
    (Bisque, BISQUE),
    (Black, BLACK),
    (BlanchedAlmond, BLANCHEDALMOND),
    (Blue, BLUE),
    (BlueViolet, BLUEVIOLET),
    (Brown, BROWN),
    (BurlyWood, BURLYWOOD),
    (CadetBlue, CADETBLUE),
    (Chartreuse, CHARTREUSE),
    (Chocolate, CHOCOLATE),
    (Coral, CORAL),
    (CornflowerBlue, CORNFLOWERBLUE),
    (Cornsilk, CORNSILK),
    (Crimson, CRIMSON),
    (Cyan, CYAN),
    (DarkBlue, DARKBLUE),
    (DarkCyan, DARKCYAN),
    (DarkGoldenrod, DARKGOLDENROD),
    (DarkGray, DARKGRAY),
    (DarkGreen, DARKGREEN),
    (DarkGrey, DARKGREY),
    (DarkKhaki, DARKKHAKI),
    (DarkMagenta, DARKMAGENTA),
    (DarkOliveGreen, DARKOLIVEGREEN),
    (DarkOrange, DARKORANGE),
    (DarkOrchid, DARKORCHID),
    (DarkRed, DARKRED),
    (DarkSalmon, DARKSALMON),
    (DarkSeaGreen, DARKSEAGREEN),
    (DarkSlateBlue, DARKSLATEBLUE),
    (DarkSlateGray, DARKSLATEGRAY),
    (DarkSlateGrey, DARKSLATEGREY),
    (DarkTurquoise, DARKTURQUOISE),
    (DarkViolet, DARKVIOLET),
    (DeepPink, DEEPPINK),
    (DeepSkyBlue, DEEPSKYBLUE),
    (DimGray, DIMGRAY),
    (DimGrey, DIMGREY),
    (DodgerBlue, DODGERBLUE),
    (FireBrick, FIREBRICK),
    (FloralWhite, FLORALWHITE),
    (ForestGreen, FORESTGREEN),
    (Fuchsia, FUCHSIA),
    (Gainsboro, GAINSBORO),
    (GhostWhite, GHOSTWHITE),
    (Gold, GOLD),
    (Goldenrod, GOLDENROD),
    (Gray, GRAY),
    (Grey, GREY),
    (Green, GREEN),
    (GreenYellow, GREENYELLOW),
    (Honeydew, HONEYDEW),
    (HotPink, HOTPINK),
    (IndianRed, INDIANRED),
    (Indigo, INDIGO),
    (Ivory, IVORY),
    (Khaki, KHAKI),
    (Lavender, LAVENDER),
    (LavenderBlush, LAVENDERBLUSH),
    (LawnGreen, LAWNGREEN),
    (LemonChiffon, LEMONCHIFFON),
    (LightBlue, LIGHTBLUE),
    (LightCoral, LIGHTCORAL),
    (LightCyan, LIGHTCYAN),
    (LightGoldenrodYellow, LIGHTGOLDENRODYELLOW),
    (LightGray, LIGHTGRAY),
    (LightGreen, LIGHTGREEN),
    (LightGrey, LIGHTGREY),
    (LightPink, LIGHTPINK),
    (LightSalmon, LIGHTSALMON),
    (LightSeaGreen, LIGHTSEAGREEN),
    (LightSkyBlue, LIGHTSKYBLUE),
    (LightSlateGray, LIGHTSLATEGRAY),
    (LightSlateGrey, LIGHTSLATEGREY),
    (LightSteelBlue, LIGHTSTEELBLUE),
    (LightYellow, LIGHTYELLOW),
    (Lime, LIME),
    (LimeGreen, LIMEGREEN),
    (Linen, LINEN),
    (Magenta, MAGENTA),
    (Maroon, MAROON),
    (MediumAquamarine, MEDIUMAQUAMARINE),
    (MediumBlue, MEDIUMBLUE),
    (MediumOrchid, MEDIUMORCHID),
    (MediumPurple, MEDIUMPURPLE),
    (MediumSeaGreen, MEDIUMSEAGREEN),
    (MediumSlateBlue, MEDIUMSLATEBLUE),
    (MediumSpringGreen, MEDIUMSPRINGGREEN),
    (MediumTurquoise, MEDIUMTURQUOISE),
    (MediumVioletRed, MEDIUMVIOLETRED),
    (MidnightBlue, MIDNIGHTBLUE),
    (MintCream, MINTCREAM),
    (MistyRose, MISTYROSE),
    (Moccasin, MOCCASIN),
    (NavajoWhite, NAVAJOWHITE),
    (Navy, NAVY),
    (OldLace, OLDLACE),
    (Olive, OLIVE),
    (OliveDrab, OLIVEDRAB),
    (Orange, ORANGE),
    (OrangeRed, ORANGERED),
    (Orchid, ORCHID),
    (PaleGoldenrod, PALEGOLDENROD),
    (PaleGreen, PALEGREEN),
    (PaleTurquoise, PALETURQUOISE),
    (PaleVioletRed, PALEVIOLETRED),
    (PapayaWhip, PAPAYAWHIP),
    (PeachPuff, PEACHPUFF),
    (Peru, PERU),
    (Pink, PINK),
    (Plum, PLUM),
    (PowderBlue, POWDERBLUE),
    (Purple, PURPLE),
    (RebeccaPurple, REBECCAPURPLE),
    (Red, RED),
    (RosyBrown, ROSYBROWN),
    (RoyalBlue, ROYALBLUE),
    (SaddleBrown, SADDLEBROWN),
    (Salmon, SALMON),
    (SandyBrown, SANDYBROWN),
    (SeaGreen, SEAGREEN),
    (Seashell, SEASHELL),
    (Sienna, SIENNA),
    (Silver, SILVER),
    (SkyBlue, SKYBLUE),
    (SlateBlue, SLATEBLUE),
    (SlateGray, SLATEGRAY),
    (SlateGrey, SLATEGREY),
    (Snow, SNOW),
    (SpringGreen, SPRINGGREEN),
    (SteelBlue, STEELBLUE),
    (Tan, TAN),
    (Teal, TEAL),
    (Thistle, THISTLE),
    (Tomato, TOMATO),
    (Turquoise, TURQUOISE),
    (Violet, VIOLET),
    (Wheat, WHEAT),
    (White, WHITE),
    (WhiteSmoke, WHITESMOKE),
    (Yellow, YELLOW),
    (YellowGreen, YELLOWGREEN),
}

#[allow(dead_code)]
impl Color {
    pub fn lerp(&self, other: &Color, t: f32) -> Color {
        let mixed = self.to_srgba_f32().mix(other.to_srgba_f32(), t);
        Color::Custom(mixed)
    }

    pub fn to_cosmic_color(self) -> cosmic_text::Color {
        let [r, g, b, a] = self.values_u8();
        cosmic_text::Color::rgba(r, g, b, a)
    }

    pub fn lighten(self, factor: f32) -> Color {
        self.to_srgba_f32().lighten(factor).into()
    }

    pub fn darken(self, factor: f32) -> Color {
        self.to_srgba_f32().darken(factor).into()
    }

    pub fn values_f32(&self) -> [f32; 4] {
        let color = self.to_srgba_f32();
        [color.red, color.green, color.blue, color.alpha]
    }

    pub fn values_u8(&self) -> [u8; 4] {
        let color = self.to_srgba_f32().into_format();
        [color.red, color.green, color.blue, color.alpha]
    }

    /// Method to create a 2D gradient texture
    pub fn generate_2d_gradient_texture(
        gradient_config: BackgroundGradientConfig,
        width: u32,
        height: u32,
    ) -> image::RgbaImage {
        let palette_colors: Vec<(f32, Srgba<f32>)> = gradient_config
            .color_stops
            .iter()
            .map(|stop| (stop.position, stop.color.to_srgba_f32()))
            .collect();

        // Helper function to interpolate between colors in a gradient
        let interpolate_gradient = |t: f32| -> Srgba<f32> {
            let t = t.clamp(0.0, 1.0);

            // Find the two colors to interpolate between
            for i in 0..palette_colors.len() - 1 {
                let (pos1, color1) = palette_colors[i];
                let (pos2, color2) = palette_colors[i + 1];

                if t <= pos2 {
                    if pos1 == pos2 {
                        return color1;
                    }
                    let local_t = (t - pos1) / (pos2 - pos1);
                    return color1.mix(color2, local_t);
                }
            }

            // If we're beyond the last position, return the last color
            palette_colors.last().unwrap().1
        };

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
                    let color = interpolate_gradient(clamped_proj);
                    image::Rgba([
                        (color.red * 255.0) as u8,
                        (color.green * 255.0) as u8,
                        (color.blue * 255.0) as u8,
                        (color.alpha * 255.0) as u8,
                    ])
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
                    let color = interpolate_gradient(normalized_dist);
                    image::Rgba([
                        (color.red * 255.0) as u8,
                        (color.green * 255.0) as u8,
                        (color.blue * 255.0) as u8,
                        (color.alpha * 255.0) as u8,
                    ])
                })
            }
        }
    }

    /// Helper to convert a gradient to a WGPU texture
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

impl From<Alpha<palette::rgb::Rgb, f32>> for Color {
    fn from(val: Alpha<palette::rgb::Rgb, f32>) -> Self {
        Color::Custom(Srgba::new(val.red, val.green, val.blue, val.alpha))
    }
}
