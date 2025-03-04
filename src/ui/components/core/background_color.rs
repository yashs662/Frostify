use crate::{
    ui::{
        Configurable, Positionable, Renderable,
        component::{Component, ComponentConfig, ComponentMetaData},
        layout::Bounds,
    },
    wgpu_ctx::{AppPipelines, WgpuCtx},
};
use log::error;
use wgpu::util::DeviceExt;

pub struct BackgroundColorComponent;

impl Configurable for BackgroundColorComponent {
    fn configure(
        component: &mut Component,
        _config: ComponentConfig,
        wgpu_ctx: &mut WgpuCtx,
    ) -> Vec<ComponentMetaData> {
        // Initial vertices with default bounds, will be recalculated on resize
        let screen_size = wgpu_ctx.get_screen_size();
        let vertices = component.calculate_vertices(Some(Bounds::default()), None, screen_size);
        let indices = component.get_indices();

        // Create buffers
        let vertex_buffer = wgpu_ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(format!("{} Vertex Buffer", component.id).as_str()),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let index_buffer = wgpu_ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(format!("{} Index Buffer", component.id).as_str()),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        // Create an empty bind group for solid colors and gradients
        let bind_group_layout =
            wgpu_ctx
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[],
                    label: Some(format!("{} Bind Group Layout", component.id).as_str()),
                });

        let bind_group = wgpu_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[],
                label: Some(format!("{} Bind Group", component.id).as_str()),
            });

        vec![
            ComponentMetaData::VertexBuffer(vertex_buffer),
            ComponentMetaData::IndexBuffer(index_buffer),
            ComponentMetaData::BindGroup(bind_group),
        ]
    }
}

impl Renderable for BackgroundColorComponent {
    fn draw(
        component: &mut Component,
        render_pass: &mut wgpu::RenderPass,
        app_pipelines: &mut AppPipelines,
    ) {
        let indices = component.get_indices();
        let vertex_buffer = component.get_vertex_buffer();
        let index_buffer = component.get_index_buffer();
        let bind_group = component.get_bind_group();

        if vertex_buffer.is_none() || index_buffer.is_none() || bind_group.is_none() {
            error!(
                "Vertex buffer, index buffer, or bind group not found for component id: {}, unable to draw",
                component.id
            );
            return;
        }

        let vertex_buffer = vertex_buffer.unwrap();
        let index_buffer = index_buffer.unwrap();
        let bind_group = bind_group.unwrap();

        render_pass.set_pipeline(&app_pipelines.color_pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }
}

impl Positionable for BackgroundColorComponent {
    fn set_position(component: &mut Component, wgpu_ctx: &mut WgpuCtx, bounds: Bounds) {
        // Convert to NDC space
        let screen_size = wgpu_ctx.get_screen_size();
        let clip_bounds = component.convert_to_ndc(bounds, screen_size);
        let vertices = component.calculate_vertices(Some(clip_bounds), None, screen_size);

        // Update vertex buffer
        if let Some(vertex_buffer) = component.get_vertex_buffer() {
            wgpu_ctx
                .queue
                .write_buffer(vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        }
    }
}
