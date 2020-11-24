use crate::graphics::{
    global::Context as GlobalContext, shaders::Shaders,
    COLOR_FORMAT, DEPTH_FORMAT,
};
use crate::utils::*;
use cgmath::prelude::*;

use std::mem;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

pub struct VertexDesc {
    attributes: [wgpu::VertexAttributeDescriptor; 2],
}

impl VertexDesc {
    pub fn new() -> Self {
        VertexDesc {
            attributes: wgpu::vertex_attr_array![0 => Float3, 1 => Float4],
        }
    }

    pub fn buffer_desc(&self) -> wgpu::VertexBufferDescriptor {
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &self.attributes,
        }
    }
}

#[rustfmt::skip]
pub const QUAD_VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, 0.5, 0.0], color: [1.0, 1.0, 1.0, 1.0], },
    Vertex { position: [-0.5, -0.5, 0.0], color: [1.0, 1.0, 1.0, 1.0], },
    Vertex { position: [0.5, -0.5, 0.0], color: [1.0, 1.0, 1.0, 1.0], },
    Vertex { position: [0.5, 0.5, 0.0], color: [1.0, 1.0, 1.0, 1.0], },
];

#[rustfmt::skip]
pub const QUAD_INDICES: &[u16] = &[
    0, 1, 1, 3, 3, 0,
    1, 2, 2, 3,
];

#[rustfmt::skip]
pub const CUBE_VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, 0.5, 0.5], color: [1.0, 1.0, 1.0, 1.0], },
    Vertex { position: [-0.5, -0.5, 0.5], color: [1.0, 1.0, 1.0, 1.0], },
    Vertex { position: [0.5, -0.5, 0.5], color: [1.0, 1.0, 1.0, 1.0], },
    Vertex { position: [0.5, 0.5, 0.5], color: [1.0, 1.0, 1.0, 1.0], },
    Vertex { position: [-0.5, 0.5, -0.5], color: [1.0, 1.0, 1.0, 1.0], },
    Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 1.0, 1.0, 1.0], },
    Vertex { position: [0.5, -0.5, -0.5], color: [1.0, 1.0, 1.0, 1.0], },
    Vertex { position: [0.5, 0.5, -0.5], color: [1.0, 1.0, 1.0, 1.0], },
];

#[rustfmt::skip]
pub const CUBE_INDICES: &[u16] = &[
    0, 1, 1, 2, 2, 3, 3, 0,
    4, 5, 5, 6, 6, 7, 7, 4,
    0, 4, 1, 5, 2, 6, 3, 7,
];

pub struct Context {
    pub pipeline_layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::RenderPipeline,
}

impl Context {
    fn create_pipeline(
        layout: &wgpu::PipelineLayout,
        device: &wgpu::Device,
        shaders: &Shaders,
    ) -> wgpu::RenderPipeline {
        let vertex_desc = VertexDesc::new();
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("debug_pipe"),
            layout: Some(layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &shaders.debug_vs,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &shaders.debug_fs,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                clamp_depth: false,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::LineList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: COLOR_FORMAT,
                alpha_blend: wgpu::BlendDescriptor {
                    operation: wgpu::BlendOperation::Add,
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                },
                color_blend: wgpu::BlendDescriptor {
                    operation: wgpu::BlendOperation::Add,
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                },
                write_mask: wgpu::ColorWrite::all(),
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[vertex_desc.buffer_desc()],
            },
            sample_count: 1,
            alpha_to_coverage_enabled: false,
            sample_mask: !0,
        })
    }

    pub fn new(device: &wgpu::Device, global: &GlobalContext, shaders: &Shaders) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("debug"),
            bind_group_layouts: &[&global.bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = Self::create_pipeline(&pipeline_layout, device, shaders);

        Context {
            pipeline_layout,
            pipeline,
        }
    }

    pub fn reload(&mut self, device: &wgpu::Device, shaders: &Shaders) {
        self.pipeline = Self::create_pipeline(&self.pipeline_layout, device, shaders);
    }
}
