// MIT License

// Copyright (c) 2022 AnonmousDapper

use pixels::wgpu::util::{BufferInitDescriptor, DeviceExt};
use pixels::wgpu::{
    include_wgsl, AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    BufferSize, BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoder, Device,
    Extent3d, FilterMode, FragmentState, LoadOp, MultisampleState, Operations,
    PipelineLayoutDescriptor, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor,
    ShaderStages, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension, VertexState,
};

use crate::PIPELINE_TEXTURE_FORMAT;

fn create_texture_view(
    device: &Device,
    width: u32,
    height: u32,
    format: TextureFormat,
) -> TextureView {
    device
        .create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
        })
        .create_view(&TextureViewDescriptor::default())
}

fn create_sampler(device: &Device) -> Sampler {
    device.create_sampler(&SamplerDescriptor {
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        mipmap_filter: FilterMode::Nearest,
        ..Default::default()
    })
}

pub struct ShaderPipeline {
    texture: TextureView,
    trail: TrailPass,
    bloom: BloomPass,
    hdr: HDRPass,
}

impl ShaderPipeline {
    pub fn new(pixels: &pixels::Pixels, width: u32, height: u32) -> Self {
        let device = pixels.device();

        let output_texture_format = pixels.surface_texture_format();

        let texture = Self::create_resources(device, height, width);

        let shader_module = device.create_shader_module(&include_wgsl!("shaders/vert.wgsl"));

        let vertex = VertexState {
            module: &shader_module,
            entry_point: "main",
            buffers: &[],
        };

        let trail = TrailPass::new(device, &texture, width, height, vertex.clone());

        let bloom = BloomPass::new(device, &texture, width, height, vertex.clone());

        let hdr = HDRPass::new(device, &texture, output_texture_format, vertex);

        Self {
            texture,
            trail,
            hdr,
            bloom,
        }
    }

    pub fn get_texture_view(&self) -> &TextureView {
        &self.texture
    }

    fn create_resources(device: &Device, width: u32, height: u32) -> TextureView {
        create_texture_view(device, width, height, PIPELINE_TEXTURE_FORMAT)
    }

    pub fn resize(&mut self, pixels: &pixels::Pixels, width: u32, height: u32) {
        let device = pixels.device();
        self.texture = Self::create_resources(device, width, height);

        self.trail.resize(device, &self.texture, width, height);
        self.bloom.resize(device, &self.texture, width, height);
        self.hdr.resize(device, &self.texture);
    }

    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        target: &TextureView,
        clip: (u32, u32, u32, u32),
    ) {
        self.bloom.render(encoder, &self.texture, clip);
        self.trail.render(encoder, &self.texture, clip);
        self.hdr.render(encoder, target, clip);
    }
}

pub struct TrailPass {
    mix_pipeline: RenderPipeline,
    mix_group_layout: BindGroupLayout,
    mix_group: BindGroup,
    cut_pipeline: RenderPipeline,
    cut_group_layout: BindGroupLayout,
    cut_group: BindGroup,
    cut_buffer: TextureView,
    send_buffer: TextureView,
    sampler: Sampler,
}

