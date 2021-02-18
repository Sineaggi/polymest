use std::borrow::Cow;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use crate::blit::Blit;

pub mod blit;

async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::BackendBit::all());
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    // Load the shaders from disk
    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        flags: wgpu::ShaderFlags::all(),
    });

    let fractal_module = device.create_shader_module(&wgpu::include_spirv!("fractal.comp.spv"));

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let swapchain_format = adapter.get_swap_chain_preferred_format(&surface);

    // todo: start with compute shader
    //  alternatively a mesh shader?
    //  and then copy that result to the screen image
    //  so first thing is to create an image using compute, then render
    // todo: secondly we need to make the compute shader start handling a doom style renderer

    //let fractal_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    //    label: None,
    //    layout: &(),
    //    entries: &[]
    //});

    let storage_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: 1024,
            height: 1024,
            depth: 1
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsage::STORAGE | wgpu::TextureUsage::COPY_SRC
    });

    let blit = Blit::new(&device, &storage_texture.create_view(&wgpu::TextureViewDescriptor::default()));
    //blot.draw();

    let fractual_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStage::COMPUTE,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: wgpu::TextureFormat::Rgba8Unorm,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None
        }]
    });

    /*
    var textureView = storageTexture.createView(GPUTextureViewDescriptor.builder()
                .label("")
                .build());
     */
    let texture_view = storage_texture.create_view(&wgpu::TextureViewDescriptor::default());


    let fractal_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &fractual_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&texture_view)
        }]
    });

    let fractal_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&fractual_bind_group_layout],
        push_constant_ranges: &[]
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&fractal_layout),
        module: &fractal_module,
        entry_point: "main"
    });

    /*
    let blit_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[]
    });

    let blit_shad = device.create_shader_module(&ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("blit.wgsl"))),
        flags: wgpu::ShaderFlags::all(),
    });

    let blit_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&blit_bind_group_layout],
        push_constant_ranges: &[]
    });

    let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
            targets: &[swapchain_format.into()],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
    });
     */

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[swapchain_format.into()],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
    });

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&instance, &adapter, &shader, &pipeline_layout);

        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Recreate the swap chain with the new size
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                swap_chain = device.create_swap_chain(&surface, &sc_desc);
            }
            Event::RedrawRequested(_) => {
                let frame = swap_chain
                    .get_current_frame()
                    .expect("Failed to acquire next swap chain texture")
                    .output;

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: None
                    });
                    cpass.set_pipeline(&compute_pipeline);
                    cpass.set_bind_group(0, &fractal_bind_group, &[]);
                    cpass.dispatch(1024, 1024, 1);
                }

                {
                    // todo: instead of copyin the texture, need to blit it via a quad
                    blit.draw(&mut encoder, &frame.view);
                }

                /*
                encoder.copy_texture_to_texture(TextureCopyView {

                },
                                                TextureCopyView {
                                                    texture: frame,
                                                    mip_level: 0,
                                                    origin: Default::default()
                                                },
                                                Extent3d {
                    width: 256,
                    height: 256,
                    depth: 0
                });
                 */

                //let mut encoder =
                //    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&render_pipeline);
                    rpass.draw(0..3, 0..1);
                }

                queue.submit(vec!(encoder.finish()));
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

fn main() {
    polymest::kekso();
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    {
        wgpu_subscriber::initialize_default_subscriber(None);
        // Temporarily avoid srgb formats for the swapchain on the web
        pollster::block_on(run(event_loop, window));
    }
}
