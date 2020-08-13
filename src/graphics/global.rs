use crate::utils::*;
use crate::camera;
use bytemuck::{Pod, Zeroable};
use cgmath::prelude::*;

use std::mem;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Uniforms {
    view_proj: Matrix4,
}

unsafe impl Pod for Uniforms {}
unsafe impl Zeroable for Uniforms {}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            view_proj: Matrix4::identity(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &camera::Camera) {
        self.view_proj = camera::OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix();
    }
}

pub struct Context {
    pub uniform_buf: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl Context {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Global"),
            bindings: &[
                // View matrix
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::all(),
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        //min_binding_size: None,
                    },
                },
            ],
        });
        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform"),
            size: mem::size_of::<Uniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            //mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Global"),
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buf,
                        range: 0..mem::size_of::<Uniforms>() as wgpu::BufferAddress,
                    },
                },
            ],
        });

        Context {
            uniform_buf,
            bind_group_layout,
            bind_group,
        }
    }
}