impl TrailPass {
    pub fn new(
        device: &Device,
        input: &TextureView,
        width: u32,
        height: u32,
        vertex: VertexState<'_>,
    ) -> Self {
        let sampler = create_sampler(device);

        let mix_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let cut_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let (cut_buffer, send_buffer, mix_group, cut_group) = Self::create_resources(
            device,
            input,
            width,
            height,
            &mix_group_layout,
            &cut_group_layout,
            &sampler,
        );

        let mix_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("trail_mix_pipeline"),
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&mix_group_layout],
                push_constant_ranges: &[],
            })),
            vertex: vertex.clone(),
            fragment: Some(FragmentState {
                module: &device.create_shader_module(&include_wgsl!("shaders/mix.wgsl")),
                entry_point: "main",
                targets: &[ColorTargetState {
                    format: PIPELINE_TEXTURE_FORMAT,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        let cut_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("trail_cut_pipeline"),
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&cut_group_layout],
                push_constant_ranges: &[],
            })),
            vertex,
            fragment: Some(FragmentState {
                module: &device.create_shader_module(&include_wgsl!("shaders/cut.wgsl")),
                entry_point: "main",
                targets: &[
                    ColorTargetState {
                        format: PIPELINE_TEXTURE_FORMAT,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    },
                    ColorTargetState {
                        format: PIPELINE_TEXTURE_FORMAT,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    },
                ],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        Self {
            mix_pipeline,
            mix_group_layout,
            mix_group,
            cut_pipeline,
            cut_group_layout,
            cut_group,
            cut_buffer,
            send_buffer,
            sampler,
        }
    }

    fn create_resources(
        device: &Device,
        input: &TextureView,
        width: u32,
        height: u32,
        mix_layout: &BindGroupLayout,
        cut_layout: &BindGroupLayout,
        sampler: &Sampler,
    ) -> (TextureView, TextureView, BindGroup, BindGroup) {
        let cut_buffer = create_texture_view(device, width, height, PIPELINE_TEXTURE_FORMAT);
        let send_buffer = create_texture_view(device, width, height, PIPELINE_TEXTURE_FORMAT);

        let mix_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("trail_mix_bind_group"),
            layout: mix_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(input),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&cut_buffer),
                },
            ],
        });

        let cut_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("trail_cut_group"),
            layout: cut_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&send_buffer),
                },
            ],
        });

        (cut_buffer, send_buffer, mix_group, cut_group)
    }

    pub fn resize(&mut self, device: &Device, input: &TextureView, width: u32, height: u32) {
        let (cut_buffer, send_buffer, mix_group, cut_group) = Self::create_resources(
            device,
            input,
            width,
            height,
            &self.mix_group_layout,
            &self.cut_group_layout,
            &self.sampler,
        );

        self.cut_buffer = cut_buffer;
        self.send_buffer = send_buffer;
        self.mix_group = mix_group;
        self.cut_group = cut_group;
    }

    pub fn render(
        &self,
        encoder: &mut CommandEncoder,
        target: &TextureView,
        clip: (u32, u32, u32, u32),
    ) {
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("trail_mix_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &self.send_buffer,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            pass.set_pipeline(&self.mix_pipeline);
            pass.set_bind_group(0, &self.mix_group, &[]);
            pass.set_scissor_rect(clip.0, clip.1, clip.2, clip.3);
            pass.draw(0..3, 0..1);
        }

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("trail_cut_pass"),
                color_attachments: &[
                    RenderPassColorAttachment {
                        view: target,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: true,
                        },
                    },
                    RenderPassColorAttachment {
                        view: &self.cut_buffer,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: true,
                        },
                    },
                ],
                depth_stencil_attachment: None,
            });

            pass.set_pipeline(&self.cut_pipeline);
            pass.set_bind_group(0, &self.cut_group, &[]);
            pass.set_scissor_rect(clip.0, clip.1, clip.2, clip.3);
            pass.draw(0..3, 0..1);
        }
    }
}

pub struct BloomPass {
    split_pipeline: RenderPipeline,
    split_group_layout: BindGroupLayout,
    split_group: BindGroup,
    blur_pipeline: RenderPipeline,
    blur_pass_h_group: BindGroup,
    blur_pass_v_group: BindGroup,
    blur_group_layout: BindGroupLayout,
    blur_even_group: BindGroup,
    blur_odd_group: BindGroup,
    combine_pipeline: RenderPipeline,
    combine_group_layout: BindGroupLayout,
    combine_group: BindGroup,
    split_full_buffer: TextureView,
    even_buffer: TextureView,
    odd_buffer: TextureView,
    sampler: Sampler,
}

