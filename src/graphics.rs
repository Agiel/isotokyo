use crate::camera;
use crate::utils::*;
use cgmath::prelude::*;
use std::{collections::HashMap, mem, sync::Arc};

pub mod debug;
pub mod global;
pub mod object;
pub mod shaders;
pub mod texture;

pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

#[rustfmt::skip]
const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.015, g: 0.015, b: 0.015, a: 1.0,
};

pub struct ScreenTargets<'a> {
    pub extent: wgpu::Extent3d,
    pub color: &'a wgpu::TextureView,
    pub depth: &'a wgpu::TextureView,
}

struct InstanceArray {
    data: Vec<object::InstanceRaw>,
    texture: Arc<texture::Texture>,
    buffer: Option<wgpu::Buffer>,
}

struct Batcher {
    instances: HashMap<*const texture::Texture, InstanceArray>,
    instances_alpha: HashMap<*const texture::Texture, InstanceArray>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl Batcher {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(object::VERTICES),
            wgpu::BufferUsage::VERTEX,
        );
        let index_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(object::INDICES),
            wgpu::BufferUsage::INDEX,
        );
        Batcher {
            instances: HashMap::new(),
            instances_alpha: HashMap::new(),
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn add_quad(&mut self, texture: &Arc<texture::Texture>, instance: object::Instance, alpha: bool) {
        let instances = match alpha {
            true => &mut self.instances_alpha,
            false => &mut self.instances,
        };
        instances
            .entry(&**texture)
            .or_insert_with(|| InstanceArray {
                data: Vec::new(),
                texture: Arc::clone(texture),
                buffer: None,
            })
            .data
            .push(instance.to_raw());
    }

    pub fn draw<'a>(&'a mut self, pass: &mut wgpu::RenderPass<'a>, device: &wgpu::Device) {
        let num_indices = object::INDICES.len() as u32;
        pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
        pass.set_index_buffer(&self.index_buffer, 0, 0);


        for array in self.instances.values_mut() {
            if array.data.is_empty() {
                continue;
            }
            array.buffer = Some(device.create_buffer_with_data(
                bytemuck::cast_slice(&array.data),
                wgpu::BufferUsage::VERTEX,
            ));
            pass.set_bind_group(1, array.texture.bind_group.as_ref().unwrap(), &[]);
            pass.set_vertex_buffer(1, array.buffer.as_ref().unwrap(), 0, 0);
            pass.draw_indexed(0..num_indices, 0, 0..array.data.len() as u32);
            array.data.clear();
        }

        // TODO: Sort?
        for array in self.instances_alpha.values_mut() {
            if array.data.is_empty() {
                continue;
            }
            array.buffer = Some(device.create_buffer_with_data(
                bytemuck::cast_slice(&array.data),
                wgpu::BufferUsage::VERTEX,
            ));
            pass.set_bind_group(1, array.texture.bind_group.as_ref().unwrap(), &[]);
            pass.set_vertex_buffer(1, array.buffer.as_ref().unwrap(), 0, 0);
            pass.draw_indexed(0..num_indices, 0, 0..array.data.len() as u32);
            array.data.clear();
        }

    }

    pub fn clear(&mut self) {
        self.instances.clear();
        self.instances_alpha.clear();
    }
}

struct DebugLines {
    vertices: Vec<debug::Vertex>,
    indices: Vec<u16>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
}

impl DebugLines {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_buffer: None,
            index_buffer: None,
        }
    }

    fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    fn draw<'a>(&'a mut self, pass: &mut wgpu::RenderPass<'a>, device: &wgpu::Device) {
        if self.vertices.is_empty() {
            return;
        }

        let num_indices = self.indices.len() as u32;

        self.vertex_buffer = Some(device.create_buffer_with_data(
            bytemuck::cast_slice(&self.vertices),
            wgpu::BufferUsage::VERTEX,
        ));
        self.index_buffer = Some(device.create_buffer_with_data(
            bytemuck::cast_slice(&self.indices),
            wgpu::BufferUsage::INDEX,
        ));

        self.clear();

        pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap(), 0, 0);
        pass.set_index_buffer(self.index_buffer.as_ref().unwrap(), 0, 0);
        pass.draw_indexed(0..num_indices, 0, 0..1);
    }

    fn add_lines(&mut self, vertices: &[debug::Vertex], indices: &[u16]) {
        let offset = self.vertices.len() as u16;
        let indices: Vec<_> = indices.into_iter().map(|i| i + offset).collect();
        self.vertices.extend_from_slice(vertices);
        self.indices.extend_from_slice(&indices);
    }
}

