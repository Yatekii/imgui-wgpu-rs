use imgui::{
    Context, DrawCmd::Elements, DrawData, DrawIdx, DrawList, DrawVert, TextureId, Textures,
};
use smallvec::SmallVec;
use std::error::Error;
use std::fmt;
use std::mem::size_of;
use std::sync::Arc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;

static VS_ENTRY_POINT: &str = "vs_main";
static FS_ENTRY_POINT_LINEAR: &str = "fs_main_linear";
static FS_ENTRY_POINT_SRGB: &str = "fs_main_srgb";

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

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            RendererError::BadTexture(id) => {
                write!(f, "imgui render error: bad texture id '{}'", id.id())
            }
        }
    }
}

impl Error for RendererError {}

#[allow(dead_code)]
enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

/// Config for creating a texture from raw parts
///
#[derive(Clone)]
pub struct RawTextureConfig<'a> {
    /// An optional label for the bind group used for debugging.
    pub label: Option<&'a str>,
    /// The sampler descriptor of the texture.
    pub sampler_desc: SamplerDescriptor<'a>,
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
    pub usage: TextureUsages,
    /// The mip level of the texture.
    pub mip_level_count: u32,
    /// The sample count of the texture.
    pub sample_count: u32,
    /// The dimension of the texture.
    pub dimension: TextureDimension,
    /// The sampler descriptor of the texture.
    pub sampler_desc: SamplerDescriptor<'a>,
}

