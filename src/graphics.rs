use crate::camera;
use crate::config;
use crate::utils::*;
use cgmath::prelude::*;
use std::{collections::HashMap, mem, sync::Arc};
use wgpu_glyph::{ab_glyph, FontId, GlyphBrush, GlyphBrushBuilder, Section, Text};
use wgpu::util::DeviceExt as _;

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
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex_buffer"),
            contents: bytemuck::cast_slice(object::VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index_buffer"),
            contents: bytemuck::cast_slice(object::INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });
        Batcher {
            instances: HashMap::new(),
            instances_alpha: HashMap::new(),
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn add_quad(
        &mut self,
        texture: &Arc<texture::Texture>,
        instance: object::Instance,
        alpha: bool,
    ) {
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

    pub fn draw<'a>(
        &'a mut self,
        pass: &mut wgpu::RenderPass<'a>,
        device: &wgpu::Device,
        object: &'a object::Context,
    ) {
        let num_indices = object::INDICES.len() as u32;
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..));

        pass.set_pipeline(&object.pipeline);
        for array in self.instances.values_mut() {
            if array.data.is_empty() {
                continue;
            }
            array.buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instance"),
                contents: bytemuck::cast_slice(&array.data),
                usage: wgpu::BufferUsage::VERTEX,
            }));
            pass.set_bind_group(1, array.texture.bind_group.as_ref().unwrap(), &[]);
            pass.set_vertex_buffer(1, array.buffer.as_ref().unwrap().slice(..));
            pass.draw_indexed(0..num_indices, 0, 0..array.data.len() as u32);
            array.data.clear();
        }

        // TODO: Sort?
        pass.set_pipeline(&object.pipeline_alpha);
        for array in self.instances_alpha.values_mut() {
            if array.data.is_empty() {
                continue;
            }
            array.buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instance_alpha"),
                contents: bytemuck::cast_slice(&array.data),
                usage: wgpu::BufferUsage::VERTEX,
            }));
            pass.set_bind_group(1, array.texture.bind_group.as_ref().unwrap(), &[]);
            pass.set_vertex_buffer(1, array.buffer.as_ref().unwrap().slice(..));
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

    fn draw<'a>(
        &'a mut self,
        pass: &mut wgpu::RenderPass<'a>,
        device: &wgpu::Device,
        debug: &'a debug::Context,
    ) {
        if self.vertices.is_empty() {
            return;
        }

        let num_indices = self.indices.len() as u32;

        self.vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex_buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsage::VERTEX,
        }));
        self.index_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index_buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsage::INDEX,
        }));

        self.clear();

        pass.set_pipeline(&debug.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        pass.set_index_buffer(self.index_buffer.as_ref().unwrap().slice(..));
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
        targets: &ScreenTargets,
        device: &wgpu::Device,
    ) {
        let mut uniforms = global::Uniforms::new();
        uniforms.update_view_proj(camera);
        let global_staging = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("global_staging"),
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsage::COPY_SRC,
            });
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
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: targets.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        pass.set_bind_group(0, &self.global.bind_group, &[]);

        batcher.draw(&mut pass, device, &self.object);

        debug_lines.draw(&mut pass, device, &self.debug);
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
    present_mode: wgpu::PresentMode,

    staging_belt: wgpu::util::StagingBelt,
    local_pool: futures::executor::LocalPool,
    local_spawner: futures::executor::LocalSpawner,

    render: Render,
    batcher: Batcher,
    debug_lines: DebugLines,
    glyph_brush: Option<GlyphBrush<()>>,
    fonts: HashMap<String, FontId>,
}

impl Graphics {
    pub async fn new(window: &winit::window::Window, config: &config::Config) -> Self {
        let size = window.inner_size();
        let extent = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth: 1,
        };

        let instance = wgpu::Instance::new(wgpu::BackendBit::all());
        let surface = unsafe { instance.create_surface(window) };

