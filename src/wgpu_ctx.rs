use crate::{
    ui::{
        components::core::frosted_glass::FrostedGlassComponent,
        layout::{ComponentSize, LayoutContext},
        text_renderer::TextHandler,
    },
    utils::create_unified_pipeline,
};
use std::sync::Arc;
use wgpu::MemoryHints::Performance;
use winit::window::Window;

pub struct AppPipelines {
    pub unified_pipeline: wgpu::RenderPipeline,
}

pub struct WgpuCtx<'window> {
    surface: wgpu::Surface<'window>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub text_handler: TextHandler,
    pub app_pipelines: AppPipelines,
    // Main render texture for all drawing operations
    main_render_texture: Option<wgpu::Texture>,
    main_render_view: Option<wgpu::TextureView>,
    // Texture that can be sampled for frosted glass effects
    frame_sample_texture: Option<wgpu::Texture>,
    frame_sample_view: Option<wgpu::TextureView>,
}

impl<'window> WgpuCtx<'window> {
    pub async fn new_async(window: Arc<Window>) -> WgpuCtx<'window> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: Performance,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        // Get the supported alpha modes from the surface capabilities
        let surface_caps = surface.get_capabilities(&adapter);
        let alpha_mode = surface_caps.alpha_modes[0]; // Use the first supported alpha mode

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps.formats[0],
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode, // Use the supported alpha mode instead of hardcoding
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let unified_pipeline = create_unified_pipeline(&device, surface_config.format);
        let text_handler = TextHandler::new(&device, &surface_config, &queue);