impl<'a> Default for TextureConfig<'a> {
    /// Create a new texture config.
    fn default() -> Self {
        let sampler_desc = SamplerDescriptor {
            label: Some("imgui-wgpu sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        };

        Self {
            size: Extent3d {
                width: 0,
                height: 0,
                depth_or_array_layers: 1,
            },
            label: None,
            format: None,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            sampler_desc,
        }
    }
}

/// A container for a bindable texture.
pub struct Texture {
    texture: Arc<wgpu::Texture>,
    view: Arc<wgpu::TextureView>,
    bind_group: Arc<BindGroup>,
    size: Extent3d,
}

impl Texture {
    /// Create a `Texture` from its raw parts.
    /// - `bind_group`: The bind group used by the texture. If it is `None`, the bind group will be created like in `Self::new`.
    /// - `config`: The config used for creating the bind group. If `bind_group` is `Some(_)`, it will be ignored
    pub fn from_raw_parts(
        device: &Device,
        renderer: &Renderer,
        texture: Arc<wgpu::Texture>,
        view: Arc<wgpu::TextureView>,
        bind_group: Option<Arc<BindGroup>>,
        config: Option<&RawTextureConfig>,
        size: Extent3d,
    ) -> Self {
        let bind_group = bind_group.unwrap_or_else(|| {
            let config = config.unwrap();

            // Create the texture sampler.
            let sampler = device.create_sampler(&config.sampler_desc);

            // Create the texture bind group from the layout.
            Arc::new(device.create_bind_group(&BindGroupDescriptor {
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
            }))
        });

        Self {
            texture,
            view,
            bind_group,
            size,
        }
    }

    /// Create a new GPU texture width the specified `config`.
    pub fn new(device: &Device, renderer: &Renderer, config: TextureConfig) -> Self {
        // Create the wgpu texture.
        let texture = Arc::new(device.create_texture(&TextureDescriptor {
            label: config.label,
            size: config.size,
            mip_level_count: config.mip_level_count,
            sample_count: config.sample_count,
            dimension: config.dimension,
            format: config.format.unwrap_or(renderer.config.texture_format),
            usage: config.usage,
            view_formats: &[config.format.unwrap_or(renderer.config.texture_format)],
        }));

        // Extract the texture view.
        let view = Arc::new(texture.create_view(&TextureViewDescriptor::default()));

        // Create the texture sampler.
        let sampler = device.create_sampler(&config.sampler_desc);

        // Create the texture bind group from the layout.
        let bind_group = Arc::new(device.create_bind_group(&BindGroupDescriptor {
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
        }));

        Self {
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
            ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d { x: 0, y: 0, z: 0 },
                aspect: TextureAspect::All,
            },
            // source bitmap data
            data,
            // layout of the source bitmap
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            // size of the source bitmap
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
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
        self.size.depth_or_array_layers
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

/// Configuration for the renderer.
pub struct RendererConfig<'s> {
    pub texture_format: TextureFormat,
    pub depth_format: Option<TextureFormat>,
    pub sample_count: u32,
    pub shader: Option<ShaderModuleDescriptor<'s>>,
    pub vertex_shader_entry_point: Option<&'s str>,
    pub fragment_shader_entry_point: Option<&'s str>,
}

impl<'s> RendererConfig<'s> {
    /// Create a new renderer config with custom shaders.
    pub fn with_shaders(shader: ShaderModuleDescriptor<'s>) -> Self {
        RendererConfig {
            texture_format: TextureFormat::Rgba8Unorm,
            depth_format: None,
            sample_count: 1,
            shader: Some(shader),
            vertex_shader_entry_point: Some(VS_ENTRY_POINT),
            fragment_shader_entry_point: Some(FS_ENTRY_POINT_LINEAR),
        }
    }
}

impl Default for RendererConfig<'_> {
    /// Create a new renderer config with precompiled default shaders outputting linear color.
    ///
    /// If you write to a Bgra8UnormSrgb framebuffer, this is what you want.
    fn default() -> Self {
        Self::new()
    }
}

impl RendererConfig<'_> {
    /// Create a new renderer config with precompiled default shaders outputting linear color.
    ///
    /// If you write to a Bgra8UnormSrgb framebuffer, this is what you want.
    pub fn new() -> Self {
        RendererConfig {
            fragment_shader_entry_point: Some(FS_ENTRY_POINT_LINEAR),
            ..Self::with_shaders(include_wgsl!("imgui.wgsl"))
        }
    }

    /// Create a new renderer config with precompiled default shaders outputting srgb color.
    ///
    /// If you write to a Bgra8Unorm framebuffer, this is what you want.
    pub fn new_srgb() -> Self {
        RendererConfig {
            fragment_shader_entry_point: Some(FS_ENTRY_POINT_SRGB),
            ..Self::with_shaders(include_wgsl!("imgui.wgsl"))
        }
    }
}

pub struct RenderData {
    fb_size: [f32; 2],
    last_size: [f32; 2],
    last_pos: [f32; 2],
    vertex_buffer: Option<Buffer>,
    vertex_buffer_size: usize,
    index_buffer: Option<Buffer>,
    index_buffer_size: usize,
    draw_list_offsets: SmallVec<[(i32, u32); 4]>,
    render: bool,
}

pub struct Renderer {
    pipeline: RenderPipeline,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,
    /// Textures of the font atlas and all images.
    pub textures: Textures<Texture>,
    texture_layout: BindGroupLayout,
    render_data: Option<RenderData>,
    config: RendererConfig<'static>,
}

impl Renderer {
    /// Create an entirely new imgui wgpu renderer.
    pub fn new(
        imgui: &mut Context,
        device: &Device,
        queue: &Queue,
        config: RendererConfig,
    ) -> Self {
        let RendererConfig {
            texture_format,
            depth_format,
            sample_count,
            shader,
            vertex_shader_entry_point,
            fragment_shader_entry_point,
        } = config;

        // Load shaders.
        let shader_module = device.create_shader_module(shader.unwrap());

        // Create the uniform matrix buffer.
        let size = 64;
        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("imgui-wgpu uniform buffer"),
            size,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create the uniform matrix buffer bind group layout.
        let uniform_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
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
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create the texture layout for further usage.
        let texture_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("imgui-wgpu bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
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
        // Create the render pipeline.
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("imgui-wgpu pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: vertex_shader_entry_point.unwrap(),
                buffers: &[VertexBufferLayout {
                    array_stride: size_of::<DrawVert>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Unorm8x4],
                }],
                compilation_options: Default::default(),
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: sample_count,
                ..Default::default()
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: fragment_shader_entry_point.unwrap(),
                targets: &[Some(ColorTargetState {
                    format: texture_format,
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent {
                            src_factor: BlendFactor::OneMinusDstAlpha,
                            dst_factor: BlendFactor::One,
                            operation: BlendOperation::Add,
                        },
                    }),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
        });

        let mut renderer = Self {
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            textures: Textures::new(),
            texture_layout,
            render_data: None,
            config: RendererConfig {
                texture_format,
                depth_format,
                sample_count,
                shader: None,
                vertex_shader_entry_point: None,
                fragment_shader_entry_point: None,
            },
        };

        // Immediately load the font texture to the GPU.
        renderer.reload_font_texture(imgui, device, queue);

        renderer
    }

