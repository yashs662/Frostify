use crate::{
    ui::{
        Configurable, Positionable, Renderable,
        component::{Component, ComponentBufferData, ComponentConfig, ComponentMetaData},
        layout::Bounds,
    },
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
        let render_data_buffer =
            wgpu_ctx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(format!("{} Render Data Buffer", component.id).as_str()),
                    contents: bytemuck::cast_slice(&[component.get_render_data()]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let bind_group_layout =
            wgpu_ctx
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some(format!("{} Gradient Bind Group Layout", component.id).as_str()),
                });

        let bind_group = wgpu_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: render_data_buffer.as_entire_binding(),
                }],
                label: Some(format!("{} Gradient Bind Group", component.id).as_str()),
            });

        vec![
            ComponentMetaData::BindGroup(bind_group),
            ComponentMetaData::RenderDataBuffer(render_data_buffer),
        ]
    }
}

impl Renderable for BackgroundGradientComponent {
    fn draw(
        component: &mut Component,
        render_pass: &mut wgpu::RenderPass,
        app_pipelines: &mut AppPipelines,
    ) {
        let bind_group = component.get_bind_group();

        if bind_group.is_none() {
            error!(
                "Required resources not found for gradient component id: {}, unable to draw",
                component.id
            );
            return;
        }

        let bind_group = bind_group.unwrap();

        render_pass.set_pipeline(&app_pipelines.color_pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        // Draw a single triangle that covers the whole screen
        render_pass.draw(0..3, 0..1);
    }
}

impl Positionable for BackgroundGradientComponent {
    fn set_position(component: &mut Component, wgpu_ctx: &mut WgpuCtx, bounds: Bounds) {
        let screen_size = wgpu_ctx.get_screen_size();
        let clip_bounds = Component::convert_to_ndc(bounds, screen_size);

        // For SDF rendering, update only the uniform data
        let component_data = ComponentBufferData {
            color: component.get_render_data().color,
            location: [clip_bounds.position.x, clip_bounds.position.y],
            size: [clip_bounds.size.width, clip_bounds.size.height],
        };

        if let Some(render_data_buffer) = component.get_render_data_buffer() {
            wgpu_ctx.queue.write_buffer(
                render_data_buffer,
                0,
                bytemuck::cast_slice(&[component_data]),
            );
        }
    }
}
