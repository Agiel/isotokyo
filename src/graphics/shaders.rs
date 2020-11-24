use std::{
    fs::File,
    io::{BufReader, Error as IoError, Read, Write},
    path::PathBuf,
};

pub struct Shaders {
    pub vs: wgpu::ShaderModule,
    pub fs: wgpu::ShaderModule,
    pub debug_vs: wgpu::ShaderModule,
    pub debug_fs: wgpu::ShaderModule,
}

impl Shaders {
    fn fail(name: &str, source: &str, log: &str) -> ! {
        println!("Generated shader:");
        for (i, line) in source.lines().enumerate() {
            println!("{:3}| {}", i + 1, line);
        }
        let msg = log.replace("\\n", "\n");
        panic!("\nUnable to compile '{}': {}", name, msg);
    }

    fn compile(device: &wgpu::Device, compiler: &mut shaderc::Compiler, src: &str, kind:shaderc::ShaderKind, name: &str, entry: &str) -> wgpu::ShaderModule {
        let spirv = compiler.compile_into_spirv(src, kind, name, entry, None).unwrap();
        let data = wgpu::util::make_spirv(&spirv.as_binary_u8());
        device.create_shader_module(data)
    }

    pub fn new(
        device: &wgpu::Device,
    ) -> Result<Self, IoError> {
        let mut compiler = shaderc::Compiler::new().unwrap();

        let vs_src = include_str!("shaders/shader.vert");
        let vs_module = Self::compile(device, &mut compiler, vs_src, shaderc::ShaderKind::Vertex, "shader.vert", "main");
        let fs_src = include_str!("shaders/shader.frag");
        let fs_module = Self::compile(device, &mut compiler, fs_src, shaderc::ShaderKind::Fragment, "shader.frag", "main");

        let debug_vs_src = include_str!("shaders/debug.vert");
        let debug_vs = Self::compile(device, &mut compiler, debug_vs_src, shaderc::ShaderKind::Vertex, "debug.vert", "main");
        let debug_fs_src = include_str!("shaders/debug.frag");
        let debug_fs = Self::compile(device, &mut compiler, debug_fs_src, shaderc::ShaderKind::Fragment, "debug.frag", "main");

        Ok(Self {
            vs: vs_module,
            fs: fs_module,
            debug_vs,
            debug_fs,
        })
    }
}
