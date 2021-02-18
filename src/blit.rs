use std::borrow::Cow::Borrowed;

pub struct Blit {
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl Blit {
    pub fn new(
        device: &wgpu::Device,
        input: &wgpu::TextureView,
    ) -> Blit {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -1000.0,
            lod_max_clamp: 1000.0,
            compare: None,
            anisotropy_clamp: None,
            ..Default::default()
        });

        let blit_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::COMPUTE,
                ty: wgpu::BindingType::Sampler {
                    filtering: false,
                    comparison: false
                },
                count: None
            }]
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("blit bind group"),
            layout: &blit_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(input),
                },
            ],
        });

        let blit_shad = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Borrowed(include_str!("blit.wgsl"))),
            flags: wgpu::ShaderFlags::all(),
        });

        let blit_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&blit_bind_group_layout],
            push_constant_ranges: &[]
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&blit_layout),
            vertex: wgpu::VertexState {
                module: &blit_shad,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_shad,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R8Unorm,
                    alpha_blend: Default::default(),
                    color_blend: Default::default(),
                    write_mask: Default::default()
                }],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        });

        return Blit {
            render_pipeline,
            bind_group
        };
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN), // clear color kinda doesn't matter but sure, green it is
                    store: true,
                },
            }],
            depth_stencil_attachment: None
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        //render_pass.draw();
        // todo: return or something?
    }
}
