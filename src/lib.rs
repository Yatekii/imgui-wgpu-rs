use imgui::DrawCmd::Elements;
use std::mem::size_of;
use std::slice::from_raw_parts;
use std::str::from_utf8;
use imgui::{DrawList, Context, DrawVert, DrawIdx, TextureId, Textures, Ui};

pub type RendererResult<T> = Result<T, RendererError>;

#[derive(Clone, Debug)]
pub enum RendererError {
  VertexBufferTooSmall,
  IndexBufferTooSmall,
  BadTexture(TextureId),
}

#[allow(dead_code)]
pub enum ShaderStage {
  Vertex,
  Fragment,
  Compute,
}

pub enum Shaders {}

impl Shaders {
    fn compile_glsl(code: &str, stage: ShaderStage) -> Vec<u8> {
        use std::io::Read;

        let ty = match stage {
        ShaderStage::Vertex => glsl_to_spirv::ShaderType::Vertex,
        ShaderStage::Fragment => glsl_to_spirv::ShaderType::Fragment,
        ShaderStage::Compute => glsl_to_spirv::ShaderType::Compute,
        };
        
        let mut output = glsl_to_spirv::compile(code, ty).unwrap();
        let mut spv = Vec::new();
        output.read_to_end(&mut spv).unwrap();
        spv
    }

    fn to_string(data: &'static [u8]) -> &'static str {
        match from_utf8(data) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        }
    }

    fn get_program_code() -> (&'static str, &'static str) {
        (
        Shaders::to_string(include_bytes!("imgui.vert")),
        Shaders::to_string(include_bytes!("imgui.frag")),
        )
    }
}

pub struct Texture {
    texture: wgpu::Texture,
    sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
}

impl Texture {
    pub fn new(texture: wgpu::Texture, layout: &wgpu::BindGroupLayout, device: &wgpu::Device) -> Self {

        let view = texture.create_default_view();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Always,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ]
        });

        Texture {
            texture,
            sampler,
            bind_group,
        }
    }

    pub fn view(&self) -> wgpu::TextureView {
        self.texture.create_default_view()
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

#[allow(dead_code)]
pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u64,
    vertex_max: u64,
    index_buffer: wgpu::Buffer,
    index_count: u64,
    index_max: u64,
    textures: Textures<Texture>,
    texture_layout: wgpu::BindGroupLayout,
    clear_color: Option<wgpu::Color>,
}