    /// Prepares buffers for the current imgui frame.  This must be
    /// called before `Renderer::split_render`, and its output must
    /// be passed to the render call.
    pub fn prepare(
        &self,
        draw_data: &DrawData,
        render_data: Option<RenderData>,
        queue: &Queue,
        device: &Device,
    ) -> RenderData {
        let fb_width = draw_data.display_size[0] * draw_data.framebuffer_scale[0];
        let fb_height = draw_data.display_size[1] * draw_data.framebuffer_scale[1];

        let mut render_data = render_data.unwrap_or_else(|| RenderData {
            fb_size: [fb_width, fb_height],
            last_size: [0.0, 0.0],
            last_pos: [0.0, 0.0],
            vertex_buffer: None,
            vertex_buffer_size: 0,
            index_buffer: None,
            index_buffer_size: 0,
            draw_list_offsets: SmallVec::<[_; 4]>::new(),
            render: false,
        });

        // If the render area is <= 0, exit here and now.
        if fb_width <= 0.0 || fb_height <= 0.0 {
            render_data.render = false;
            return render_data;
        } else {
            render_data.render = true;
        }

        // Only update matrices if the size or position changes
        if (render_data.last_size[0] - draw_data.display_size[0]).abs() > f32::EPSILON
            || (render_data.last_size[1] - draw_data.display_size[1]).abs() > f32::EPSILON
            || (render_data.last_pos[0] - draw_data.display_pos[0]).abs() > f32::EPSILON
            || (render_data.last_pos[1] - draw_data.display_pos[1]).abs() > f32::EPSILON
        {
            render_data.fb_size = [fb_width, fb_height];
            render_data.last_size = draw_data.display_size;
            render_data.last_pos = draw_data.display_pos;

            let width = draw_data.display_size[0];
            let height = draw_data.display_size[1];

            let offset_x = draw_data.display_pos[0] / width;
            let offset_y = draw_data.display_pos[1] / height;

            // Create and update the transform matrix for the current frame.
            // This is required to adapt to vulkan coordinates.
            let matrix = [
                [2.0 / width, 0.0, 0.0, 0.0],
                [0.0, 2.0 / -height, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [-1.0 - offset_x * 2.0, 1.0 + offset_y * 2.0, 0.0, 1.0],
            ];
            self.update_uniform_buffer(queue, &matrix);
        }

        render_data.draw_list_offsets.clear();

        let mut vertex_count = 0;
        let mut index_count = 0;
        for draw_list in draw_data.draw_lists() {
            render_data
                .draw_list_offsets
                .push((vertex_count as i32, index_count as u32));
            vertex_count += draw_list.vtx_buffer().len();
            index_count += draw_list.idx_buffer().len();
        }

        let mut vertices = Vec::with_capacity(vertex_count * std::mem::size_of::<DrawVertPod>());
        let mut indices = Vec::with_capacity(index_count * std::mem::size_of::<DrawIdx>());

        for draw_list in draw_data.draw_lists() {
            // Safety: DrawVertPod is #[repr(transparent)] over DrawVert and DrawVert _should_ be Pod.
            let vertices_pod: &[DrawVertPod] = unsafe { draw_list.transmute_vtx_buffer() };
            vertices.extend_from_slice(bytemuck::cast_slice(vertices_pod));
            indices.extend_from_slice(bytemuck::cast_slice(draw_list.idx_buffer()));
        }

        // Copies in wgpu must be padded to 4 byte alignment
        indices.resize(
            indices.len() + COPY_BUFFER_ALIGNMENT as usize
                - indices.len() % COPY_BUFFER_ALIGNMENT as usize,
            0,
        );

        // If the buffer is not created or is too small for the new indices, create a new buffer
        if render_data.index_buffer.is_none() || render_data.index_buffer_size < indices.len() {
            let buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("imgui-wgpu index buffer"),
                contents: &indices,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            });
            render_data.index_buffer = Some(buffer);
            render_data.index_buffer_size = indices.len();
        } else if let Some(buffer) = render_data.index_buffer.as_ref() {
            // The buffer is large enough for the new indices, so reuse it
            queue.write_buffer(buffer, 0, &indices);
        } else {
            unreachable!()
        }

