use imgui::{
    Context, DrawCmd::Elements, DrawData, DrawIdx, DrawList, DrawVert, TextureId, Textures,
};
use smallvec::SmallVec;
use std::mem::size_of;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;

pub type RendererResult<T> = Result<T, RendererError>;

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
struct DrawVertPod(DrawVert);

unsafe impl bytemuck::Zeroable for DrawVertPod {}
unsafe impl bytemuck::Pod for DrawVertPod {}

#[derive(Clone, Debug)]
pub enum RendererError {
    BadTexture(TextureId),
}

#[allow(dead_code)]
enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

#[cfg(feature = "shaderc")]
struct Shaders;

#[cfg(feature = "shaderc")]
impl Shaders {
    fn compile_glsl(code: &str, stage: ShaderStage) -> ShaderModuleSource<'static> {
        let ty = match stage {
            ShaderStage::Vertex => shaderc::ShaderKind::Vertex,
            ShaderStage::Fragment => shaderc::ShaderKind::Fragment,
            ShaderStage::Compute => shaderc::ShaderKind::Compute,
        };

        let mut compiler = shaderc::Compiler::new().unwrap();
        let binary_result = compiler
            .compile_into_spirv(code, ty, "shader.glsl", "main", None)
            .unwrap();

        let source = util::make_spirv(&binary_result.as_binary_u8());
        if let ShaderModuleSource::SpirV(cow) = source {
            ShaderModuleSource::SpirV(std::borrow::Cow::Owned(cow.into()))
        } else {
            unreachable!()
        }
    }

    fn get_program_code() -> (&'static str, &'static str) {
        (include_str!("imgui.vert"), include_str!("imgui.frag"))
    }
}

/// Config for creating a texture.
///
/// Uses the builder pattern.
#[derive(Clone)]
pub struct TextureConfig<'a> {
    /// The size of the texture.
    pub size: Extent3d,
    /// An optional label for the texture used for debugging.
    pub label: Option<&'a str>,
    /// The format of the texture, if not set uses the format from the renderer.
    pub format: Option<TextureFormat>,
    /// The usage of the texture.
    pub usage: TextureUsage,
    /// The mip level of the texture.
    pub mip_level_count: u32,
    /// The sample count of the texture.
    pub sample_count: u32,
    /// The dimension of the texture.
    pub dimension: TextureDimension,
}

impl<'a> TextureConfig<'a> {
    /// Create a new texture config with the specified `width` and `height`.
    pub fn new(width: u32, height: u32) -> TextureConfig<'static> {
        TextureConfig {
            size: Extent3d {
                width,
                height,
                depth: 1,
            },
            label: None,
            format: None,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
        }
    }

    /// Set the depth of the texture.
    pub fn set_depth(mut self, depth: u32) -> Self {
        self.size.depth = depth;
        self
    }

    /// Set the debug label of the texture.
    pub fn set_label<'b>(mut self, label: &'b str) -> TextureConfig<'b> {
        self.label = None;

        // Change the lifetime from 'a to 'b.
        // Safe because there is guaranteed to be no reference in `label`.
        let mut result: TextureConfig<'b> = unsafe { std::mem::transmute(self) };

        result.label = Some(label);
        result
    }

    /// Set the texture format.
    pub fn set_format(mut self, format: TextureFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Set the texture usage.
    pub fn set_usage(mut self, usage: TextureUsage) -> Self {
        self.usage = usage;
        self
    }

    /// Set the texture mip level.
    pub fn set_mip_level_count(mut self, mip_level_count: u32) -> Self {
        self.mip_level_count = mip_level_count;
        self
    }

    /// Set the texture sample count.
    pub fn set_sample_count(mut self, sample_count: u32) -> Self {
        self.sample_count = sample_count;
        self
    }

    /// Set the texture dimension.
    pub fn set_dimension(mut self, dimension: TextureDimension) -> Self {
        self.dimension = dimension;
        self
    }

    /// Build a new `Texture` consuming this config.
    pub fn build(self, device: &Device, renderer: &Renderer) -> Texture {
        Texture::new(device, renderer, self)
    }
}

/// A container for a bindable texture.
pub struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    bind_group: BindGroup,
    size: Extent3d,
}

impl Texture {
    /// Create a `Texture` from its raw parts.
    pub fn from_raw_parts(
        texture: wgpu::Texture,
        view: wgpu::TextureView,
        bind_group: BindGroup,
        size: Extent3d,
    ) -> Self {
        Texture {
            texture,
            view,
            bind_group,
            size,
        }
    }

