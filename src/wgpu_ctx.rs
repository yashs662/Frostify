use crate::{
    constants::UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
    ui::{
        ecs::{
            EntityId, World,
            components::{
                BoundsComponent, InteractionComponent, RenderDataComponent, VisualComponent,
            },
            resources::RenderGroupsResource,
            systems::render::RenderPrepareSystem,
        },
        layout::Size,
    },
    utils::create_unified_pipeline,
};
use std::sync::Arc;
use wgpu::{MemoryHints::Performance, Texture};
use winit::window::Window;

pub struct AppPipelines {
    pub unified_pipeline: wgpu::RenderPipeline,
    pub blit_pipeline: Option<wgpu::RenderPipeline>,
    pub blit_bind_group_layout: Option<wgpu::BindGroupLayout>,
}

pub struct WgpuCtx<'window> {
    surface: Option<wgpu::Surface<'window>>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub app_pipelines: AppPipelines,
    // Main render texture for all drawing operations
    main_render_texture: Option<wgpu::Texture>,
    main_render_view: Option<wgpu::TextureView>,
    // Texture that can be sampled for frosted glass effects
    frame_sample_texture: Option<wgpu::Texture>,
    frame_sample_view: Option<wgpu::TextureView>,
    blit_sampler: Option<wgpu::Sampler>,
    blit_bind_group: Option<wgpu::BindGroup>,
    pub unified_bind_group_layout: wgpu::BindGroupLayout,
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
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to create device");

        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync, // TODO: Allow the user to configure this in future
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // Create unified bind group layout compatible with the shader
        let unified_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
                label: Some("Unified Bind Group Layout"),
            });

        let unified_pipeline =
            create_unified_pipeline(&device, surface_config.format, &unified_bind_group_layout);
        WgpuCtx {
            surface: Some(surface),
            surface_config,
            device,
            queue,
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
            unified_bind_group_layout,
        }
    }

    #[cfg(test)]
    /// no-op context for testing purposes
    pub async fn new_noop() -> WgpuCtx<'window> {
        use winit::dpi::PhysicalSize;

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::NOOP,
            backend_options: wgpu::BackendOptions {
                noop: wgpu::NoopBackendOptions { enable: true },
                ..Default::default()
            },
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("adapter");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("device");

        let size: PhysicalSize<u32> = (100, 100).into(); // Dummy size for no-op context
        let width = size.width.max(1);
        let height = size.height.max(1);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // Create unified bind group layout compatible with the shader
        let unified_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
                label: Some("Unified Bind Group Layout"),
            });

        let unified_pipeline =
            create_unified_pipeline(&device, surface_config.format, &unified_bind_group_layout);

        WgpuCtx {
            surface: None,
            surface_config,
            device,
            queue,
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
            unified_bind_group_layout,
        }
    }

    pub fn new(window: Arc<Window>) -> WgpuCtx<'window> {
        pollster::block_on(WgpuCtx::new_async(window))
    }

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

            // Create textures
            self.create_main_render_texture(texture_size);
            self.create_frame_sample_texture(texture_size);

            // Reset the blit bind group since we have new texture views
            self.blit_bind_group = None;
        }
    }

    fn create_main_render_texture(&mut self, texture_size: wgpu::Extent3d) {
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

        let main_render_view =
            main_render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.main_render_texture = Some(main_render_texture);
        self.main_render_view = Some(main_render_view);
    }

    fn create_frame_sample_texture(&mut self, texture_size: wgpu::Extent3d) {
        let frame_sample_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Frame Sample Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.surface_config.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let frame_sample_view =
            frame_sample_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.frame_sample_texture = Some(frame_sample_texture);
        self.frame_sample_view = Some(frame_sample_view);
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        let (width, height) = new_size;
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        if let Some(surface) = &self.surface {
            surface.configure(&self.device, &self.surface_config);
        }

        // Reset render textures to be recreated at the right size
        self.main_render_texture = None;
        self.main_render_view = None;
        self.frame_sample_texture = None;
        self.frame_sample_view = None;
        self.blit_bind_group = None;
    }

    pub fn get_screen_size(&self) -> Size<u32> {
        Size {
            width: self.surface_config.width,
            height: self.surface_config.height,
        }
    }

    /// Draw the UI using the ECS system approach
    pub fn draw(&mut self, world: &mut World) {
        // check if we have any thing to draw
        if world.is_empty() {
            log::warn!("Tried to draw an empty world");
            return;
        }
        let surface_texture = if let Some(surface) = &self.surface {
            surface
                .get_current_texture()
                .expect("Failed to acquire next swap chain texture")
        } else {
            panic!("Surface not initialized");
        };

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Ensure we have render textures and resources
        self.ensure_render_textures();
        self.ensure_blit_sampler();
        self.ensure_blit_bind_group_layout();
        self.ensure_blit_pipeline();
        self.ensure_blit_bind_group();

        // Create main rendering encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Run RenderPrepareSystem to create render groups
        world.run_system(RenderPrepareSystem);

        // Create a local clone of the device and frame_sample_view
        let device = &self.device;
        let frame_sample_view = self.frame_sample_view.as_ref();
        let main_render_texture_view = self.main_render_view.as_ref().unwrap();
        let main_render_texture = &self.main_render_texture;
        let frame_sample_texture = &self.frame_sample_texture;
        let app_pipelines = &mut self.app_pipelines;
        let surface_width = self.surface_config.width;
        let surface_height = self.surface_config.height;

        // Get the render groups resource and clone it to avoid borrowing issues
        let render_groups = world
            .resources
            .get_resource::<RenderGroupsResource>()
            .cloned()
            .expect("RenderGroupsResource not found")
            .groups;

        // Process each render group
        for (idx, render_group) in render_groups.iter().enumerate() {
            let load_op = if idx == 0 {
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
            } else {
                wgpu::LoadOp::Load
            };

            if render_group.is_frosted_glass {
                // Frosted glass rendering logic
                if idx != 0 {
                    WgpuCtx::capture_current_frame_texture(
                        main_render_texture,
                        frame_sample_texture,
                        surface_width,
                        surface_height,
                        &mut encoder,
                    );
                }

                for (frosted_idx, entity_id) in render_group.entity_ids.iter().enumerate() {
                    // Create a render pass for this frosted glass component
                    {
                        if let Some(frame_view) = frame_sample_view {
                            update_frosted_glass_with_frame_texture(
                                world,
                                *entity_id,
                                frame_view,
                                device,
                                &self.unified_bind_group_layout,
                            );
                        }

                        let mut render_pass =
                            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("Frosted Glass Render Pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: main_render_texture_view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: if frosted_idx == 0 && idx == 0 {
                                            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                                        } else {
                                            wgpu::LoadOp::Load
                                        },
                                        store: wgpu::StoreOp::Store,
                                    },
                                    depth_slice: None,
                                })],
                                depth_stencil_attachment: None,
                                occlusion_query_set: None,
                                timestamp_writes: None,
                            });
                        render_pass.set_pipeline(&app_pipelines.unified_pipeline);

                        // Extract entity data
                        if let (Some(_bounds), Some(render_data)) = (
                            world
                                .components
                                .get_component::<BoundsComponent>(*entity_id),
                            world
                                .components
                                .get_component::<RenderDataComponent>(*entity_id),
                        ) {
                            // Simple rendering
                            render_pass.set_bind_group(
                                0,
                                render_data.bind_group.as_ref().expect(
                                    "Expected bind group to exist for entity before rendering",
                                ),
                                &[],
                            );
                            render_pass.set_vertex_buffer(
                                0,
                                render_data
                                    .vertex_buffer
                                    .as_ref()
                                    .expect(
                                        "expected vertex buffer to be set before rendering entity",
                                    )
                                    .slice(..),
                            );
                            render_pass.set_index_buffer(
                                render_data
                                    .index_buffer
                                    .as_ref()
                                    .expect(
                                        "expected index buffer to be set before rendering entity",
                                    )
                                    .slice(..),
                                wgpu::IndexFormat::Uint16,
                            );
                            render_pass.draw_indexed(0..6, 0, 0..1);
                        }
                    }

                    if render_group.entity_ids.len() > 1
                        && frosted_idx < render_group.entity_ids.len() - 1
                    {
                        WgpuCtx::capture_current_frame_texture(
                            main_render_texture,
                            frame_sample_texture,
                            surface_width,
                            surface_height,
                            &mut encoder,
                        );
                    }
                }
            } else {
                // Normal rendering logic - handle in a single pass
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("ECS Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: main_render_texture_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: load_op,
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                    render_pass.set_pipeline(&app_pipelines.unified_pipeline);
                    // Render each entity manually
                    for &entity_id in &render_group.entity_ids {
                        if let (Some(visual), Some(render_data), Some(interaction)) = (
                            world.components.get_component::<VisualComponent>(entity_id),
                            world
                                .components
                                .get_component::<RenderDataComponent>(entity_id),
                            world
                                .components
                                .get_component::<InteractionComponent>(entity_id),
                        ) {
                            // Skip if not visible or if not active
                            if !visual.is_visible() || !interaction.is_active {
                                continue;
                            }

                            // Simple rendering
                            render_pass.set_bind_group(
                                0,
                                render_data.bind_group.as_ref().expect(
                                    "Expected bind group to exist for entity before rendering",
                                ),
                                &[],
                            );
                            render_pass.set_vertex_buffer(
                                0,
                                render_data
                                    .vertex_buffer
                                    .as_ref()
                                    .expect(
                                        "expected vertex buffer to be set before rendering entity",
                                    )
                                    .slice(..),
                            );
                            render_pass.set_index_buffer(
                                render_data
                                    .index_buffer
                                    .as_ref()
                                    .expect(
                                        "expected index buffer to be set before rendering entity",
                                    )
                                    .slice(..),
                                wgpu::IndexFormat::Uint16,
                            );
                            render_pass.draw_indexed(0..6, 0, 0..1);
                        }
                    }
                }
            }
        }

        // Final blit to screen
        {
            let blit_pipeline = self.app_pipelines.blit_pipeline.as_ref().unwrap();
            let bind_group = self.blit_bind_group.as_ref().unwrap();

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Final Surface Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(blit_pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // Submit main rendering commands
        self.queue.submit(Some(encoder.finish()));

        surface_texture.present();
    }

    fn ensure_blit_sampler(&mut self) {
        if self.blit_sampler.is_some() {
            return;
        }

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Blit Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        self.blit_sampler = Some(sampler);
    }

    fn ensure_blit_bind_group_layout(&mut self) {
        if self.app_pipelines.blit_bind_group_layout.is_some() {
            return;
        }

        let layout = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
        self.app_pipelines.blit_bind_group_layout = Some(layout);
    }

    fn ensure_blit_pipeline(&mut self) {
        if self.app_pipelines.blit_pipeline.is_some() {
            return;
        }

        let bind_group_layout = match &self.app_pipelines.blit_bind_group_layout {
            Some(layout) => layout,
            None => return,
        };

        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Blit Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../assets/shaders/blit.wgsl").into(),
                ),
            });

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Blit Pipeline Layout"),
                bind_group_layouts: &[bind_group_layout],
                push_constant_ranges: &[],
            });

        let blit_pipeline = self
            .device
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
    }

    fn ensure_blit_bind_group(&mut self) {
        if self.blit_bind_group.is_some() {
            return;
        }

        // Make sure we have all required resources
        let texture_view = match &self.main_render_view {
            Some(view) => view,
            None => return,
        };

        let sampler = match &self.blit_sampler {
            Some(sampler) => sampler,
            None => return,
        };

        let bind_group_layout = match &self.app_pipelines.blit_bind_group_layout {
            Some(layout) => layout,
            None => return,
        };

        // Create the bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });
        self.blit_bind_group = Some(bind_group);
    }

    fn capture_current_frame_texture(
        main_render_texture: &Option<Texture>,
        frame_sample_texture: &Option<Texture>,
        width: u32,
        height: u32,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if let (Some(main_tex), Some(frame_tex)) = (main_render_texture, frame_sample_texture) {
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: main_tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: frame_tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
        }
    }
}

fn update_frosted_glass_with_frame_texture(
    world: &mut World,
    entity_id: EntityId,
    frame_view: &wgpu::TextureView,
    device: &wgpu::Device,
    unified_bind_group_layout: &wgpu::BindGroupLayout,
) {
    let render_comp = world
        .components
        .get_component_mut::<RenderDataComponent>(entity_id)
        .expect("expected RenderDataComponent to exist to update frosted glass");

    // Create bind group with all required resources
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: unified_bind_group_layout,
        entries: &[
            // Component uniform data
            wgpu::BindGroupEntry {
                binding: 0,
                resource: render_comp.render_data_buffer.as_entire_binding(),
            },
            // Texture view
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(frame_view),
            },
            // Sampler
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&render_comp.sampler),
            },
        ],
        label: Some(format!("{entity_id} Unified Bind Group").as_str()),
    });

    render_comp.bind_group = Some(bind_group);
}