struct Render {
    global: global::Context,
    object: object::Context,
    debug: debug::Context,
    shaders: shaders::Shaders,
}

impl Render {
    pub fn new(device: &wgpu::Device) -> Self {
        let shaders = shaders::Shaders::new(device).unwrap();
        let global = global::Context::new(device);
        let object = object::Context::new(device, &global, &shaders);
        let debug = debug::Context::new(device, &global, &shaders);

        Self {
            global,
            object,
            debug,
            shaders,
        }
    }

    pub fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        batcher: &mut Batcher,
        debug_lines: &mut DebugLines,
        camera: &camera::Camera,
        targets: ScreenTargets,
        device: &wgpu::Device,
    ) {
        let mut uniforms = global::Uniforms::new();
        uniforms.update_view_proj(camera);
        let global_staging = device
            .create_buffer_with_data(bytemuck::bytes_of(&uniforms), wgpu::BufferUsage::COPY_SRC);
        encoder.copy_buffer_to_buffer(
            &global_staging,
            0,
            &self.global.uniform_buf,
            0,
            mem::size_of::<global::Uniforms>() as wgpu::BufferAddress,
        );

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: targets.color,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: CLEAR_COLOR,
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: targets.depth,
                depth_load_op: wgpu::LoadOp::Clear,
                depth_store_op: wgpu::StoreOp::Store,
                clear_depth: 1.0,
                stencil_load_op: wgpu::LoadOp::Clear,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_stencil: 0,
            }),
        });

        pass.set_bind_group(0, &self.global.bind_group, &[]);

        pass.set_pipeline(&self.object.pipeline);
        batcher.draw(&mut pass, device);

        pass.set_pipeline(&self.debug.pipeline);
        debug_lines.draw(&mut pass, device);
    }

    pub fn reload(&mut self, device: &wgpu::Device) {
        self.object.reload(device, &self.shaders);
    }

    pub fn resize(&mut self, extent: wgpu::Extent3d, device: &wgpu::Device) {}
}

pub struct Graphics {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    swap_chain: wgpu::SwapChain,
    pub extent: wgpu::Extent3d,
    depth_target: wgpu::TextureView,

    render: Render,
    batcher: Batcher,
    debug_lines: DebugLines,
}