impl Renderer {
    pub fn new(
        imgui: &mut Context,
        device: &mut wgpu::Device,
        format: wgpu::TextureFormat,
        clear_color: Option<wgpu::Color>,
    ) -> Renderer {

        // Create shaders
        let (vs_code, fs_code) = Shaders::get_program_code();

        let vs_module = device.create_shader_module(&Shaders::compile_glsl(vs_code, ShaderStage::Vertex));
        let fs_module = device.create_shader_module(&Shaders::compile_glsl(fs_code, ShaderStage::Fragment));

        // Create uniform matrix buffer
        let size = 64;
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size, usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::TRANSFER_DST,
        });

        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            wgpu::BindGroupLayoutBinding {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer,
                },
            ]
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &uniform_layout,
        bindings: &[
            wgpu::Binding {
            binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &uniform_buffer,
                    range: 0..size,
                },
            },
        ]
        });

        // Create uniform texture
        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture,
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler,
                },
            ]
        });

        // Create render pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&uniform_layout, &texture_layout],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::PipelineStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::PipelineStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Cw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            },
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[
                wgpu::ColorStateDescriptor {
                    format,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                },
            ],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[
                wgpu::VertexBufferDescriptor {
                    stride: size_of::<DrawVert>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float2,
                            shader_location: 0,
                            offset: 0,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float2,
                            shader_location: 1,
                            offset: 8,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Uint,
                            shader_location: 2,
                            offset: 16,
                        },
                    ]
                },
            ],
            sample_count: 1,
        });

        // Create vertex/index buffer
        let vertex_max = 32768;
        let vertex_size = size_of::<DrawVert>() as u64;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: vertex_max * vertex_size,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::TRANSFER_DST,
        });

        let index_max = 32768;
        let index_size = size_of::<u16>() as u64;
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: vertex_max * index_size,
            usage: wgpu::BufferUsage::INDEX | wgpu::BufferUsage::TRANSFER_DST,
        });

        let mut textures = Textures::new();

        let mut renderer = Renderer {
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            vertex_buffer,
            vertex_count: 0,
            vertex_max,
            index_buffer,
            index_count: 0,
            index_max,
            textures,
            texture_layout,
            clear_color,
        };

        renderer.reload_font_texture(imgui, device);

        renderer
    }

    pub fn textures(&mut self) -> &mut Textures<Texture> {
        &mut self.textures
    }

    pub fn render<'a>(
        &mut self,
        ui: Ui<'a>,
        width: f64,
        height: f64,
        hidpi_factor: f64,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) -> RendererResult<()> {
        if !(width > 0.0 && height > 0.0) {
            return Ok(());
        }
        let fb_size = (
            (width * hidpi_factor) as f32,
            (height * hidpi_factor) as f32,
        );

        let matrix = [
            [(2.0 / width) as f32, 0.0, 0.0, 0.0],
            [0.0, (2.0 / height) as f32, 0.0, 0.0],
            [0.0, 0.0, -1.0, 0.0],
            [-1.0, -1.0, 0.0, 1.0],
        ];

        self.update_uniform_buffer(device, encoder, &matrix);

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &view,
                resolve_target: None,
                load_op: match self.clear_color { Some(_) => wgpu::LoadOp::Clear, _ => wgpu::LoadOp::Load },
                store_op: wgpu::StoreOp::Store,
                clear_color: self.clear_color.unwrap_or(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
            }],
            depth_stencil_attachment: None,
        });

        rpass.set_pipeline(&self.pipeline);
        // rpass.set_vertex_buffers(&[(&self.vertex_buffer, 0)]);
        // rpass.set_index_buffer(&self.index_buffer, 0);
        // self.vertex_count = 0;
        // self.index_count = 0;

        rpass.set_bind_group(0, &self.uniform_bind_group, &[]);

        let fb_scale = ui.io().display_framebuffer_scale;
        let mut draw_data = ui.render();
        
        for draw_list in draw_data.draw_lists() {
            self.render_draw_list(device, &mut rpass, &draw_list, fb_size, (fb_scale[0], fb_scale[1]))?;
        }
        Ok(())
    }

    fn render_draw_list<'render>(
        &mut self,
        device: &wgpu::Device,
        rpass: &mut wgpu::RenderPass<'render>,
        draw_list: &DrawList,
        fb_size: (f32, f32),
        fb_scale: (f32, f32)
    ) -> RendererResult<()> {
        let (fb_width, fb_height) = fb_size;

        let base_vertex = self.vertex_count;
        let mut start = self.index_count as u32;

        let vertex_buffer = self.upload_vertex_buffer(device, draw_list.vtx_buffer());
        let index_buffer = self.upload_index_buffer(device, draw_list.idx_buffer());

        rpass.set_vertex_buffers(&[(&vertex_buffer, 0)]);
        rpass.set_index_buffer(&index_buffer, 0);
        
        for cmd in draw_list.commands() {
            match cmd {
                Elements {
                    count,
                    cmd_params,
                } => {
                    let clip_rect = [
                        cmd_params.clip_rect[0] * fb_scale.0,
                        cmd_params.clip_rect[1] * fb_scale.1,
                        cmd_params.clip_rect[2] * fb_scale.0,
                        cmd_params.clip_rect[3] * fb_scale.1,
                    ];
                    let texture_id = cmd_params.texture_id.into();
                    let tex = self
                        .textures
                        .get(texture_id)
                        .ok_or_else(|| RendererError::BadTexture(texture_id))?;

                    rpass.set_bind_group(1, tex.bind_group(), &[]);

                    let end = start + count as u32;
                    let scissors = (
                        clip_rect[0].max(0.0).min(fb_width).round() as u32,
                        clip_rect[1].max(0.0).min(fb_height).round() as u32,
                        (clip_rect[2] - clip_rect[0])
                            .abs()
                            .min(fb_width)
                            .round() as u32,
                        (clip_rect[3] - clip_rect[1])
                            .abs()
                            .min(fb_height)
                            .round() as u32,
                    );

                    rpass.set_scissor_rect(scissors.0, scissors.1, scissors.2, scissors.3);

                    rpass.draw_indexed(start..end, base_vertex as i32, 0..1);
                    
                    start = end;
                },
                _ => {},
            }
        }
        Ok(())
    }

    fn update_uniform_buffer(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        matrix: &[[f32; 4]; 4],
    ) {
        let buffer = device.create_buffer_mapped(
            16,
            wgpu::BufferUsage::TRANSFER_SRC,
        )
        .fill_from_slice(matrix.iter().flatten().map(|f| *f).collect::<Vec<f32>>().as_slice());
        encoder.copy_buffer_to_buffer(
            &buffer,
            0,
            &self.uniform_buffer,
            0,
            64
        );
    }

    fn upload_vertex_buffer(
        &mut self,
        device: &wgpu::Device,
        vtx_buffer: &[DrawVert],
    //  ) -> RendererResult<wgpu::Buffer> {
    ) -> wgpu::Buffer {
        let vertex_count = vtx_buffer.len();
        device.create_buffer_mapped(
            vertex_count,
            wgpu::BufferUsage::TRANSFER_SRC | wgpu::BufferUsage::VERTEX,
        )
        .fill_from_slice(vtx_buffer)
        /*
        let size = (vtx_buffer.len() * size_of::<ImDrawVert>()) as u32;
        let (buffer, data) = device.create_buffer_mapped(&wgpu::BufferDescriptor {
        size, usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::TRANSFER_DST | wgpu::BufferUsage::MAP_WRITE,
        });

        data.copy_from_slice(&vtx_buffer);
        buffer.unmap();
        Ok(buffer)
        */
    }

    fn upload_index_buffer(
        &mut self,
        device: &wgpu::Device,
        idx_buffer: &[DrawIdx],
    //  ) -> RendererResult<wgpu::Buffer> {
    ) -> wgpu::Buffer {
        let index_count = idx_buffer.len();
        device.create_buffer_mapped(
            index_count,
            wgpu::BufferUsage::TRANSFER_SRC | wgpu::BufferUsage::INDEX,
        )
        .fill_from_slice(idx_buffer)
        /*
        let size = (idx_buffer.len() * size_of::<ImDrawIdx>()) as u32;
        let (buffer, data) = device.create_buffer_mapped(&wgpu::BufferDescriptor {
        size, usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::TRANSFER_DST | wgpu::BufferUsage::MAP_WRITE,
        });

        data.copy_from_slice(&idx_buffer);
        buffer.unmap();
        Ok(buffer)
        */
    }

    pub fn reload_font_texture(&mut self, imgui: &mut Context, device: &mut wgpu::Device) {
        let mut data;
        let handle = {
            let mut atlas = imgui.fonts();
            let handle = atlas.build_rgba32_texture();
            data = handle.data.to_vec();
            
            imgui::FontAtlasTexture {
                width: handle.width,
                height: handle.height,
                data: data.as_slice(),
            }
        };
        let font_texture_id = self.upload_texture(imgui, device, &handle);
        
        let mut atlas = imgui.fonts();
        atlas.tex_id = font_texture_id;
    }

    fn upload_texture(
        &mut self,
        imgui: &mut Context,
        device: &mut wgpu::Device,
        handle: &imgui::FontAtlasTexture,
    ) -> TextureId {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d { width: handle.width, height: handle.height, depth: 1 },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::TRANSFER_DST,
        });

        // Place in wgpu buffer
        let bytes = handle.data.len();
        let buffer = device.create_buffer_mapped(
            bytes,
            wgpu::BufferUsage::TRANSFER_SRC,
        )
        .fill_from_slice(handle.data);

        // Upload immediately
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            todo: 0,
        });

        let pixel_size = bytes as u32 / handle.width / handle.height;
        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &buffer,
                offset: 0,
                row_pitch: pixel_size * handle.width,
                image_height: handle.height,
            },
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            wgpu::Extent3d {
                width: handle.width,
                height: handle.height,
                depth: 1,
            },
        );

        device
            .get_queue()
            .submit(&[encoder.finish()]);

        let texture = Texture::new(texture, &self.texture_layout, device);
        self.textures.insert(texture)
    }
}