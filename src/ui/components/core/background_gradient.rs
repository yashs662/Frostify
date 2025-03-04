use crate::{
    color::Color,
    ui::{
        Configurable, Positionable, Renderable,
        component::{Component, ComponentConfig, ComponentMetaData},
        layout::Bounds,
    },
    vertex::Vertex,
    wgpu_ctx::{AppPipelines, WgpuCtx},
};
use log::error;
use wgpu::util::DeviceExt;

pub struct BackgroundGradientComponent;

impl Configurable for BackgroundGradientComponent {
    fn configure(
        component: &mut Component,
        config: ComponentConfig,
        wgpu_ctx: &mut WgpuCtx,
    ) -> Vec<ComponentMetaData> {
        let gradient_config = config.get_gradient_config().unwrap();

        // Create initial vertices with gradient colors
        let vertices = create_gradient_vertices(
            Bounds::default(),
            gradient_config.start_color,
            gradient_config.end_color,
            gradient_config.angle,
        );
        let indices = component.get_indices();

        let vertex_buffer = wgpu_ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(format!("{} Gradient Vertex Buffer", component.id).as_str()),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let index_buffer = wgpu_ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(format!("{} Gradient Index Buffer", component.id).as_str()),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let bind_group_layout =
            wgpu_ctx
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[],
                    label: Some(format!("{} Gradient Bind Group Layout", component.id).as_str()),
                });

        let bind_group = wgpu_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[],
                label: Some(format!("{} Gradient Bind Group", component.id).as_str()),
            });

        vec![
            ComponentMetaData::VertexBuffer(vertex_buffer),
            ComponentMetaData::IndexBuffer(index_buffer),
            ComponentMetaData::BindGroup(bind_group),
        ]
    }
}

impl Renderable for BackgroundGradientComponent {
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
                "Required resources not found for gradient component id: {}, unable to draw",
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

impl Positionable for BackgroundGradientComponent {
    fn set_position(component: &mut Component, wgpu_ctx: &mut WgpuCtx, bounds: Bounds) {
        let screen_size = wgpu_ctx.get_screen_size();
        let clip_bounds = component.convert_to_ndc(bounds, screen_size);

        if let Some(config) = &component.config {
            if let Some(gradient_config) = config.clone().get_gradient_config() {
                let vertices = create_gradient_vertices(
                    clip_bounds,
                    gradient_config.start_color,
                    gradient_config.end_color,
                    gradient_config.angle,
                );

                if let Some(vertex_buffer) = component.get_vertex_buffer() {
                    wgpu_ctx
                        .queue
                        .write_buffer(vertex_buffer, 0, bytemuck::cast_slice(&vertices));
                }
            }
        }
    }
}

fn create_gradient_vertices(
    bounds: Bounds,
    start_color: Color,
    end_color: Color,
    angle_degrees: f32,
) -> Vec<Vertex> {
    let angle_rad = angle_degrees.to_radians();

    // Calculate gradient direction vector
    let (dx, dy) = (angle_rad.cos(), angle_rad.sin());

    // Normalize coordinates
    let left = bounds.position.x;
    let right = left + bounds.size.width;
    let top = bounds.position.y;
    let bottom = top - bounds.size.height;

    // Calculate gradient progress for each corner
    let get_gradient_factor = |x: f32, y: f32| -> f32 {
        let normalized_pos = dx * x + dy * y;
        (normalized_pos - bounds.position.x) / bounds.size.width
    };

    // Create vertices with interpolated colors
    vec![
        // Top-left
        create_gradient_vertex(
            [left, top],
            get_gradient_factor(left, top),
            start_color,
            end_color,
        ),
        // Top-right
        create_gradient_vertex(
            [right, top],
            get_gradient_factor(right, top),
            start_color,
            end_color,
        ),
        // Bottom-right
        create_gradient_vertex(
            [right, bottom],
            get_gradient_factor(right, bottom),
            start_color,
            end_color,
        ),
        // Bottom-left
        create_gradient_vertex(
            [left, bottom],
            get_gradient_factor(left, bottom),
            start_color,
            end_color,
        ),
    ]
}

fn create_gradient_vertex(
    position: [f32; 2],
    factor: f32,
    start_color: Color,
    end_color: Color,
) -> Vertex {
    let factor = factor.clamp(0.0, 1.0);
    let start_color = start_color.value();
    let end_color = end_color.value();
    let interpolated_color = Color::Custom([
        (start_color[0] + (end_color[0] - start_color[0]) * factor),
        (start_color[1] + (end_color[1] - start_color[1]) * factor),
        (start_color[2] + (end_color[2] - start_color[2]) * factor),
        (start_color[3] + (end_color[3] - start_color[3]) * factor),
    ]);

    Vertex::new(
        [position[0], position[1], 0.0],
        interpolated_color.value(),
        [0.0, 0.0], // UV coordinates not used for gradients
    )
}
