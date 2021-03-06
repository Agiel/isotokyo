use crate::graphics::{
    global::Context as GlobalContext, shaders::Shaders, COLOR_FORMAT, DEPTH_FORMAT,
};
use crate::utils::*;
use bytemuck::{Pod, Zeroable};
use cgmath::prelude::*;

use std::mem;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

pub struct VertexDesc {
    attributes: [wgpu::VertexAttributeDescriptor; 2],
}

impl VertexDesc {
    pub fn new() -> Self {
        VertexDesc {
            attributes: wgpu::vertex_attr_array![0 => Float3, 1 => Float2],
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
pub const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 0.0], },
    Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0], },
    Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 1.0], },
    Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 0.0], },
];

#[rustfmt::skip]
pub const INDICES: &[u16] = &[
    0, 1, 3,
    1, 2, 3,
];

pub struct Instance {
    pub position: Point3,
    pub orientation: Quaternion,
    pub scale: Vector3,
    pub color: Vector4,
    pub source: Rect,
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            position: Point3::origin(),
            orientation: Quaternion::one(),
            scale: (1., 1., 1.).into(),
            color: WHITE.into(),
            source: Rect {
                position: Point2::origin(),
                size: (1., 1.).into(),
            }
        }
    }
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: Matrix4::from_translation(self.position.to_vec())
                * Matrix4::from(self.orientation)
                * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z),
            color: self.color,
            source: Vector4::new(
                self.source.position.x,
                self.source.position.y,
                self.source.size.x,
                self.source.size.y,
            ),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct InstanceRaw {
    model: Matrix4,
    color: Vector4,
    source: Vector4,
}

unsafe impl Pod for InstanceRaw {}
unsafe impl Zeroable for InstanceRaw {}

struct InstanceDesc {
    attributes: [wgpu::VertexAttributeDescriptor; 6],
}

impl InstanceDesc {
    pub fn new() -> Self {
        InstanceDesc {
            attributes: wgpu::vertex_attr_array![
                // model
                2 => Float4, 3 => Float4, 4 => Float4, 5 => Float4,
                // tint
                6 => Float4,
                // source
                7 => Float4
            ],
        }
    }

    pub fn buffer_desc(&self) -> wgpu::VertexBufferDescriptor {
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &self.attributes,
        }
    }
}

pub struct Context {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::RenderPipeline,
    pub pipeline_alpha: wgpu::RenderPipeline,
}

impl Context {
    fn create_pipeline(
        layout: &wgpu::PipelineLayout,
        device: &wgpu::Device,
        shaders: &Shaders,
        depth_write_enabled: bool,
    ) -> wgpu::RenderPipeline {
        let vertex_desc = VertexDesc::new();
        let instance_desc = InstanceDesc::new();
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("object_pipe"),
            layout: Some(layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &shaders.vs,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &shaders.fs,
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
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
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
                depth_write_enabled,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[vertex_desc.buffer_desc(), instance_desc.buffer_desc()],
            },
            sample_count: 1,
            alpha_to_coverage_enabled: false,
            sample_mask: !0,
        })
    }

    pub fn new(device: &wgpu::Device, global: &GlobalContext, shaders: &Shaders) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Object"),
            entries: &[
                // Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Uint,
                    },
                    count: None,
                },
                // Texture sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("object"),
            bind_group_layouts: &[&global.bind_group_layout, &bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = Self::create_pipeline(&pipeline_layout, device, shaders, true);
        let pipeline_alpha = Self::create_pipeline(&pipeline_layout, device, shaders, false);

        Context {
            bind_group_layout,
            pipeline_layout,
            pipeline,
            pipeline_alpha,
        }
    }

    pub fn reload(&mut self, device: &wgpu::Device, shaders: &Shaders) {
        self.pipeline = Self::create_pipeline(&self.pipeline_layout, device, shaders, true);
        self.pipeline_alpha = Self::create_pipeline(&self.pipeline_layout, device, shaders, false);
    }
}
