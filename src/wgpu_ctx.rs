use crate::{
    ui::{
        component::ComponentType,
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
    pub blit_pipeline: Option<wgpu::RenderPipeline>,
    pub blit_bind_group_layout: Option<wgpu::BindGroupLayout>,
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
    blit_sampler: Option<wgpu::Sampler>,
    blit_bind_group: Option<wgpu::BindGroup>,
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
            app_pipelines: AppPipelines {
                unified_pipeline,
                blit_pipeline: None,
                blit_bind_group_layout: None,
            },
            main_render_texture: None,
            main_render_view: None,
            frame_sample_texture: None,
            frame_sample_view: None,
            blit_sampler: None,
            blit_bind_group: None,
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
        self.blit_bind_group = None;
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

        self.render(&mut encoder, &surface_view, layout_context);

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        final_surface_view: &wgpu::TextureView,
        layout_context: &mut LayoutContext,
    ) {
        // Get the main render view and render order
        let main_render_view = match &self.main_render_view {
            Some(view) => view,
            None => return, // Can't render without a view
        };

        let render_groups = prepare_render_groups(layout_context);

        for (idx, (render_group, (frosted_group, text_group))) in render_groups.iter().enumerate() {
            // First render pass should clear the main render target
            let load_op = if idx == 0 {
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
            } else {
                wgpu::LoadOp::Load
            };

            if *frosted_group {
                if idx != 0 {
                    self.capture_current_frame_texture(encoder);
                }

                for (frosted_idx, frosted_component) in render_group.iter().enumerate() {
                    if let Some(component) = layout_context.get_component_mut(frosted_component) {
                        if let Some(frame_sample_view) = &self.frame_sample_view {
                            FrostedGlassComponent::update_with_frame_texture(
                                component,
                                &self.device,
                                frame_sample_view,
                            );
                        }

                        {
                            let mut render_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("Progressive Render Pass"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: main_render_view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: if frosted_idx == 0 && idx == 0 {
                                                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                                            } else {
                                                wgpu::LoadOp::Load
                                            },
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
                                frosted_component,
                            );
                        }

                        if frosted_idx != render_group.len() - 1 {
                            self.capture_current_frame_texture(encoder);
                        }
                    }
                }
            } else {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Progressive Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: main_render_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: load_op,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                if *text_group {
                    self.text_handler.render(
                        &self.device,
                        &self.queue,
                        &mut render_pass,
                        render_group.clone(),
                    );
                } else {
                    layout_context.draw_group(
                        &mut render_pass,
                        &mut self.app_pipelines,
                        render_group.clone(),
                    );
                }
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
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
    ) {
        if let Some(main_render_view) = &self.main_render_view {
            // Create a sampler for texture sampling
            let sampler = if let Some(sampler) = &self.blit_sampler {
                sampler
            } else {
                let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("Blit Sampler"),
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                });
                self.blit_sampler = Some(sampler);
                self.blit_sampler.as_ref().unwrap()
            };

            // Create or get the bind group layout
            let bind_group_layout = if let Some(layout) = &self.app_pipelines.blit_bind_group_layout
            {
                layout
            } else {
                let layout =
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
                                    ty: wgpu::BindingType::Sampler(
                                        wgpu::SamplerBindingType::Filtering,
                                    ),
                                    count: None,
                                },
                            ],
                            label: Some("texture_bind_group_layout"),
                        });
                self.app_pipelines.blit_bind_group_layout = Some(layout);
                self.app_pipelines.blit_bind_group_layout.as_ref().unwrap()
            };

            // Create or get the blit pipeline
            let blit_pipeline = if let Some(pipeline) = &self.app_pipelines.blit_pipeline {
                pipeline
            } else {
                // Create shader and pipeline as before
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
                            bind_group_layouts: &[bind_group_layout],
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
                self.app_pipelines.blit_pipeline = Some(blit_pipeline);
                self.app_pipelines.blit_pipeline.as_ref().unwrap()
            };

            // Create a bind group for texture sampling
            let bind_group = if let Some(bind_group) = &self.blit_bind_group {
                bind_group
            } else {
                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(main_render_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                    ],
                    label: Some("texture_bind_group"),
                });
                self.blit_bind_group = Some(bind_group);
                self.blit_bind_group.as_ref().unwrap()
            };

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

            render_pass.set_pipeline(blit_pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.draw(0..6, 0..1); // Draw two triangles (a quad)
        }
    }
}

fn prepare_render_groups(
    layout_context: &mut LayoutContext,
) -> Vec<(Vec<uuid::Uuid>, (bool, bool))> {
    let render_order = layout_context.get_render_order().clone();

    // (Vec<Uuid>, is_frosted_glass, is_text)
    let mut render_groups = Vec::new();
    let mut sub_render_group = Vec::new();
    let mut frosted_group = false;
    let mut text_group = false;

    for component_id in render_order {
        if let Some(component) = layout_context.get_component(&component_id) {
            if component.component_type == ComponentType::Container {
                // Containers are not rendered directly, so we skip them
                continue;
            }

            let is_frosted = component.is_frosted_component();
            let is_text = component.is_text_component();

            // If component type changes, push current group and start a new one
            if (is_frosted && !frosted_group)
                || (is_text && !text_group)
                || (!is_frosted && !is_text && (frosted_group || text_group))
            {
                // Only push if we have components in the current group
                if !sub_render_group.is_empty() {
                    render_groups.push((sub_render_group, (frosted_group, text_group)));
                    sub_render_group = Vec::new();
                }

                // Update flags for the new group
                frosted_group = is_frosted;
                text_group = is_text;
            }

            // Add component to current group
            sub_render_group.push(component_id);
        }
    }

    // Don't forget to add the final group if it contains components
    if !sub_render_group.is_empty() {
        render_groups.push((sub_render_group, (frosted_group, text_group)));
    }
    render_groups
}