        let present_mode = match config.graphics.present_mode {
            config::PresentMode::Immediate => wgpu::PresentMode::Immediate,
            config::PresentMode::Mailbox => wgpu::PresentMode::Mailbox,
            config::PresentMode::Fifo => wgpu::PresentMode::Fifo,
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: false,
                },
                None,
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: COLOR_FORMAT,
            width: extent.width,
            height: extent.height,
            present_mode,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let depth_target = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth"),
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: DEPTH_FORMAT,
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            })
            .create_view(&wgpu::TextureViewDescriptor::default());

        let staging_belt = wgpu::util::StagingBelt::new(1024);
        let local_pool = futures::executor::LocalPool::new();
        let local_spawner = local_pool.spawner();

        let render = Render::new(&device);
        let batcher = Batcher::new(&device);
        let debug_lines = DebugLines::new();
        let glyph_brush = None;
        let fonts = HashMap::new();

        Self {
            device,
            queue,
            surface,
            swap_chain,
            extent,
            depth_target,
            staging_belt,
            local_pool,
            local_spawner,
            render,
            batcher,
            debug_lines,
            glyph_brush,
            fonts,
            present_mode,
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
            present_mode: self.present_mode,
        };
        self.swap_chain = self.device.create_swap_chain(&self.surface, &sc_desc);
        self.depth_target = self
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth"),
                size: self.extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: DEPTH_FORMAT,
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            })
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render.resize(self.extent, &self.device);
    }

    pub fn flush(&mut self, camera: &camera::Camera) {
        match self.swap_chain.get_current_frame() {
            Ok(frame) => {
                let targets = ScreenTargets {
                    extent: self.extent,
                    color: &frame.output.view,
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
                    &targets,
                    &self.device,
                );

                let device = &self.device;
                let extent = &self.extent;
                if let Some(glyph_brush) = &mut self.glyph_brush {
                    glyph_brush
                        .draw_queued(
                            device,
                            &mut self.staging_belt,
                            &mut encoder,
                            &targets.color,
                            extent.width,
                            extent.height,
                        )
                        .expect("Draw queued text");
                }

                self.staging_belt.finish();
                self.batcher.clear();
                self.debug_lines.clear();
                self.queue.submit(Some(encoder.finish()));

                // Recall unused staging buffers
                use futures::task::SpawnExt;
                self.local_spawner
                    .spawn(self.staging_belt.recall())
                    .expect("Recall staging belt");
                self.local_pool.run_until_stalled();
            }
            Err(_) => {}
        };
    }

    pub fn load_texture_bytes(
        &self,
        bytes: &[u8],
        label: &str,
    ) -> Result<Arc<texture::Texture>, texture::ImageError> {
        let mut texture = texture::Texture::from_bytes(&self.device, &self.queue, bytes, label)?;
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.render.object.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some(&format!("{}_bind_group", label)),
        });
        texture.bind_group = Some(bind_group);
        Ok(Arc::new(texture))
    }

    pub fn load_font_bytes(
        &mut self,
        name: &str,
        bytes: Vec<u8>,
    ) -> Result<FontId, ab_glyph::InvalidFont> {
        let mut font_id = FontId(0);
        let font = ab_glyph::FontArc::try_from_vec(bytes)?;
        let glyph_brush = if let Some(glyph_brush) = &mut self.glyph_brush {
            font_id = glyph_brush.add_font(font);
            None
        } else {
            Some(GlyphBrushBuilder::using_font(font).build(&self.device, COLOR_FORMAT))
        };
        if glyph_brush.is_some() {
            self.glyph_brush = glyph_brush;
        }

        Ok(font_id)
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        font: FontId,
        size: f32,
        position: Vector2,
        color: (f32, f32, f32, f32),
    ) {
        if let Some(glyph_brush) = &mut self.glyph_brush {
            glyph_brush.queue(Section {
                screen_position: (position.x, position.y),
                bounds: (self.extent.width as f32, self.extent.height as f32),
                text: vec![Text::new(text)
                    .with_color([color.0, color.1, color.2, color.3])
                    .with_scale(size)
                    .with_font_id(font)],
                ..Default::default()
            });
        }
    }

    pub fn draw_plane(
        &mut self,
        texture: &Arc<texture::Texture>,
        center: Point3,
        size: f32,
        color: Vector4,
    ) {
        let instance = object::Instance {
            position: center,
            scale: Vector3::new(size, size, size),
            color,
            ..Default::default()
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
            source.size.y / texture.size.height as f32,
        );

        let instance = object::Instance {
            position: center,
            orientation,
            scale: Vector3::new(size.x, size.y, 1.0),
            source,
            ..Default::default()
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
