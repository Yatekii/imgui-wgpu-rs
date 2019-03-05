use std::mem::size_of;
use std::slice::from_raw_parts;
use std::str::from_utf8;
use imgui::{DrawList, FrameSize, ImDrawIdx, ImDrawVert, ImGui, ImTexture, Textures, Ui};

pub type RendererResult<T> = Result<T, RendererError>;

#[derive(Clone, Debug)]
pub enum RendererError {
  VertexBufferTooSmall,
  IndexBufferTooSmall,
  BadTexture(ImTexture),
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
  pub fn new(texture: wgpu::Texture, sampler: wgpu::Sampler, layout: &wgpu::BindGroupLayout, device: &mut wgpu::Device) -> Self {

    let view = texture.create_default_view();

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
  vertex_count: u32,
  vertex_max: u32,
  index_buffer: wgpu::Buffer,
  index_count: u32,
  index_max: u32,
  textures: Textures<Texture>,
  atlas: ImTexture,
  clear_color: Option<wgpu::Color>,
}

impl Renderer {
  pub fn new(
    imgui: &mut ImGui,
    device: &mut wgpu::Device,
    format: wgpu::TextureFormat,
    clear_color: Option<wgpu::Color>,
  ) -> RendererResult<Renderer> {

    // Create shaders
    let (vs_code, fs_code) = Shaders::get_program_code();

    let vs_module = device.create_shader_module(&Shaders::compile_glsl(vs_code, ShaderStage::Vertex));
    let fs_module = device.create_shader_module(&Shaders::compile_glsl(fs_code, ShaderStage::Fragment));

    // Create uniform matrix buffer
    let size = 64;
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      size, usage: wgpu::BufferUsageFlags::UNIFORM | wgpu::BufferUsageFlags::TRANSFER_DST,
    });