        WgpuCtx {
            surface,
            surface_config,
            device,
            queue,
            text_handler,
            app_pipelines: AppPipelines { unified_pipeline },
            main_render_texture: None,
            main_render_view: None,
            frame_sample_texture: None,
            frame_sample_view: None,
        }
    }

    pub fn new(window: Arc<Window>) -> WgpuCtx<'window> {
        pollster::block_on(WgpuCtx::new_async(window))
    }

    // Improved texture creation with better usage flags
    fn ensure_render_textures(&mut self) {
        // Check if we need to create or resize the textures
        let needs_new_textures = match (&self.main_render_texture, &self.frame_sample_texture) {
            (None, _) | (_, None) => true,
            (Some(main_tex), Some(sample_tex)) => {
                let main_size = main_tex.size();
                let sample_size = sample_tex.size();
                main_size.width != self.surface_config.width
                    || main_size.height != self.surface_config.height
                    || sample_size.width != self.surface_config.width
                    || sample_size.height != self.surface_config.height
            }
        };

        if needs_new_textures {
            let texture_size = wgpu::Extent3d {
                width: self.surface_config.width,
                height: self.surface_config.height,
                depth_or_array_layers: 1,
            };

            // Create main render texture with explicit usage flags
            let main_render_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Main Render Texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.surface_config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            // Calculate appropriate mip levels for the frame sample texture
            let mip_level_count = Self::calculate_mip_level_count(
                self.surface_config.width,
                self.surface_config.height,
            );

            // Create a texture for sampling with mipmap support
            let frame_sample_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Frame Sample Texture"),
                size: texture_size,
                mip_level_count,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.surface_config.format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            let main_render_view =
                main_render_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let frame_sample_view =
                frame_sample_texture.create_view(&wgpu::TextureViewDescriptor {
                    base_mip_level: 0,
                    mip_level_count: Some(mip_level_count), // Specify all mip levels explicitly
                    ..Default::default()
                });

            self.main_render_texture = Some(main_render_texture);
            self.main_render_view = Some(main_render_view);
            self.frame_sample_texture = Some(frame_sample_texture);
            self.frame_sample_view = Some(frame_sample_view);
        }
    }

    // Helper function to calculate the optimal number of mip levels for a texture
    fn calculate_mip_level_count(width: u32, height: u32) -> u32 {
        let max_dimension = width.max(height);
        32 - max_dimension.leading_zeros()
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        let (width, height) = new_size;
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
        self.text_handler
            .update_viewport_size(&self.surface_config, &self.queue);

        // Reset render textures to be recreated at the right size
        self.main_render_texture = None;
        self.main_render_view = None;
        self.frame_sample_texture = None;
        self.frame_sample_view = None;
    }

    pub fn get_screen_size(&self) -> ComponentSize {
        ComponentSize {
            width: self.surface_config.width as f32,
            height: self.surface_config.height as f32,
        }
    }

    pub fn draw(&mut self, layout_context: &mut LayoutContext) {
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Ensure we have render textures
        self.ensure_render_textures();

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Get components that need frame capture for frosted glass effect
        let components_needing_capture = layout_context.get_frame_capture_components();

        // Use optimized rendering process with proper texture synchronization
        self.render_with_progressive_updates(
            &mut encoder,
            &surface_view,
            layout_context,
            components_needing_capture,
        );

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }

    fn render_with_progressive_updates(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        final_surface_view: &wgpu::TextureView,
        layout_context: &mut LayoutContext,
        components_needing_capture: Vec<(usize, uuid::Uuid)>,
    ) {
        // Get the main render view and render order
        let main_render_view = match &self.main_render_view {
            Some(view) => view,
            None => return, // Can't render without a view
        };

        let render_order = layout_context.get_render_order().clone();
        let mut last_idx = 0;

        // If no frosted glass components, render everything in one pass
        if components_needing_capture.is_empty() {
            // Single render pass for all components
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: main_render_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Already cleared in draw()
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            layout_context.draw(&mut render_pass, &mut self.app_pipelines);
            self.text_handler.render(
                &self.device,
                &self.queue,
                &mut render_pass,
                render_order.clone(),
            );
        } else {
            // For each frosted glass component or group
            for (idx, component_id) in &components_needing_capture {
                // If there are components to render before this frosted glass component
                if *idx > last_idx {
                    // Render components between last_idx and current index
                    {
                        let mut render_pass =
                            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("Progressive Render Pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: main_render_view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                occlusion_query_set: None,
                                timestamp_writes: None,
                            });

                        layout_context.draw_range(
                            &mut render_pass,
                            &mut self.app_pipelines,
                            last_idx,
                            *idx,
                        );

                        let components_to_render = render_order[last_idx..*idx].to_vec();
                        self.text_handler.render(
                            &self.device,
                            &self.queue,
                            &mut render_pass,
                            components_to_render,
                        );
                    }

                    // Copy the current state to the sample texture and generate mipmaps
                    self.capture_current_frame_texture(encoder);
                }

                // Update the frosted glass component with the current frame texture
                // IMPORTANT: Do this after copying the texture to ensure we have the latest frame
                if let Some(component) = layout_context.get_component_mut(component_id) {
                    if let Some(frame_sample_view) = &self.frame_sample_view {
                        FrostedGlassComponent::update_with_frame_texture(
                            component,
                            &self.device,
                            frame_sample_view,
                        );
                    }
                }

                // Render the frosted glass component
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Glass Component Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: main_render_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                    layout_context.draw_single(
                        &mut render_pass,
                        &mut self.app_pipelines,
                        component_id,
                    );
                }

                self.capture_current_frame_texture(encoder);

                last_idx = *idx + 1;
            }

            // Render any remaining components
            if last_idx < render_order.len() {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Final Components Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: main_render_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                layout_context.draw_range(
                    &mut render_pass,
                    &mut self.app_pipelines,
                    last_idx,
                    render_order.len(),
                );

                let components_to_render = render_order[last_idx..].to_vec();
                self.text_handler.render(
                    &self.device,
                    &self.queue,
                    &mut render_pass,
                    components_to_render,
                );
            }
        }

        // Blit the main texture to the surface using a final render pass
        self.blit_to_surface(encoder, final_surface_view);
    }

    // New helper method to properly capture the current frame and generate mipmaps
    fn capture_current_frame_texture(&self, encoder: &mut wgpu::CommandEncoder) {
        if let (Some(main_texture), Some(sample_texture)) =
            (&self.main_render_texture, &self.frame_sample_texture)
        {
            let texture_size = wgpu::Extent3d {
                width: self.surface_config.width,
                height: self.surface_config.height,
                depth_or_array_layers: 1,
            };

            // Copy from main render texture to frame sample texture (base mip level only)
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: main_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: sample_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                texture_size,
            );
        }
    }

    // New helper method to blit the final texture to the surface
    fn blit_to_surface(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
    ) {
        if let Some(main_render_view) = &self.main_render_view {
            // Create a bind group for texture sampling
            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

            let bind_group_layout =
                self.device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    multisampled: false,
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    sample_type: wgpu::TextureSampleType::Float {
                                        filterable: true,
                                    },
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                            },
                        ],
                        label: Some("texture_bind_group_layout"),
                    });

            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(main_render_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: Some("texture_bind_group"),
            });

            // Create pipeline for blitting (can be cached/stored as a member for better performance)
            let shader = self
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Blit Shader"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("../assets/shaders/blit.wgsl").into(),
                    ),
                });

            let pipeline_layout =
                self.device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Blit Pipeline Layout"),
                        bind_group_layouts: &[&bind_group_layout],
                        push_constant_ranges: &[],
                    });

            let blit_pipeline =
                self.device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("Blit Pipeline"),
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: Some("vs_main"),
                            buffers: &[],
                            compilation_options: Default::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: Some("fs_main"),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: self.surface_config.format,
                                blend: Some(wgpu::BlendState::REPLACE),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                            compilation_options: Default::default(),
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: None,
                            unclipped_depth: false,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            conservative: false,
                        },
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState {
                            count: 1,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        multiview: None,
                        cache: None,
                    });

            // Start the render pass to render to the surface
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Final Surface Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&blit_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..6, 0..1); // Draw two triangles (a quad)
        }
    }
}