        // If the buffer is not created or is too small for the new vertices, create a new buffer
        if render_data.vertex_buffer.is_none() || render_data.vertex_buffer_size < vertices.len() {
            let buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("imgui-wgpu vertex buffer"),
                contents: &vertices,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });
            render_data.vertex_buffer = Some(buffer);
            render_data.vertex_buffer_size = vertices.len();
        } else if let Some(buffer) = render_data.vertex_buffer.as_ref() {
            // The buffer is large enough for the new vertices, so reuse it
            queue.write_buffer(buffer, 0, &vertices);
        } else {
            unreachable!()
        }

        render_data
    }

    /// Render the current imgui frame.  `Renderer::prepare` must be
    /// called first, and the output render data must be kept for the
    /// lifetime of the renderpass.
    pub fn split_render<'r>(
        &'r self,
        draw_data: &DrawData,
        render_data: &'r RenderData,
        rpass: &mut RenderPass<'r>,
    ) -> RendererResult<()> {
        if !render_data.render {
            return Ok(());
        }

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
        rpass.set_vertex_buffer(0, render_data.vertex_buffer.as_ref().unwrap().slice(..));
        rpass.set_index_buffer(
            render_data.index_buffer.as_ref().unwrap().slice(..),
            IndexFormat::Uint16,
        );

        // Execute all the imgui render work.
        for (draw_list, bases) in draw_data
            .draw_lists()
            .zip(render_data.draw_list_offsets.iter())
        {
            self.render_draw_list(
                rpass,
                draw_list,
                render_data.fb_size,
                draw_data.display_pos,
                draw_data.framebuffer_scale,
                *bases,
            )?;
        }

        Ok(())
    }

    /// Render the current imgui frame.
    pub fn render<'r>(
        &'r mut self,
        draw_data: &DrawData,
        queue: &Queue,
        device: &Device,
        rpass: &mut RenderPass<'r>,
    ) -> RendererResult<()> {
        let render_data = self.render_data.take();
        self.render_data = Some(self.prepare(draw_data, render_data, queue, device));
        self.split_render(draw_data, self.render_data.as_ref().unwrap(), rpass)
    }

    /// Render a given `DrawList` from imgui onto a wgpu frame.
    fn render_draw_list<'render>(
        &'render self,
        rpass: &mut RenderPass<'render>,
        draw_list: &DrawList,
        fb_size: [f32; 2],
        clip_off: [f32; 2],
        clip_scale: [f32; 2],
        (vertex_base, index_base): (i32, u32),
    ) -> RendererResult<()> {
        let mut start = index_base;

        for cmd in draw_list.commands() {
            if let Elements { count, cmd_params } = cmd {
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
                let end = start + count as u32;
                if clip_rect[0] < fb_size[0]
                    && clip_rect[1] < fb_size[1]
                    && clip_rect[2] >= 0.0
                    && clip_rect[3] >= 0.0
                {
                    let scissors = (
                        clip_rect[0].max(0.0).floor() as u32,
                        clip_rect[1].max(0.0).floor() as u32,
                        (clip_rect[2].min(fb_size[0]) - clip_rect[0].max(0.0))
                            .abs()
                            .ceil() as u32,
                        (clip_rect[3].min(fb_size[1]) - clip_rect[1].max(0.0))
                            .abs()
                            .ceil() as u32,
                    );

                    // XXX: Work-around for wgpu issue [1] by only issuing draw
                    // calls if the scissor rect is valid (by wgpu's flawed
                    // logic). Regardless, a zero-width or zero-height scissor
                    // is essentially a no-op render anyway, so just skip it.
                    // [1]: https://github.com/gfx-rs/wgpu/issues/1750
                    if scissors.2 > 0 && scissors.3 > 0 {
                        rpass.set_scissor_rect(scissors.0, scissors.1, scissors.2, scissors.3);

                        // Draw the current batch of vertices with the renderpass.
                        rpass.draw_indexed(start..end, vertex_base, 0..1);
                    }
                }

                // Increment the index regardless of whether or not this batch
                // of vertices was drawn.
                start = end;
            }
        }
        Ok(())
    }

    /// Updates the current uniform buffer containing the transform matrix.
    fn update_uniform_buffer(&self, queue: &Queue, matrix: &[[f32; 4]; 4]) {
        let data = bytemuck::bytes_of(matrix);
        queue.write_buffer(&self.uniform_buffer, 0, data);
    }

    /// Updates the texture on the GPU corresponding to the current imgui font atlas.
    ///
    /// This has to be called after loading a font.
    pub fn reload_font_texture(&mut self, imgui: &mut Context, device: &Device, queue: &Queue) {
        let fonts = imgui.fonts();
        // Remove possible font atlas texture.
        self.textures.remove(fonts.tex_id);

        // Create font texture and upload it.
        let handle = fonts.build_rgba32_texture();
        let font_texture_cnfig = TextureConfig {
            label: Some("imgui-wgpu font atlas"),
            size: Extent3d {
                width: handle.width,
                height: handle.height,
                ..Default::default()
            },
            ..Default::default()
        };

        let font_texture = Texture::new(device, self, font_texture_cnfig);
        font_texture.write(queue, handle.data, handle.width, handle.height);
        fonts.tex_id = self.textures.insert(font_texture);
        // Clear imgui texture data to save memory.
        fonts.clear_tex_data();
    }
}