    /// Create a new GPU texture width the specified `config`.
    pub fn new(device: &Device, renderer: &Renderer, config: TextureConfig) -> Self {
        // Create the wgpu texture.
        let texture = device.create_texture(&TextureDescriptor {
            label: config.label,
            size: config.size,
            mip_level_count: config.mip_level_count,
            sample_count: config.sample_count,
            dimension: config.dimension,
            format: config.format.unwrap_or(renderer.config.texture_format),
            usage: config.usage,
        });

        // Extract the texture view.
        let view = texture.create_view(&TextureViewDescriptor::default());

        // Create the texture sampler.
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("imgui-wgpu sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: None,
        });

        // Create the texture bind group from the layout.
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: config.label,
            layout: &renderer.texture_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        Texture {
            texture,
            view,
            bind_group,
            size: config.size,
        }
    }

    /// Write `data` to the texture.
    ///
    /// - `data`: 32-bit RGBA bitmap data.
    /// - `width`: The width of the source bitmap (`data`) in pixels.
    /// - `height`: The height of the source bitmap (`data`) in pixels.
    pub fn write(&self, queue: &Queue, data: &[u8], width: u32, height: u32) {
        queue.write_texture(
            // destination (sub)texture
            TextureCopyView {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d { x: 0, y: 0, z: 0 },
            },
            // source bitmap data
            data,
            // layout of the source bitmap
            TextureDataLayout {
                offset: 0,
                bytes_per_row: width * 4,
                rows_per_image: height,
            },
            // size of the source bitmap
            Extent3d {
                width,
                height,
                depth: 1,
            },
        );
    }

    /// The width of the texture in pixels.
    pub fn width(&self) -> u32 {
        self.size.width
    }

    /// The height of the texture in pixels.
    pub fn height(&self) -> u32 {
        self.size.height
    }

    /// The depth of the texture.
    pub fn depth(&self) -> u32 {
        self.size.depth
    }

    /// The size of the texture in pixels.
    pub fn size(&self) -> Extent3d {
        self.size
    }

    /// The underlying `wgpu::Texture`.
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    /// The `wgpu::TextureView` of the underlying texture.
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

/// Congiguration for the renderer.
pub struct RendererConfig<'vs, 'fs> {
    texture_format: TextureFormat,
    depth_format: Option<TextureFormat>,
    sample_count: u32,
    vertex_shader: Option<ShaderModuleSource<'vs>>,
    fragment_shader: Option<ShaderModuleSource<'fs>>,
}

impl RendererConfig<'_, '_> {
    /// Create a new renderer config with custom shaders.
    pub fn with_shaders<'vs, 'fs>(
        vertex_shader: ShaderModuleSource<'vs>,
        fragment_shader: ShaderModuleSource<'fs>,
    ) -> RendererConfig<'vs, 'fs> {
        RendererConfig {
            texture_format: TextureFormat::Rgba8Unorm,
            depth_format: None,
            sample_count: 1,
            vertex_shader: Some(vertex_shader),
            fragment_shader: Some(fragment_shader),
        }
    }

    /// Create a new renderer config with precompiled default shaders.
    pub fn new() -> RendererConfig<'static, 'static> {
        Self::with_shaders(
            include_spirv!("imgui.vert.spv"),
            include_spirv!("imgui.frag.spv"),
        )
    }

    /// Create a new renderer config with newly compiled shaders.
    #[cfg(feature = "shaderc")]
    pub fn new_glsl() -> RendererConfig<'static, 'static> {
        let (vs_code, fs_code) = Shaders::get_program_code();
        let vs_raw = Shaders::compile_glsl(vs_code, ShaderStage::Vertex);
        let fs_raw = Shaders::compile_glsl(fs_code, ShaderStage::Fragment);

        Self::with_shaders(vs_raw, fs_raw)
    }

    /// Set the texture format used by the renderer.
    pub fn set_texture_format(mut self, texture_format: TextureFormat) -> Self {
        self.texture_format = texture_format;
        self
    }

    /// Set the depth format used by the renderer.
    pub fn set_depth_format(mut self, depth_format: TextureFormat) -> Self {
        self.depth_format = Some(depth_format);
        self
    }

    /// Set the sample count used by the renderer.
    pub fn set_sample_count(mut self, sample_count: u32) -> Self {
        self.sample_count = sample_count;
        self
    }

    /// Build a new `Renderer` consuming this config.
    pub fn build(self, imgui: &mut Context, device: &Device, queue: &Queue) -> Renderer {
        Renderer::new(imgui, device, queue, self)
    }
}