impl BloomPass {
    pub fn new(
        device: &Device,
        input: &TextureView,
        width: u32,
        height: u32,
        vertex: VertexState<'_>,
    ) -> Self {
        let sampler = create_sampler(device);

        let split_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let blur_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let blur_pass_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: pixels::wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(std::mem::size_of::<u32>() as _),
                },
                count: None,
            }],
        });

        let blur_pass_h_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("bloom_blur_pass_h_group"),
            layout: &blur_pass_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: Some("bloom_uniform_h_buffer"),
                        contents: bytemuck::cast_slice(&[true as u32]),
                        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                    })
                    .as_entire_binding(),
            }],
        });

        let blur_pass_v_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("bloom_blur_pass_v_group"),
            layout: &blur_pass_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: Some("bloom_uniform_v_buffer"),
                        contents: bytemuck::cast_slice(&[false as u32]),
                        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                    })
                    .as_entire_binding(),
            }],
        });

        let combine_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let (
            even_buffer,
            odd_buffer,
            split_full_buffer,
            split_group,
            blur_even_group,
            blur_odd_group,
            combine_group,
        ) = Self::create_resources(
            device,
            input,
            width,
            height,
            &split_group_layout,
            &blur_group_layout,
            &combine_group_layout,
            &sampler,
        );

        let split_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("bloom_split_pipeline"),
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&split_group_layout],
                push_constant_ranges: &[],
            })),
            vertex: vertex.clone(),
            fragment: Some(FragmentState {
                module: &device.create_shader_module(&include_wgsl!("shaders/split_bright.wgsl")),
                entry_point: "main",
                targets: &[
                    ColorTargetState {
                        format: PIPELINE_TEXTURE_FORMAT,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    },
                    ColorTargetState {
                        format: PIPELINE_TEXTURE_FORMAT,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    },
                ],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        let blur_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("bloom_blur_pipeline"),
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&blur_group_layout, &blur_pass_group_layout],
                push_constant_ranges: &[],
            })),
            vertex: vertex.clone(),
            fragment: Some(FragmentState {
                module: &device.create_shader_module(&include_wgsl!("shaders/blur.wgsl")),
                entry_point: "main",
                targets: &[ColorTargetState {
                    format: PIPELINE_TEXTURE_FORMAT,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        let combine_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("bloom_combine_pipeline"),
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&combine_group_layout],
                push_constant_ranges: &[],
            })),
            vertex,
            fragment: Some(FragmentState {
                module: &device.create_shader_module(&include_wgsl!("shaders/combine.wgsl")),
                entry_point: "main",
                targets: &[ColorTargetState {
                    format: PIPELINE_TEXTURE_FORMAT,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        Self {
            split_pipeline,
            split_group_layout,
            split_group,
            blur_pipeline,
            blur_pass_h_group,
            blur_pass_v_group,
            blur_group_layout,
            blur_even_group,
            blur_odd_group,
            combine_pipeline,
            combine_group_layout,
            combine_group,
            split_full_buffer,
            even_buffer,
            odd_buffer,
            sampler,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn create_resources(
        device: &Device,
        input: &TextureView,
        width: u32,
        height: u32,
        split_layout: &BindGroupLayout,
        blur_layout: &BindGroupLayout,
        combine_layout: &BindGroupLayout,
        sampler: &Sampler,
    ) -> (
        TextureView,
        TextureView,
        TextureView,
        BindGroup,
        BindGroup,
        BindGroup,
        BindGroup,
    ) {
        let even_buffer = create_texture_view(device, width, height, PIPELINE_TEXTURE_FORMAT);
        let odd_buffer = create_texture_view(device, width, height, PIPELINE_TEXTURE_FORMAT);

        let split_full_buffer = create_texture_view(device, width, height, PIPELINE_TEXTURE_FORMAT);

        let split_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("bloom_split_bind_group"),
            layout: split_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(input),
                },
            ],
        });

        let blur_even_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("bloom_blur_evens_bind_group"),
            layout: blur_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&even_buffer),
                },
            ],
        });

        let blur_odd_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("bloom_blur_odds_bind_group"),
            layout: blur_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&odd_buffer),
                },
            ],
        });

        let combine_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("bloom_combine_bind_group"),
            layout: combine_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&split_full_buffer),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&even_buffer),
                },
            ],
        });

        (
            even_buffer,
            odd_buffer,
            split_full_buffer,
            split_group,
            blur_even_group,
            blur_odd_group,
            combine_group,
        )
    }

    pub fn resize(&mut self, device: &Device, input: &TextureView, width: u32, height: u32) {
        let (
            even_buffer,
            odd_buffer,
            split_full_buffer,
            split_group,
            blur_even_group,
            blur_odd_group,
            combine_group,
        ) = Self::create_resources(
            device,
            input,
            width,
            height,
            &self.split_group_layout,
            &self.blur_group_layout,
            &self.combine_group_layout,
            &self.sampler,
        );

        self.even_buffer = even_buffer;
        self.odd_buffer = odd_buffer;
        self.split_full_buffer = split_full_buffer;
        self.split_group = split_group;
        self.blur_even_group = blur_even_group;
        self.blur_odd_group = blur_odd_group;
        self.combine_group = combine_group;
    }

    pub fn render(
        &self,
        encoder: &mut CommandEncoder,
        target: &TextureView,
        clip: (u32, u32, u32, u32),
    ) {
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("bloom_split_pass"),
                color_attachments: &[
                    RenderPassColorAttachment {
                        view: &self.split_full_buffer,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: true,
                        },
                    },
                    RenderPassColorAttachment {
                        view: &self.even_buffer,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: true,
                        },
                    },
                ],
                depth_stencil_attachment: None,
            });

            pass.set_pipeline(&self.split_pipeline);
            pass.set_bind_group(0, &self.split_group, &[]);
            pass.set_scissor_rect(clip.0, clip.1, clip.2, clip.3);
            pass.draw(0..3, 0..1);
        }

        for i in 0..10 {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("bloom_blur_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: if i % 2 == 1 {
                        &self.even_buffer
                    } else {
                        &self.odd_buffer
                    },
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            pass.set_pipeline(&self.blur_pipeline);
            pass.set_bind_group(
                0,
                if i % 2 == 1 {
                    &self.blur_odd_group
                } else {
                    &self.blur_even_group
                },
                &[],
            );
            pass.set_bind_group(
                1,
                if i % 2 == 0 {
                    &self.blur_pass_h_group
                } else {
                    &self.blur_pass_v_group
                },
                &[],
            );
            pass.set_scissor_rect(clip.0, clip.1, clip.2, clip.3);
            pass.draw(0..3, 0..1);
        }

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("bloom_combine_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            pass.set_pipeline(&self.combine_pipeline);
            pass.set_bind_group(0, &self.combine_group, &[]);
            pass.set_scissor_rect(clip.0, clip.1, clip.2, clip.3);
            pass.draw(0..3, 0..1);
        }
    }
}