impl Graphics {
    pub async fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();
        let extent = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth: 1,
        };

        let surface = wgpu::Surface::create(window);

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY, // Vulkan + Metal + DX12 + Browser WebGPU
        )
        .await
        .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: Default::default(),
            })
            .await;

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: COLOR_FORMAT,
            width: extent.width,
            height: extent.height,
            //present_mode: wgpu::PresentMode::Mailbox,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let depth_target = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth"),
                size: extent,
                array_layer_count: 1,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: DEPTH_FORMAT,
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            })
            .create_default_view();

        let render = Render::new(&device);
        let batcher = Batcher::new(&device);
        let debug_lines = DebugLines::new();

        Self {
            device,
            queue,
            surface,
            swap_chain,
            extent,
            depth_target,
            render,
            batcher,
            debug_lines,
        }
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.extent = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth: 1,
        };
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: COLOR_FORMAT,
            width: size.width,
            height: size.height,
            //present_mode: wgpu::PresentMode::Mailbox,
            present_mode: wgpu::PresentMode::Fifo,
        };
        self.swap_chain = self.device.create_swap_chain(&self.surface, &sc_desc);
        self.depth_target = self
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth"),
                size: self.extent,
                array_layer_count: 1,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: DEPTH_FORMAT,
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            })
            .create_default_view();

        self.render.resize(self.extent, &self.device);
    }

    pub fn flush(&mut self, camera: &camera::Camera) {
        match self.swap_chain.get_next_texture() {
            Ok(frame) => {
                let targets = ScreenTargets {
                    extent: self.extent,
                    color: &frame.view,
                    depth: &self.depth_target,
                };
                let mut encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Draw"),
                        });
                self.render.draw(
                    &mut encoder,
                    &mut self.batcher,
                    &mut self.debug_lines,
                    camera,
                    targets,
                    &self.device,
                );
                self.batcher.clear();
                self.debug_lines.clear();
                self.queue.submit(&[encoder.finish()]);
            }
            Err(_) => {}
        };
    }

    pub fn load_texture_bytes(
        &self,
        bytes: &[u8],
        label: &str,
    ) -> Result<Arc<texture::Texture>, texture::ImageError> {
        let (mut texture, cmds) = texture::Texture::from_bytes(&self.device, bytes, label)?;
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.render.object.bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some(&format!("{}_bind_group", label)),
        });
        texture.bind_group = Some(bind_group);
        self.queue.submit(&[cmds]);
        Ok(Arc::new(texture))
    }

    pub fn draw_plane(&mut self, texture: &Arc<texture::Texture>, center: Point3, size: f32, color: Vector4) {
        let instance = object::Instance {
            position: center,
            scale: Vector3::new(size, size, size),
            color,
            ..object::Instance::default()
        };
        let alpha = instance.color.w < 1.0;
        self.batcher.add_quad(texture, instance, alpha);
    }

    pub fn draw_billboard(
        &mut self,
        camera: &camera::Camera,
        texture: &Arc<texture::Texture>,
        source: Rect,
        center: Point3,
        size: Vector2,
        offset: Vector3,
    ) {
        let view = Matrix4::look_at(camera.eye, camera.target, camera.up);
        let right = Vector3::new(view.x.x, view.y.x, view.z.x);
        let up = Vector3::new(view.x.y, view.y.y, view.z.y);
        let center = center + right * offset.x + up * offset.y + Vector3::unit_z() * offset.z;

        let plane_quat = Quaternion::look_at(-Vector3::unit_z(), Vector3::unit_y());
        let wish_quat = Quaternion::look_at((camera.target - camera.eye).normalize(), camera.up);
        let orientation = wish_quat.invert() * plane_quat;

        let source = Rect::new(
            source.position.x / texture.size.width as f32,
            source.position.y / texture.size.height as f32,
            source.size.x / texture.size.width as f32,
            source.size.y / texture.size.height as f32
        );

        let instance = object::Instance {
            position: center,
            orientation,
            scale: Vector3::new(size.x, size.y, 1.0),
            source,
            ..object::Instance::default()
        };
        let alpha = instance.color.w < 1.0;
        self.batcher.add_quad(texture, instance, alpha);
    }

    pub fn draw_debug_cube(&mut self, center: Point3, size: Vector3, color: Vector4) {
        let vertices: Vec<_> = debug::CUBE_VERTICES
            .into_iter()
            .map(|v| debug::Vertex {
                position: [
                    v.position[0] * size.x + center.x,
                    v.position[1] * size.y + center.y,
                    v.position[2] * size.z + center.z,
                ],
                color: color.into(),
            })
            .collect();
        self.debug_lines.add_lines(&vertices, debug::CUBE_INDICES);
    }

    pub fn draw_debug_line(&mut self, start: Point3, end: Point3, color: Vector4) {
        self.debug_lines.add_lines(
            &[
                debug::Vertex {
                    position: start.into(),
                    color: color.into(),
                },
                debug::Vertex {
                    position: end.into(),
                    color: color.into(),
                },
            ],
            &[0, 1],
        );
    }

    pub fn draw_debug_grid(&mut self, center: Point3, size: u16) {
        let center = center - Vector3::new(0.5, 0.5, 0.0);
        let color = (0.75, 0.75, 0.75, 0.5);
        for i in 0..size + 1 {
            let offset = size as f32 / -2.0 + i as f32;
            self.draw_debug_line(
                center + Vector3::new(offset, size as f32 / -2.0, 0.0),
                center + Vector3::new(offset, size as f32 / 2.0, 0.0),
                color.into(),
            );
            self.draw_debug_line(
                center + Vector3::new(size as f32 / -2.0, offset, 0.0),
                center + Vector3::new(size as f32 / 2.0, offset, 0.0),
                color.into(),
            );
        }
    }
}