pub struct Renderer {
    pipeline: RenderPipeline,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,
    /// Textures of the font atlas and all images.
    pub textures: Textures<Texture>,
    texture_layout: BindGroupLayout,
    index_buffers: SmallVec<[Buffer; 4]>,
    vertex_buffers: SmallVec<[Buffer; 4]>,
    config: RendererConfig<'static, 'static>,
}

impl Renderer {
    /// Create an entirely new imgui wgpu renderer.
    pub fn new(
        imgui: &mut Context,
        device: &Device,
        queue: &Queue,
        config: RendererConfig,
    ) -> Renderer {
        let RendererConfig {
            texture_format,
            depth_format,
            sample_count,
            vertex_shader,
            fragment_shader,
        } = config;

        // Load shaders.
        let vs_module = device.create_shader_module(vertex_shader.unwrap());
        let fs_module = device.create_shader_module(fragment_shader.unwrap());

        // Create the uniform matrix buffer.
        let size = 64;
        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("imgui-wgpu uniform buffer"),
            size,
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        // Create the uniform matrix buffer bind group layout.
        let uniform_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Create the uniform matrix buffer bind group.
        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("imgui-wgpu bind group"),
            layout: &uniform_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(uniform_buffer.slice(..)),
            }],
        });

        // Create the texture layout for further usage.
        let texture_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("imgui-wgpu bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        component_type: TextureComponentType::Float,
                        dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler { comparison: false },
                    count: None,
                },
            ],
        });

        // Create the render pipeline layout.
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("imgui-wgpu pipeline layout"),
            bind_group_layouts: &[&uniform_layout, &texture_layout],
            push_constant_ranges: &[],
        });

        // Create the render pipeline.
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("imgui-wgpu pipeline"),
            layout: Some(&pipeline_layout),
            vertex_stage: ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Cw,
                cull_mode: CullMode::None,
                clamp_depth: false,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format: texture_format,
                color_blend: BlendDescriptor {
                    src_factor: BlendFactor::SrcAlpha,
                    dst_factor: BlendFactor::OneMinusSrcAlpha,
                    operation: BlendOperation::Add,
                },
                alpha_blend: BlendDescriptor {
                    src_factor: BlendFactor::OneMinusDstAlpha,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: depth_format.map(|format| wgpu::DepthStencilStateDescriptor {
                format,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint16,
                vertex_buffers: &[VertexBufferDescriptor {
                    stride: size_of::<DrawVert>() as BufferAddress,
                    step_mode: InputStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float2, 1 => Float2, 2 => Uint],
                }],
            },
            sample_count,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let mut renderer = Renderer {
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            textures: Textures::new(),
            texture_layout,
            vertex_buffers: SmallVec::new(),
            index_buffers: SmallVec::new(),
            config: RendererConfig {
                texture_format,
                depth_format,
                sample_count,
                vertex_shader: None,
                fragment_shader: None,
            },
        };

        // Immediately load the font texture to the GPU.
        renderer.reload_font_texture(imgui, device, queue);

        renderer
    }

    /// Render the current imgui frame.
    pub fn render<'r>(
        &'r mut self,
        draw_data: &DrawData,
        queue: &Queue,
        device: &Device,
        rpass: &mut RenderPass<'r>,
    ) -> RendererResult<()> {
        let fb_width = draw_data.display_size[0] * draw_data.framebuffer_scale[0];
        let fb_height = draw_data.display_size[1] * draw_data.framebuffer_scale[1];

        // If the render area is <= 0, exit here and now.
        if !(fb_width > 0.0 && fb_height > 0.0) {
            return Ok(());
        }

        let width = draw_data.display_size[0];
        let height = draw_data.display_size[1];

        let offset_x = draw_data.display_pos[0] / width;
        let offset_y = draw_data.display_pos[1] / height;

        // Create and update the transform matrix for the current frame.
        // This is required to adapt to vulkan coordinates.
        // let matrix = [
        //     [2.0 / width, 0.0, 0.0, 0.0],
        //     [0.0, 2.0 / height as f32, 0.0, 0.0],
        //     [0.0, 0.0, -1.0, 0.0],
        //     [-1.0, -1.0, 0.0, 1.0],
        // ];
        let matrix = [
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, 2.0 / -height as f32, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0 - offset_x * 2.0, 1.0 + offset_y * 2.0, 0.0, 1.0],
        ];
        self.update_uniform_buffer(queue, &matrix);

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.uniform_bind_group, &[]);

        self.vertex_buffers.clear();
        self.index_buffers.clear();

        for draw_list in draw_data.draw_lists() {
            self.vertex_buffers
                .push(self.upload_vertex_buffer(device, draw_list.vtx_buffer()));
            self.index_buffers
                .push(self.upload_index_buffer(device, draw_list.idx_buffer()));
        }

        // Execute all the imgui render work.
        for (draw_list_buffers_index, draw_list) in draw_data.draw_lists().enumerate() {
            self.render_draw_list(
                rpass,
                &draw_list,
                draw_data.display_pos,
                draw_data.framebuffer_scale,
                draw_list_buffers_index,
            )?;
        }

        Ok(())
    }

    /// Render a given `DrawList` from imgui onto a wgpu frame.
    fn render_draw_list<'render>(
        &'render self,
        rpass: &mut RenderPass<'render>,
        draw_list: &DrawList,
        clip_off: [f32; 2],
        clip_scale: [f32; 2],
        draw_list_buffers_index: usize,
    ) -> RendererResult<()> {
        let mut start = 0;

        let index_buffer = &self.index_buffers[draw_list_buffers_index];
        let vertex_buffer = &self.vertex_buffers[draw_list_buffers_index];

        // Make sure the current buffers are attached to the render pass.
        rpass.set_index_buffer(index_buffer.slice(..));
        rpass.set_vertex_buffer(0, vertex_buffer.slice(..));

        for cmd in draw_list.commands() {
            match cmd {
                Elements { count, cmd_params } => {
                    let clip_rect = [
                        (cmd_params.clip_rect[0] - clip_off[0]) * clip_scale[0],
                        (cmd_params.clip_rect[1] - clip_off[1]) * clip_scale[1],
                        (cmd_params.clip_rect[2] - clip_off[0]) * clip_scale[0],
                        (cmd_params.clip_rect[3] - clip_off[1]) * clip_scale[1],
                    ];

                    // Set the current texture bind group on the renderpass.
                    let texture_id = cmd_params.texture_id;
                    let tex = self
                        .textures
                        .get(texture_id)
                        .ok_or(RendererError::BadTexture(texture_id))?;
                    rpass.set_bind_group(1, &tex.bind_group, &[]);

                    // Set scissors on the renderpass.
                    let scissors = (
                        clip_rect[0].max(0.0).floor() as u32,
                        clip_rect[1].max(0.0).floor() as u32,
                        (clip_rect[2] - clip_rect[0]).abs().ceil() as u32,
                        (clip_rect[3] - clip_rect[1]).abs().ceil() as u32,
                    );
                    rpass.set_scissor_rect(scissors.0, scissors.1, scissors.2, scissors.3);

                    // Draw the current batch of vertices with the renderpass.
                    let end = start + count as u32;
                    rpass.draw_indexed(start..end, 0, 0..1);
                    start = end;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Updates the current uniform buffer containing the transform matrix.
    fn update_uniform_buffer(&mut self, queue: &Queue, matrix: &[[f32; 4]; 4]) {
        let data = bytemuck::bytes_of(matrix);

        queue.write_buffer(&self.uniform_buffer, 0, data);
    }

    /// Upload the vertex buffer to the GPU.
    fn upload_vertex_buffer(&self, device: &Device, vertices: &[DrawVert]) -> Buffer {
        // Safety: DrawVertPod is #[repr(transparent)] over DrawVert and DrawVert _should_ be Pod.
        let vertices = unsafe {
            std::slice::from_raw_parts(vertices.as_ptr() as *mut DrawVertPod, vertices.len())
        };

        let data = bytemuck::cast_slice(&vertices);
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("imgui-wgpu vertex buffer"),
            contents: data,
            usage: BufferUsage::VERTEX,
        })
    }

    /// Upload the index buffer to the GPU.
    fn upload_index_buffer(&self, device: &Device, indices: &[DrawIdx]) -> Buffer {
        let data = bytemuck::cast_slice(&indices);
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("imgui-wgpu index buffer"),
            contents: data,
            usage: BufferUsage::INDEX,
        })
    }

    /// Updates the texture on the GPU corresponding to the current imgui font atlas.
    ///
    /// This has to be called after loading a font.
    pub fn reload_font_texture(&mut self, imgui: &mut Context, device: &Device, queue: &Queue) {
        let mut fonts = imgui.fonts();
        // Remove possible font atlas texture.
        self.textures.remove(fonts.tex_id);

        // Create font texture and upload it.
        let handle = fonts.build_rgba32_texture();
        let font_texture = TextureConfig::new(handle.width, handle.height)
            .set_label("imgui-wgpu font atlas")
            .build(&device, self);
        font_texture.write(&queue, handle.data, handle.width, handle.height);
        fonts.tex_id = self.textures.insert(font_texture);
        // Clear imgui texture data to save memory.
        fonts.clear_tex_data();
    }
}
