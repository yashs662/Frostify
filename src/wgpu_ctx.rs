use crate::{
    constants::UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
    text_renderer::TextHandler,
    ui::{
        components::core::frosted_glass::FrostedGlassComponent,
        layout::{ComponentSize, LayoutContext},
    },
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
    frame_capture_texture: Option<wgpu::Texture>, // For capturing frames (as render target)
    frame_capture_view: Option<wgpu::TextureView>,
    frame_sample_texture: Option<wgpu::Texture>, // For sampling from captured frames (as resource)
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
            frame_capture_texture: None,
            frame_capture_view: None,
            frame_sample_texture: None,
            frame_sample_view: None,
        }
    }

    pub fn new(window: Arc<Window>) -> WgpuCtx<'window> {
        pollster::block_on(WgpuCtx::new_async(window))
    }

    fn ensure_frame_textures(&mut self) {
        // Check if we need to create or resize the textures
        let needs_new_textures = match (&self.frame_capture_texture, &self.frame_sample_texture) {
            (None, _) | (_, None) => true,
            (Some(capture_tex), Some(sample_tex)) => {
                let capture_size = capture_tex.size();
                let sample_size = sample_tex.size();
                capture_size.width != self.surface_config.width
                    || capture_size.height != self.surface_config.height
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

            // Create a texture for capturing frames (as render target only)
            // Use a high-quality format that matches the surface
            let frame_capture_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Frame Capture Texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.surface_config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });

            // Create a separate texture for sampling (as resource only)
            // Enable mipmapping for better filtering
            let frame_sample_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Frame Sample Texture"),
                size: texture_size,
                mip_level_count: Self::calculate_mip_level_count(
                    self.surface_config.width,
                    self.surface_config.height,
                ),
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.surface_config.format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            let frame_capture_view =
                frame_capture_texture.create_view(&wgpu::TextureViewDescriptor::default());
            let frame_sample_view =
                frame_sample_texture.create_view(&wgpu::TextureViewDescriptor {
                    base_mip_level: 0,
                    mip_level_count: None, // Use all mip levels
                    ..Default::default()
                });

            self.frame_capture_texture = Some(frame_capture_texture);
            self.frame_capture_view = Some(frame_capture_view);
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

        // Reset frame textures to be recreated at the right size
        self.frame_capture_texture = None;
        self.frame_capture_view = None;
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
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Ensure we have frame textures for glass effects
        self.ensure_frame_textures();

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Get the render order and identify which components need frame capture
        let components_needing_capture = layout_context.get_frame_capture_components();

        if components_needing_capture.is_empty() {
            // No frosted glass components, proceed with standard rendering
            self.standard_render_pass(&mut encoder, &texture_view, layout_context);
        } else {
            // We have frosted glass components, render in stages
            self.staged_render_pass(
                &mut encoder,
                &texture_view,
                layout_context,
                components_needing_capture,
            );
        }

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }

    fn standard_render_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        texture_view: &wgpu::TextureView,
        layout_context: &mut LayoutContext,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: texture_view,
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

        layout_context.draw(&mut render_pass, &mut self.app_pipelines);
        self.text_handler.render(
            &self.device,
            &self.queue,
            &mut render_pass,
            layout_context.get_render_order().to_vec(),
        );
    }

    fn staged_render_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        final_texture_view: &wgpu::TextureView,
        layout_context: &mut LayoutContext,
        components_needing_capture: Vec<(usize, uuid::Uuid)>,
    ) {
        let frame_capture_view = self.frame_capture_view.as_ref().unwrap();
        let frame_sample_view = self.frame_sample_view.as_ref().unwrap();

        // Get the entire render order
        let render_order = layout_context.get_render_order().clone();
        let mut last_idx = 0;

        // Group adjacent frosted glass components to minimize render passes
        let mut grouped_components: Vec<(usize, usize, Vec<uuid::Uuid>)> = Vec::new();
        let mut current_group_start = None;
        let mut current_group_ids = Vec::new();

        // First, group adjacent frosted glass components
        for (idx, component_id) in components_needing_capture.iter() {
            if current_group_start.is_none() {
                // Start a new group
                current_group_start = Some(*idx);
                current_group_ids.push(*component_id);
            } else if *idx == current_group_start.unwrap() + current_group_ids.len() {
                // Adjacent component, add to current group
                current_group_ids.push(*component_id);
            } else {
                // Non-adjacent component, finish current group and start a new one
                grouped_components.push((
                    current_group_start.unwrap(),
                    current_group_start.unwrap() + current_group_ids.len(),
                    std::mem::take(&mut current_group_ids),
                ));
                current_group_start = Some(*idx);
                current_group_ids.push(*component_id);
            }
        }

        // Add the final group if it exists
        if !current_group_ids.is_empty() {
            grouped_components.push((
                current_group_start.unwrap(),
                current_group_start.unwrap() + current_group_ids.len(),
                current_group_ids,
            ));
        }

        // Process each group
        for (start_idx, end_idx, component_ids) in grouped_components {
            if start_idx > last_idx {
                // Step 1: Render components up to this group to the capture texture
                {
                    let mut capture_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Capture Frame Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: frame_capture_view,
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

                    // Draw all components up to this point
                    layout_context.draw_range(
                        &mut capture_pass,
                        &mut self.app_pipelines,
                        last_idx,
                        start_idx,
                    );
                    let components_to_render = render_order[last_idx..start_idx].to_vec();
                    self.text_handler.render(
                        &self.device,
                        &self.queue,
                        &mut capture_pass,
                        components_to_render,
                    );
                }

                // Step 2: Copy the captured frame to the sample texture
                if let (Some(src_texture), Some(dst_texture)) =
                    (&self.frame_capture_texture, &self.frame_sample_texture)
                {
                    let texture_size = wgpu::Extent3d {
                        width: self.surface_config.width,
                        height: self.surface_config.height,
                        depth_or_array_layers: 1,
                    };

                    encoder.copy_texture_to_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: src_texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::TexelCopyTextureInfo {
                            texture: dst_texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        texture_size,
                    );
                }

                // Step 3: Render the same components to final texture (for actual display)
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render to Final - Background"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: final_texture_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: if last_idx == 0 {
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

                    layout_context.draw_range(
                        &mut render_pass,
                        &mut self.app_pipelines,
                        last_idx,
                        start_idx,
                    );

                    let components_to_render = render_order[last_idx..start_idx].to_vec();
                    self.text_handler.render(
                        &self.device,
                        &self.queue,
                        &mut render_pass,
                        components_to_render,
                    );
                }
            }

            // Step 4: Update all frosted glass components in this group with the sample texture view
            for component_id in &component_ids {
                if let Some(component) = layout_context.get_component_mut(component_id) {
                    FrostedGlassComponent::update_with_frame_texture(
                        component,
                        &self.device,
                        frame_sample_view, // Use the sample texture view
                    );
                }
            }

            // Step 5: Render all frosted glass components in this group to the final texture
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render to Final - Glass Components"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: final_texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load, // Load the existing content
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                // Draw all frosted glass components in this group
                for component_id in &component_ids {
                    layout_context.draw_single(
                        &mut render_pass,
                        &mut self.app_pipelines,
                        component_id,
                    );
                }
            }

            last_idx = end_idx;
        }

        // If there are components after the last frosted glass component, render them too
        if last_idx < render_order.len() {
            let mut final_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Final Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: final_texture_view,
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

            // Draw the remaining components
            layout_context.draw_range(
                &mut final_pass,
                &mut self.app_pipelines,
                last_idx,
                render_order.len(),
            );
            let components_to_render = render_order[last_idx..].to_vec();
            self.text_handler.render(
                &self.device,
                &self.queue,
                &mut final_pass,
                components_to_render,
            );
        }
    }
}

fn create_unified_pipeline(
    device: &wgpu::Device,
    swap_chain_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    // Create unified shader for both color and texture
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Unified Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../assets/shaders/color.wgsl").into()),
    });

    // Create unified bind group layout that supports both color-only and texture rendering
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
        label: Some("Unified Bind Group Layout"),
    });

    // Pipeline layout that works for both standard components and frosted glass
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Unified Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Unified Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[], // No vertex buffers needed for full-screen triangle approach
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: swap_chain_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
    })
}
