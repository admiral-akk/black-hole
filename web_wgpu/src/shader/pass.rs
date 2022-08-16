use wgpu::{BindGroup, Buffer, CommandEncoder, Device, RenderPipeline, TextureView};

pub struct Pass {
    bind_group: Vec<BindGroup>,
}

impl Pass {
    pub fn new() -> Self {
        let bind_group = Vec::new();
        Pass { bind_group }
    }

    pub fn get_encoder(
        &self,
        device: &Device,
        render_pipeline: &RenderPipeline,
        output_texture_view: &TextureView,
        vertex_buffer: &Buffer,
        index_buffer: &Buffer,
        num_indices: u32,
    ) -> CommandEncoder {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(render_pipeline);
            for i in 0..self.bind_group.len() {
                render_pass.set_bind_group(i as u32, &self.bind_group[i], &[]);
            }
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..num_indices, 0, 0..1); // 3.
        }
        encoder
    }
}
