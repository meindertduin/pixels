use byteorder::{ByteOrder, LittleEndian};
use std::fmt;
use std::rc::Rc;
use wgpu::{self, TextureView};

use crate::render_pass::{BoxedRenderPass, Device, Queue, RenderPass};

/// Renderer implements [`RenderPass`].
#[derive(Debug)]
pub(crate) struct Renderer {
    device: Rc<wgpu::Device>,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    /// Factory function for generating `RenderPass` trait objects.
    pub(crate) fn factory(
        device: Device,
        _queue: Queue,
        texture_view: &TextureView,
    ) -> BoxedRenderPass {
        let vert_spv = include_bytes!("../shaders/vert.spv");
        let mut vert = Vec::new();
        vert.resize_with(
            vert_spv.len() / std::mem::size_of::<u32>(),
            Default::default,
        );
        LittleEndian::read_u32_into(vert_spv, &mut vert);

        let frag_spv = include_bytes!("../shaders/frag.spv");
        let mut frag = Vec::new();
        frag.resize_with(
            frag_spv.len() / std::mem::size_of::<u32>(),
            Default::default,
        );
        LittleEndian::read_u32_into(frag_spv, &mut frag);

        let vs_module = device.create_shader_module(&vert);
        let fs_module = device.create_shader_module(&frag);

        // Create a texture sampler with nearest neighbor
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            compare_function: wgpu::CompareFunction::Always,
        });

        // Create bind group
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Create pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Box::new(Renderer {
            device,
            bind_group,
            render_pipeline,
        })
    }
}

impl RenderPass for Renderer {
    fn update_bindings(&mut self, _input_texture: &TextureView) {}

    fn render_pass(&self, encoder: &mut wgpu::CommandEncoder, render_target: &TextureView) {
        // Draw the updated texture to the render target
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: render_target,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::BLACK,
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.draw(0..6, 0..1);
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}