pub struct HDRPass {
    pipeline: RenderPipeline,
    group_layout: BindGroupLayout,
    group: BindGroup,
    sampler: Sampler,
}

impl HDRPass {
    pub fn new(
        device: &Device,
        texture: &TextureView,
        surface_format: TextureFormat,
        vertex: VertexState<'_>,
    ) -> Self {
        let shader = device.create_shader_module(&include_wgsl!("shaders/hdr.wgsl"));

        let sampler = create_sampler(device);

        let group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let group = Self::create_resources(device, &group_layout, texture, &sampler);

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("hdr_render_pipeline"),
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&group_layout],
                push_constant_ranges: &[],
            })),
            vertex,
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "main",
                targets: &[ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        Self {
            pipeline,
            group_layout,
            group,
            sampler,
        }
    }

    fn create_resources(
        device: &Device,
        layout: &BindGroupLayout,
        input: &TextureView,
        sampler: &Sampler,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("hdr_bind_group"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(input),
                },
            ],
        })
    }

    pub fn resize(&mut self, device: &Device, texture: &TextureView) {
        self.group = Self::create_resources(device, &self.group_layout, texture, &self.sampler);
    }

    pub fn render(
        &self,
        encoder: &mut CommandEncoder,
        target: &TextureView,
        clip: (u32, u32, u32, u32),
    ) {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("hdr_render_pass"),
            color_attachments: &[RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.group, &[]);
        pass.set_scissor_rect(clip.0, clip.1, clip.2, clip.3);
        pass.draw(0..3, 0..1);
    }
}