    let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      bindings: &[
        wgpu::BindGroupLayoutBinding {
          binding: 0,
          visibility: wgpu::ShaderStageFlags::VERTEX,
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
          visibility: wgpu::ShaderStageFlags::FRAGMENT,
          ty: wgpu::BindingType::SampledTexture,
        },
        wgpu::BindGroupLayoutBinding {
          binding: 1,
          visibility: wgpu::ShaderStageFlags::FRAGMENT,
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
      fragment_stage: wgpu::PipelineStageDescriptor {
        module: &fs_module,
        entry_point: "main",
      },
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
          color: wgpu::BlendDescriptor {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
          },
          alpha: wgpu::BlendDescriptor {
            src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
          },
          write_mask: wgpu::ColorWriteFlags::ALL,
        },
      ],
      depth_stencil_state: None,
      index_format: wgpu::IndexFormat::Uint16,
      vertex_buffers: &[
        wgpu::VertexBufferDescriptor {
          stride: size_of::<ImDrawVert>() as u32,
          step_mode: wgpu::InputStepMode::Vertex,
          attributes: &[
            wgpu::VertexAttributeDescriptor {
              format: wgpu::VertexFormat::Float2,
              attribute_index: 0,
              offset: 0,
            },
            wgpu::VertexAttributeDescriptor {
              format: wgpu::VertexFormat::Float2,
              attribute_index: 1,
              offset: 8,
            },
            wgpu::VertexAttributeDescriptor {
              format: wgpu::VertexFormat::Uint,
              attribute_index: 2,
              offset: 16,
            },
          ]
        },
      ],
      sample_count: 1,
    });

    // Create vertex/index buffer
    let vertex_max = 4096;
    let vertex_size = size_of::<ImDrawVert>() as u32;
    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      size: vertex_max * vertex_size,
      usage: wgpu::BufferUsageFlags::VERTEX | wgpu::BufferUsageFlags::TRANSFER_DST,
    });

    let index_max = 4096;
    let index_size = size_of::<u16>() as u32;
    let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      size: vertex_max * index_size,
      usage: wgpu::BufferUsageFlags::INDEX | wgpu::BufferUsageFlags::TRANSFER_DST,
    });
    
    // Create texture
    let texture = imgui.prepare_texture(|handle| {
      let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d { width: handle.width, height: handle.height, depth: 1 },
        array_size: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsageFlags::SAMPLED | wgpu::TextureUsageFlags::TRANSFER_DST
      });

      Renderer::upload_immediate(handle.width, handle.height, handle.pixels, &texture, device);
      Result::Ok(texture)
    })?;

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      r_address_mode: wgpu::AddressMode::ClampToEdge,
      s_address_mode: wgpu::AddressMode::ClampToEdge,
      t_address_mode: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      mipmap_filter: wgpu::FilterMode::Linear,
      lod_min_clamp: -100.0,
      lod_max_clamp: 100.0,
      max_anisotropy: 0,
      compare_function: wgpu::CompareFunction::Always,
      border_color: wgpu::BorderColor::TransparentBlack,
    });

    let pair = Texture::new(texture, sampler, &texture_layout, device);
    let mut textures = Textures::new();
    let atlas = textures.insert(pair);
    imgui.set_font_texture_id(atlas);

    Ok(Renderer {
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
      atlas,
      clear_color,
    })
  }

  pub fn textures(&mut self) -> &mut Textures<Texture> {
    &mut self.textures
  }

  pub fn atlas(&mut self) -> &Texture {
    &mut self.textures.get(self.atlas).unwrap()
  }

  pub fn render<'a>(
    &mut self,
    ui: Ui<'a>,
    device: &mut wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
  ) -> RendererResult<()> {
        
    let FrameSize {
      logical_size: (width, height),
      hidpi_factor,
    } = ui.frame_size();

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

    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
        attachment: &view,
        load_op: match self.clear_color { Some(_) => wgpu::LoadOp::Clear, _ => wgpu::LoadOp::Load },
        store_op: wgpu::StoreOp::Store,
        clear_color: self.clear_color.unwrap_or(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
      }],
      depth_stencil_attachment: None,
    });

    rpass.set_pipeline(&self.pipeline);
    rpass.set_vertex_buffers(&[(&self.vertex_buffer, 0)]);
    rpass.set_index_buffer(&self.index_buffer, 0);
    self.vertex_count = 0;
    self.index_count = 0;

    self.update_uniform_buffer(&matrix)?;
    rpass.set_bind_group(0, &self.uniform_bind_group);

    ui.render(|ui, mut draw_data| {
      draw_data.scale_clip_rects(ui.imgui().display_framebuffer_scale());
      
      for draw_list in &draw_data {
        self.render_draw_list(device, &mut rpass, &draw_list, fb_size)?;
      }
      Ok(())
    })
  }

  fn render_draw_list<'data, 'render>(
      &mut self,
      device: &mut wgpu::Device,
      rpass: &mut wgpu::RenderPass<'render>,
      draw_list: &DrawList<'data>,
      fb_size: (f32, f32),
  ) -> RendererResult<()> {
      let (fb_width, fb_height) = fb_size;

      let base_vertex = self.vertex_count;
      let mut start = self.index_count;

      let vertex_buffer = self.upload_vertex_buffer(device, draw_list.vtx_buffer)?;
      let index_buffer = self.upload_index_buffer(device, draw_list.idx_buffer)?;

      //rpass.set_vertex_buffers(&[(&vertex_buffer, 0)]);
      //rpass.set_index_buffer(&index_buffer, 0);
    
      for cmd in draw_list.cmd_buffer {
        let texture_id = cmd.texture_id.into();
        let tex = self
          .textures
          .get(texture_id)
          .ok_or_else(|| RendererError::BadTexture(texture_id))?;

        rpass.set_bind_group(1, tex.bind_group());

        let end = start + cmd.elem_count;
        let scissor = (
          cmd.clip_rect.x.max(0.0).min(fb_width).round() as u16,
          cmd.clip_rect.y.max(0.0).min(fb_height).round() as u16,
          (cmd.clip_rect.z - cmd.clip_rect.x)
            .abs()
            .min(fb_width)
            .round() as u16,
          (cmd.clip_rect.w - cmd.clip_rect.y)
            .abs()
            .min(fb_height)
            .round() as u16,
        );

        rpass.draw_indexed(start..end, base_vertex as i32, 0..1);
          
        start = end;
      }
      Ok(())
  }

  fn update_uniform_buffer(
    &mut self,
    matrix: &[[f32; 4]; 4],
  ) -> RendererResult<()> {
    self.uniform_buffer.set_sub_data(0, cast_slice(matrix));
    Ok(())
  }

  fn upload_vertex_buffer(
    &mut self,
    _device: &mut wgpu::Device,
    vtx_buffer: &[ImDrawVert],
//  ) -> RendererResult<wgpu::Buffer> {
  ) -> RendererResult<()> {
    let vertex_count = vtx_buffer.len() as u32;
    if self.vertex_count + vertex_count < self.vertex_max {
      self.vertex_buffer.set_sub_data(self.vertex_count * (size_of::<ImDrawVert>() as u32), cast_slice(vtx_buffer));
      self.vertex_count += vertex_count;
      Ok(())
    }
    else {
      Err(RendererError::VertexBufferTooSmall)
    }
    /*
    let size = (vtx_buffer.len() * size_of::<ImDrawVert>()) as u32;
    let (buffer, data) = device.create_buffer_mapped(&wgpu::BufferDescriptor {
      size, usage: wgpu::BufferUsageFlags::VERTEX | wgpu::BufferUsageFlags::TRANSFER_DST | wgpu::BufferUsageFlags::MAP_WRITE,
    });

    data.copy_from_slice(&vtx_buffer);
    buffer.unmap();
    Ok(buffer)
    */
  }

  fn upload_index_buffer(
    &mut self,
    _device: &mut wgpu::Device,
    idx_buffer: &[ImDrawIdx],
//  ) -> RendererResult<wgpu::Buffer> {
  ) -> RendererResult<()> {
    let index_count = idx_buffer.len() as u32;
    if self.index_count + index_count < self.index_max {
      self.index_buffer.set_sub_data(self.index_count * (size_of::<ImDrawIdx>() as u32), cast_slice(idx_buffer));
      self.index_count += index_count;
      Ok(())
    }
    else {
      Err(RendererError::IndexBufferTooSmall)
    }
    /*
    let size = (idx_buffer.len() * size_of::<ImDrawIdx>()) as u32;
    let (buffer, data) = device.create_buffer_mapped(&wgpu::BufferDescriptor {
      size, usage: wgpu::BufferUsageFlags::VERTEX | wgpu::BufferUsageFlags::TRANSFER_DST | wgpu::BufferUsageFlags::MAP_WRITE,
    });

    data.copy_from_slice(&idx_buffer);
    buffer.unmap();
    Ok(buffer)
    */
  }

  fn upload_immediate(width: u32, height: u32, data: &[u8], target: &wgpu::Texture, device: &mut wgpu::Device) {

    // Place in wgpu buffer
    let bytes = data.len() as u32;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
      size: bytes,
      usage: wgpu::BufferUsageFlags::TRANSFER_SRC,
    });
    buffer.set_sub_data(0, data);

    // Upload immediately
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      todo: 0,
    });

    let pixel_size = bytes / width / height;
    encoder.copy_buffer_to_texture(
      wgpu::BufferCopyView {
        buffer: &buffer,
        offset: 0,
        row_pitch: pixel_size * width,
        image_height: height,
      },
      wgpu::TextureCopyView {
        texture: target,
        level: 0,
        slice: 0,
        origin: wgpu::Origin3d {
          x: 0.0,
          y: 0.0,
          z: 0.0,
        },
      },
      wgpu::Extent3d {
        width,
        height,
        depth: 1,
      },
    );

    device
      .get_queue()
      .submit(&[encoder.finish()]);
  }
}

pub fn cast_slice<T>(data: &[T]) -> &[u8] {
  unsafe { 
    from_raw_parts(data.as_ptr() as *const u8, data.len() * size_of::<T>())
  }
}
