#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::{egui, egui_wgpu, wgpu};
use glam::{Mat4, Vec3};
use std::time::Instant;
use wgpu::util::DeviceExt;
const UNIFORM_FLOATS: usize = 404;
const UNIFORM_STRIDE: usize = 448;
const STEP: f32 = 1.0 / 120.0;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const COPPER: egui::Color32 = egui::Color32::from_rgb(214, 158, 104);

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct Settings {
    groom: GroomDocument,
    surface_view: bool,
    god_opts: bool,
    god_density_lod: bool,
    god_curves: bool,
    god_frames: bool,
    god_materials: bool,
    god_coverage: bool,
    #[serde(default)]
    wind_sources: Option<Vec<WindSource>>,
    selected_wind: usize,
    interaction: f32,
    friction: f32,
    iterations: u32,
    frame_limit: u32,
    tool_grab: bool,
    grab_active: bool,
    grab_anchor: Vec3,
    grab_target: Vec3,
    grab_token: f32,
    guides: u32,
    fibers: u32,
    drawn: u32,
    style: u32,
    color: u32,
    wind: f32,
    memory: f32,

    quality: u32,
    custom_environment: String,
    custom_generation: u32,
    env_mode: u32,
    sky_quality: u32,
    lighting_quality: u32,
    sky_rotation: f32,
    sky_exposure: f32,
    show_sky: bool,
    sun_azimuth: f32,
    sun_elevation: f32,
    sun_strength: f32,
    moon_strength: f32,
    moon_phase: f32,
    studio_strength: f32,
    environment_strength: f32,
    stars: f32,
    time_of_day: f32,
    studio_position: Vec3,
    light_size: f32,
    show_light: bool,
    sun_size: f32,
    root_density: f32,
    clump_width: f32,
    camera_pan: Vec3,
    fov: f32,
    curl_radius: f32,
    curl_tightness: f32,
    hair_volume: f32,
    fringe: f32,

    light_rig: u32,
    clumping: f32,
    exposure: f32,
    roughness: f32,
    diameter: f32,
    flyaways: f32,
    optical_density: f32,
    debug_lobe: u32,
    length: f32,
    motion: bool,
    lod: bool,
    paused: bool,
    source: Vec3,
    azimuth: f32,
    elevation: f32,
    spread: f32,
    strength: f32,
    duration: f32,
    pulse: f32,
    show_source: bool,
    yaw: f32,
    pitch: f32,
    distance: f32,
    time: f32,
    angle: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            wind_sources: Some(vec![WindSource::default()]),
            selected_wind: 0,
            interaction: 0.6,
            friction: 0.45,
            iterations: 24,
            groom: GroomDocument::default(),
            surface_view: false,
            god_opts: false,
            god_density_lod: false,
            god_curves: true,
            god_frames: true,
            god_materials: true,
            god_coverage: true,
            frame_limit: 60,
            tool_grab: false,
            grab_active: false,
            grab_anchor: Vec3::ZERO,
            grab_target: Vec3::ZERO,
            grab_token: 0.0,
            guides: 4096,
            fibers: 200000,
            drawn: 200000,
            style: 5,
            color: 0,
            wind: 0.0,
            memory: 0.9,

            quality: 1,
            custom_environment: String::new(),
            custom_generation: 0,
            env_mode: 1,
            sky_quality: 0,
            lighting_quality: 0,
            sky_rotation: 0.0,
            sky_exposure: 0.0,
            show_sky: true,
            sun_azimuth: 180.0,
            sun_elevation: 45.0,
            sun_strength: 0.0,
            moon_strength: 0.0,
            moon_phase: 1.0,
            studio_strength: 1.0,
            environment_strength: 0.5,
            stars: 1.0,
            time_of_day: 12.0,
            studio_position: Vec3::new(-2.8, 3.5, 3.0),
            light_size: 0.16,
            show_light: false,
            sun_size: 0.01,
            root_density: 1.0,
            clump_width: 1.0,
            camera_pan: Vec3::ZERO,
            fov: 40.1,
            curl_radius: 1.0,
            curl_tightness: 1.0,
            hair_volume: 1.0,
            fringe: 1.0,

            light_rig: 0,
            clumping: 0.3,
            exposure: 0.0,
            roughness: 0.36,
            diameter: 0.65,
            flyaways: 0.025,
            optical_density: 1.0,
            debug_lobe: 0,
            length: 1.0,
            motion: false,
            lod: true,
            paused: false,
            source: Vec3::new(-1.3, 1.4, 1.2),
            azimuth: 133.0,
            elevation: -13.0,
            spread: 45.0,
            strength: 6.0,
            duration: 1.2,
            pulse: 0.0,
            show_source: true,
            yaw: 0.37,
            pitch: 0.06,
            distance: 4.9,
            time: 0.0,
            angle: 0.0,
        }
    }
}
struct Gpu {
    rs: egui_wgpu::RenderState,
    environment: EnvironmentGpu,
    uniform: wgpu::Buffer,
    strands: wgpu::Buffer,
    roots: wgpu::Buffer,
    collider: wgpu::Buffer,
    root_cache: RootCache,
    update_roots: wgpu::ComputePipeline,
    grid: wgpu::Buffer,
    clumps: wgpu::Buffer,
    bonds: wgpu::Buffer,
    post: wgpu::RenderPipeline,
    post_group: Option<wgpu::BindGroup>,
    hdr: Option<wgpu::Texture>,
    clear: wgpu::ComputePipeline,
    clear_rest: wgpu::ComputePipeline,
    seed_rest: wgpu::ComputePipeline,
    scatter: wgpu::ComputePipeline,
    shade: wgpu::ComputePipeline,
    geometry: wgpu::Buffer,
    vertices: u32,
    init: wgpu::ComputePipeline,
    sim: wgpu::ComputePipeline,
    hair: wgpu::RenderPipeline,
    god_hair: wgpu::RenderPipeline,
    god_layout: bool,
    reuse_curves: bool,
    wake_sender: Option<std::sync::mpsc::Sender<wgpu::SubmissionIndex>>,
    frame_angle: f32,
    last_segments: u32,
    last_sky_revision: u32,
    scene: wgpu::RenderPipeline,
    compute_group: wgpu::BindGroup,
    hair_group: wgpu::BindGroup,
    scene_group: wgpu::BindGroup,
    target: Option<wgpu::Texture>,
    msaa: Option<wgpu::Texture>,
    depth: Option<wgpu::Texture>,
    texture_id: Option<egui::TextureId>,
    size: [u32; 2],
    guide_count: u32,
    arena: Vec<f32>,
    last_parameters: Option<[f32; UNIFORM_FLOATS]>,
    shadow_parameters: Option<Vec<f32>>,
    profiler: Profiler,
    curves: wgpu::Buffer,
    prepare_curves: wgpu::ComputePipeline,
    curve_group: wgpu::BindGroup,
    reference_curves: bool,
    in_flight: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}
impl Gpu {
    fn strand_buffer(device: &wgpu::Device, count: u32) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Guide storage"),
            size: count as u64 * 25 * 32,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        })
    }
    fn strand_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        uniform: &wgpu::Buffer,
        strands: &wgpu::Buffer,
        resources: &[(u32, &wgpu::Buffer)],
    ) -> wgpu::BindGroup {
        let mut entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: uniform,
                    offset: 0,
                    size: std::num::NonZeroU64::new((UNIFORM_FLOATS * 4) as u64),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: strands.as_entire_binding(),
            },
        ];
        for (binding, buffer) in resources {
            entries.push(wgpu::BindGroupEntry {
                binding: *binding,
                resource: buffer.as_entire_binding(),
            });
        }
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &entries,
        })
    }

    fn new(rs: egui_wgpu::RenderState) -> Self {
        let device = &rs.device;
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Parameters"),
            size: (7 * UNIFORM_STRIDE * 4) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let entries = [
            (0, wgpu::BufferBindingType::Uniform),
            (1, wgpu::BufferBindingType::Storage { read_only: false }),
            (2, wgpu::BufferBindingType::Storage { read_only: false }),
            (3, wgpu::BufferBindingType::Storage { read_only: false }),
            (4, wgpu::BufferBindingType::Storage { read_only: false }),
            (6, wgpu::BufferBindingType::Storage { read_only: true }),
            (7, wgpu::BufferBindingType::Storage { read_only: true }),
        ]
        .map(|(binding, ty)| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset: binding == 0,
                min_binding_size: None,
            },
            count: None,
        });
        let compute_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &entries,
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&compute_layout],
            push_constant_ranges: &[],
        });
        let sim_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Physics"),
            source: wgpu::ShaderSource::Wgsl(include_str!("simulation.wgsl").into()),
        });
        let compute = |entry| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: Some(&layout),
                module: &sim_shader,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        };
        let init = compute("initialize");
        let update_roots = compute("updateRoots");
        let sim = compute("simulate");
        let clear = compute("clearGrid");
        let clear_rest = compute("clearRest");
        let seed_rest = compute("seedRest");
        let scatter = compute("scatterGrid");
        let shade = compute("shadeGrid");
        let grid = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Hair volume"),
            size: 32768 * 36,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let bonds = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Clump bonds"),
            size: 8192 * 25 * 4,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let clumps = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Moving clump field"),
            size: 4800 * 28,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let post_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("HDR output"),
            source: wgpu::ShaderSource::Wgsl(include_str!("post.wgsl").into()),
        });
        let post = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Tone map"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &post_shader,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &post_shader,
                entry_point: Some("fragment"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        let hair_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Hair"),
            source: wgpu::ShaderSource::Wgsl(include_str!("hair.wgsl").into()),
        });
        let scene_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Scene"),
            source: wgpu::ShaderSource::Wgsl(include_str!("scene.wgsl").into()),
        });
        let attributes =
            wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Float32x3,3=>Float32];
        let buffers = [wgpu::VertexBufferLayout {
            array_stride: 40,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &attributes,
        }];
        let render = |module: &wgpu::ShaderModule,
                      topology,
                      vertices: &[wgpu::VertexBufferLayout<'_>],
                      layout: Option<&wgpu::PipelineLayout>| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout,
                vertex: wgpu::VertexState {
                    module,
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: vertices,
                },
                fragment: Some(wgpu::FragmentState {
                    module,
                    entry_point: Some("fragment"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba16Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24Plus,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 4,
                    alpha_to_coverage_enabled: false,
                    ..Default::default()
                },
                multiview: None,
                cache: None,
            })
        };
        let hair = render(
            &hair_shader,
            wgpu::PrimitiveTopology::TriangleStrip,
            &[],
            None,
        );
        let scene = render(
            &scene_shader,
            wgpu::PrimitiveTopology::TriangleList,
            &buffers,
            None,
        );
        let god_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GOD OPTS hair"),
            source: wgpu::ShaderSource::Wgsl(include_str!("hairGod.wgsl").into()),
        });
        let god_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shared hair layouts"),
            bind_group_layouts: &[
                &hair.get_bind_group_layout(0),
                &hair.get_bind_group_layout(1),
            ],
            push_constant_ranges: &[],
        });
        let god_hair = render(
            &god_shader,
            wgpu::PrimitiveTopology::TriangleStrip,
            &[],
            Some(&god_pipeline_layout),
        );
        let environment = EnvironmentGpu::new(&rs, &hair, &scene, &post);
        let strands = Self::strand_buffer(device, 4096);
        let roots = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surface follicles"),
            size: (ROOT_CAPACITY * std::mem::size_of::<RootGpu>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let collider = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Anatomical head collision"),
            contents: include_bytes!("../../model/collision.bin"),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let compute_group = Self::strand_group(
            device,
            &compute_layout,
            &uniform,
            &strands,
            &[
                (2, &grid),
                (3, &clumps),
                (4, &bonds),
                (6, &roots),
                (7, &collider),
            ],
        );
        let curves = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shared guide curves"),
            size: 8192 * 97 * 32,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Guide curve preparation"),
            source: wgpu::ShaderSource::Wgsl(include_str!("curves.wgsl").into()),
        });
        let curve_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let curve_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&curve_layout],
                push_constant_ranges: &[],
            });
        let prepare_curves = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Guide curve preparation"),
            layout: Some(&curve_pipeline_layout),
            module: &module,
            entry_point: Some("prepareCurves"),
            compilation_options: Default::default(),
            cache: None,
        });
        let curve_group = Self::strand_group(
            device,
            &prepare_curves.get_bind_group_layout(0),
            &uniform,
            &strands,
            &[(5, &curves), (6, &roots), (7, &collider)],
        );
        let hair_group = Self::strand_group(
            device,
            &hair.get_bind_group_layout(0),
            &uniform,
            &strands,
            &[(2, &grid), (5, &curves), (6, &roots), (7, &collider)],
        );
        let scene_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &scene.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform,
                        offset: 0,
                        size: std::num::NonZeroU64::new((UNIFORM_FLOATS * 4) as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: grid.as_entire_binding(),
                },
            ],
        });
        let bytes = include_bytes!("scene.bin");
        let geometry = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mannequin"),
            contents: bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });
        let profiler = Profiler::new(&rs.device);
        Self {
            in_flight: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            curves,
            prepare_curves,
            curve_group,
            reference_curves: false,
            arena: vec![0.0; UNIFORM_STRIDE * 7],
            last_parameters: None,
            shadow_parameters: None,
            profiler,
            rs,
            roots,
            collider,
            root_cache: RootCache::default(),
            update_roots,
            environment,
            uniform,
            strands,
            grid,
            clumps,
            bonds,
            post,
            post_group: None,
            hdr: None,
            clear,
            clear_rest,
            seed_rest,
            scatter,
            shade,
            geometry,
            vertices: bytes.len() as u32 / 40,
            init,
            sim,
            hair,
            god_hair,
            god_layout: false,
            reuse_curves: true,
            wake_sender: None,
            frame_angle: 0.0,
            last_segments: 0,
            last_sky_revision: 0,
            scene,
            compute_group,
            hair_group,
            scene_group,
            target: None,
            msaa: None,
            depth: None,
            texture_id: None,
            size: [0, 0],
            guide_count: 4096,
        }
    }
    fn resize(&mut self, size: [u32; 2]) {
        if self.size == size {
            return;
        }
        let device = &self.rs.device;
        let texture = |format, samples, usage| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Viewport"),
                size: wgpu::Extent3d {
                    width: size[0],
                    height: size[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };
        let target = texture(
            FORMAT,
            1,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        );
        if let Some(id) = self.texture_id.take() {
            self.rs.renderer.write().free_texture(&id);
        }
        self.texture_id = Some(self.rs.renderer.write().register_native_texture(
            device,
            &target.create_view(&Default::default()),
            wgpu::FilterMode::Linear,
        ));
        self.target = Some(target);

        let hdr = texture(
            wgpu::TextureFormat::Rgba16Float,
            1,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        self.post_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("HDR resolve"),
            layout: &self.post.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.uniform,
                        offset: 0,
                        size: std::num::NonZeroU64::new((UNIFORM_FLOATS * 4) as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(
                        &hdr.create_view(&Default::default()),
                    ),
                },
            ],
        }));
        self.hdr = Some(hdr);
        self.msaa = Some(texture(
            wgpu::TextureFormat::Rgba16Float,
            4,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        ));
        self.depth = Some(texture(
            wgpu::TextureFormat::Depth24Plus,
            4,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        ));
        self.size = size;
        self.last_parameters = None;
    }
    fn render(&mut self, s: &Settings, steps: u32, mut reset: bool) -> Mat4 {
        self.environment
            .poll(&self.rs, [&self.hair, &self.scene, &self.post], s);
        if self.last_sky_revision != self.environment.revision {
            self.last_sky_revision = self.environment.revision;
            self.last_parameters = None;
        }
        let roots_changed = self.root_cache.update(s, &self.rs.queue, &self.roots);
        if roots_changed {
            self.last_parameters = None;
        }
        let segments = curve_segments(s, self.size[1]);
        let segments_changed = self.last_segments != segments;
        self.last_segments = segments;
        let device = &self.rs.device;
        let god_changed = self.god_layout != (s.god_opts && s.god_frames);
        let curve_mode_changed = self.reuse_curves != (!s.god_opts || s.god_curves);
        self.reuse_curves = !s.god_opts || s.god_curves;
        let god_frames_dirty = segments_changed
            || curve_mode_changed
            || god_changed
            || (s.god_opts && s.god_frames && self.frame_angle != s.angle);
        self.frame_angle = s.angle;
        if god_changed {
            self.god_layout = s.god_opts && s.god_frames;
            self.curves = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Guide curves and experimental frames"),
                size: 8192 * 97 * 32 * if s.god_opts && s.god_frames { 2 } else { 1 },
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
            self.hair_group = Self::strand_group(
                device,
                &self.hair.get_bind_group_layout(0),
                &self.uniform,
                &self.strands,
                &[
                    (2, &self.grid),
                    (5, &self.curves),
                    (6, &self.roots),
                    (7, &self.collider),
                ],
            );
            self.curve_group = Self::strand_group(
                device,
                &self.prepare_curves.get_bind_group_layout(0),
                &self.uniform,
                &self.strands,
                &[(5, &self.curves), (6, &self.roots), (7, &self.collider)],
            );
            self.last_parameters = None;
        }
        if self.guide_count != s.guides {
            self.strands = Self::strand_buffer(device, s.guides);
            self.compute_group = Self::strand_group(
                device,
                &self.init.get_bind_group_layout(0),
                &self.uniform,
                &self.strands,
                &[
                    (2, &self.grid),
                    (3, &self.clumps),
                    (4, &self.bonds),
                    (6, &self.roots),
                    (7, &self.collider),
                ],
            );
            self.hair_group = Self::strand_group(
                device,
                &self.hair.get_bind_group_layout(0),
                &self.uniform,
                &self.strands,
                &[
                    (2, &self.grid),
                    (5, &self.curves),
                    (6, &self.roots),
                    (7, &self.collider),
                ],
            );
            self.curve_group = Self::strand_group(
                device,
                &self.prepare_curves.get_bind_group_layout(0),
                &self.uniform,
                &self.strands,
                &[(5, &self.curves), (6, &self.roots), (7, &self.collider)],
            );
            self.guide_count = s.guides;
            reset = true;
        }
        let target = Vec3::new(
            0.0,
            match s.style {
                3 | 4 => 1.35,
                5 => 0.95,
                0 | 1 => -0.05,
                _ => 0.72,
            },
            0.0,
        );
        let target = target + s.camera_pan;
        let eye = target
            + Vec3::new(
                s.yaw.sin() * s.pitch.cos(),
                s.pitch.sin(),
                s.yaw.cos() * s.pitch.cos(),
            ) * s.distance;
        let matrix = Mat4::perspective_rh(
            s.fov.to_radians(),
            self.size[0] as f32 / self.size[1] as f32,
            0.05,
            40.0,
        ) * Mat4::look_at_rh(eye, target, Vec3::Y);
        let mut p = [0.0f32; UNIFORM_FLOATS];
        p[..16].copy_from_slice(&matrix.to_cols_array());

        p[324..340].copy_from_slice(&matrix.inverse().to_cols_array());
        p[340..344].copy_from_slice(&[
            s.env_mode as f32,
            s.sky_rotation.to_radians(),
            s.sky_exposure,
            if s.show_sky { 1.0 } else { 0.0 },
        ]);
        p[344..348].copy_from_slice(&[
            s.sun_azimuth.to_radians(),
            s.sun_elevation.to_radians(),
            s.sun_strength,
            s.moon_phase,
        ]);
        p[348..352].copy_from_slice(&[s.curl_radius, s.curl_tightness, s.hair_volume, s.fringe]);
        p[352..388].copy_from_slice(bytemuck::cast_slice(&self.environment.sh));
        p[388..392].copy_from_slice(&[
            s.sun_size,
            s.moon_strength,
            if s.god_opts {
                ((s.god_frames as u32)
                    | ((s.god_materials as u32) << 1)
                    | ((s.god_coverage as u32) << 2)) as f32
            } else {
                0.0
            },
            s.studio_strength,
        ]);
        let background_width = (1024u32 << effective_sky_tier(s.sky_quality, s.quality)) as f32;
        let lighting_width = (512u32 << effective_sky_tier(s.lighting_quality, s.quality)) as f32;
        p[392..396].copy_from_slice(&[
            (self.environment.width as f32 / background_width)
                .log2()
                .max(0.0),
            (self.environment.width as f32 / lighting_width)
                .log2()
                .max(0.0),
            s.environment_strength,
            s.stars,
        ]);
        p[396..400].copy_from_slice(&[
            s.studio_position.x,
            s.studio_position.y,
            s.studio_position.z,
            s.light_size,
        ]);
        p[400..404].copy_from_slice(&[
            s.root_density,
            s.clump_width,
            segments as f32,
            if self.reference_curves || (s.god_opts && !s.god_curves) {
                1.0
            } else {
                0.0
            },
        ]);
        p[16..20].copy_from_slice(&[eye.x, eye.y, eye.z, 1.0]);
        p[20..24].copy_from_slice(&[
            s.time,
            STEP,
            s.angle,
            if s.surface_view { 1.0 } else { 0.0 },
        ]);

        p[316..320].copy_from_slice(&[s.quality as f32, s.clumping, s.exposure, s.optical_density]);
        p[320..324].copy_from_slice(&[s.roughness, s.debug_lobe as f32, s.diameter, s.flyaways]);
        p[24..28].copy_from_slice(&[s.length, s.wind, s.memory, s.style as f32]);
        p[28..32].copy_from_slice(&[
            s.guides as f32,
            s.drawn as f32,
            s.fibers as f32,
            s.color as f32,
        ]);
        p[32..36].copy_from_slice(&[
            self.size[0] as f32,
            self.size[1] as f32,
            0.0,
            s.light_rig as f32,
        ]);
        p[44..48].copy_from_slice(&[
            s.interaction,
            s.friction,
            if s.grab_active { 1.0 } else { 0.0 },
            s.iterations as f32,
        ]);
        p[48..52].copy_from_slice(&[s.grab_target.x, s.grab_target.y, s.grab_target.z, 0.18]);
        p[52..56].copy_from_slice(&[
            s.grab_anchor.x,
            s.grab_anchor.y,
            s.grab_anchor.z,
            s.grab_token,
        ]);
        pack_wind_sources(&mut p, s, s.time);
        self.profiler.set_tag(
            ((s.quality as u64) << 8)
                | ((s.god_opts as u64) << 7)
                | ((s.god_curves as u64) << 6)
                | ((s.god_frames as u64) << 5)
                | ((s.god_materials as u64) << 4)
                | ((s.god_coverage as u64) << 3)
                | ((s.god_density_lod as u64) << 2),
        );
        self.profiler.poll(&self.rs.device);
        let unchanged = !reset
            && steps == 0
            && self.last_parameters.as_ref() == Some(&p)
            && self.environment.pending.is_none();
        if unchanged {
            return matrix;
        }
        self.last_parameters = Some(p);
        let shadow_key: Vec<f32> = [
            &p[22..23],
            &p[28..32],
            &p[35..36],
            &p[316..320],
            &p[344..348],
            &p[388..392],
            &p[396..404],
        ]
        .concat();
        let shadow_dirty = reset
            || roots_changed
            || steps > 0
            || self.shadow_parameters.as_ref() != Some(&shadow_key);
        if s.quality > 0 {
            self.shadow_parameters = Some(shadow_key);
        }
        self.profiler.begin();
        let arena = &mut self.arena;
        arena[..UNIFORM_FLOATS].copy_from_slice(&p);
        for step in 0..steps {
            let slot = (step as usize + 1) * UNIFORM_STRIDE;
            let mut sample = p;
            let (time, _) = substep_clock(s.time, 0.0, false, step, steps);
            sample[20] = time;
            sample[22] = if s.motion {
                0.26 * (time * 0.8).sin() * (time * 0.31).sin()
            } else {
                s.angle
            };
            pack_wind_sources(&mut sample, s, time);
            arena[slot..slot + UNIFORM_FLOATS].copy_from_slice(&sample);
        }
        self.rs.queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::cast_slice(&arena[..(steps as usize + 1) * UNIFORM_STRIDE]),
        );
        let mut encoder = device.create_command_encoder(&Default::default());
        if reset || roots_changed || steps > 0 || self.profiler.active {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Physics"),
                timestamp_writes: self.profiler.compute(0),
            });
            pass.set_bind_group(0, &self.compute_group, &[0]);
            if reset {
                pass.set_pipeline(&self.clear_rest);
                pass.dispatch_workgroups(256, 1, 1);
                pass.set_pipeline(&self.init);
                pass.dispatch_workgroups(s.guides.div_ceil(64), 1, 1);
                pass.set_pipeline(&self.seed_rest);
                pass.dispatch_workgroups((s.guides * 25).div_ceil(128), 1, 1);
                pass.set_pipeline(&self.clear);
                pass.dispatch_workgroups(256, 1, 1);
                pass.set_pipeline(&self.scatter);
                pass.dispatch_workgroups((s.guides * 25).div_ceil(128), 1, 1);
            }
            if roots_changed && !reset {
                pass.set_pipeline(&self.update_roots);
                pass.dispatch_workgroups(s.guides.div_ceil(64), 1, 1);
                pass.set_pipeline(&self.clear_rest);
                pass.dispatch_workgroups(256, 1, 1);
                pass.set_pipeline(&self.seed_rest);
                pass.dispatch_workgroups((s.guides * 25).div_ceil(128), 1, 1);
                pass.set_pipeline(&self.clear);
                pass.dispatch_workgroups(256, 1, 1);
                pass.set_pipeline(&self.scatter);
                pass.dispatch_workgroups((s.guides * 25).div_ceil(128), 1, 1);
            }
            pass.set_pipeline(&self.sim);
            for step in 0..steps {
                pass.set_bind_group(
                    0,
                    &self.compute_group,
                    &[(step + 1) * UNIFORM_STRIDE as u32 * 4],
                );
                pass.set_pipeline(&self.clear);
                pass.dispatch_workgroups(256, 1, 1);
                pass.set_pipeline(&self.scatter);
                pass.dispatch_workgroups((s.guides * 25).div_ceil(128), 1, 1);
                pass.set_pipeline(&self.sim);
                pass.dispatch_workgroups(s.guides.div_ceil(64), 1, 1);
            }
        }

        if reset || roots_changed || steps > 0 || god_frames_dirty || self.profiler.active {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Guide curves"),
                timestamp_writes: self.profiler.compute(1),
            });
            if (reset || roots_changed || steps > 0 || god_frames_dirty)
                && (!s.god_opts || s.god_curves || s.god_frames)
            {
                pass.set_pipeline(&self.prepare_curves);
                pass.set_bind_group(0, &self.curve_group, &[0]);
                pass.dispatch_workgroups((s.guides * 97).div_ceil(128), 1, 1);
            }
        }
        if (s.quality > 0 && shadow_dirty) || self.profiler.active {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Shadows"),
                timestamp_writes: self.profiler.compute(2),
            });
            if s.quality > 0 && shadow_dirty {
                pass.set_bind_group(0, &self.compute_group, &[0]);
                pass.set_pipeline(&self.shade);
                pass.dispatch_workgroups(256, 1, 1);
            }
        }
        let output = self
            .target
            .as_ref()
            .unwrap()
            .create_view(&Default::default());
        let hdr = self.hdr.as_ref().unwrap().create_view(&Default::default());
        let msaa = self.msaa.as_ref().unwrap().create_view(&Default::default());
        let depth = self
            .depth
            .as_ref()
            .unwrap()
            .create_view(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("MHSB"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa,
                    depth_slice: None,
                    resolve_target: Some(&hdr),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: self.profiler.render(3),
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.scene);
            pass.set_bind_group(1, &self.environment.groups[1], &[]);
            pass.set_bind_group(0, &self.scene_group, &[]);
            pass.set_vertex_buffer(0, self.geometry.slice(..));
            pass.draw(0..self.vertices, 0..1);
            pass.set_pipeline(if s.god_opts {
                &self.god_hair
            } else {
                &self.hair
            });
            pass.set_bind_group(1, &self.environment.groups[0], &[]);
            pass.set_bind_group(0, &self.hair_group, &[]);

            if !s.surface_view {
                pass.draw(0..2 * (segments + 1), 0..s.drawn);
            }
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Display transform"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: self.profiler.render(4),
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.post);
            pass.set_bind_group(1, &self.environment.groups[2], &[]);
            pass.set_bind_group(0, self.post_group.as_ref().unwrap(), &[]);
            pass.draw(0..3, 0..1);
        }
        self.profiler.resolve(&mut encoder);
        self.in_flight
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let submission = self.rs.queue.submit([encoder.finish()]);
        let in_flight = self.in_flight.clone();
        self.rs.queue.on_submitted_work_done(move || {
            in_flight.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        });
        self.profiler.map(self.rs.queue.get_timestamp_period());
        if s.god_opts
            && let Some(sender) = &self.wake_sender
        {
            let _ = sender.send(submission);
        }
        matrix
    }
}
struct Mhsb {
    gpu: Gpu,
    logo: egui::TextureHandle,
    groom_thumbnails: Vec<egui::TextureHandle>,
    wordmark: egui::TextureHandle,
    s: Settings,
    paint: PaintState,
    last: Instant,
    accumulated: f32,
    reset: bool,
    fps: f32,
    samples: u32,
    sample_time: f32,
    density_fraction: f32,
    density_mode: bool,
    fps_input: String,
    last_render: Instant,
    last_matrix: Mat4,
    tab: u8,
    presets: Vec<SavedPreset>,
    preset_name: String,
    selected_preset: usize,
    message: String,
    replay_tick: Option<u32>,
    capture_request: bool,
    capture_pending: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    capture_path: String,
    capture_width: u32,
    capture_supersample: bool,
}
impl Mhsb {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(17, 24, 34);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(31, 44, 59);
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(48, 71, 85);
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 9.0);
        style.spacing.slider_width = 130.0;
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(15.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(14.0));
        cc.egui_ctx.set_style(style);
        visuals.selection.bg_fill = COPPER;
        cc.egui_ctx.set_visuals(visuals);
        let icon = image::load_from_memory(include_bytes!("../../logo.png"))
            .expect("embedded logo")
            .to_rgba8();
        let logo = cc.egui_ctx.load_texture(
            "MHSB logo",
            egui::ColorImage::from_rgba_unmultiplied([128, 128], icon.as_raw()),
            Default::default(),
        );
        let image = image::load_from_memory(include_bytes!("../../wordmark-light.png"))
            .expect("wordmark")
            .to_rgba8();
        let wordmark = cc.egui_ctx.load_texture(
            "MHSB wordmark",
            egui::ColorImage::from_rgba_unmultiplied(
                [image.width() as usize, image.height() as usize],
                image.as_raw(),
            ),
            Default::default(),
        );

        let previews: [&[u8]; 6] = [
            include_bytes!("../../previews/style-0.png"),
            include_bytes!("../../previews/style-1.png"),
            include_bytes!("../../previews/style-2.png"),
            include_bytes!("../../previews/style-3.png"),
            include_bytes!("../../previews/style-4.png"),
            include_bytes!("../../previews/style-5.png"),
        ];
        let groom_thumbnails = previews
            .iter()
            .enumerate()
            .map(|(i, bytes)| {
                let image = image::load_from_memory(bytes)
                    .expect("Hairstyle preview")
                    .thumbnail(96, 96)
                    .to_rgba8();
                cc.egui_ctx.load_texture(
                    format!("groom-preview-{i}"),
                    egui::ColorImage::from_rgba_unmultiplied(
                        [image.width() as usize, image.height() as usize],
                        image.as_raw(),
                    ),
                    Default::default(),
                )
            })
            .collect();
        Self {
            groom_thumbnails,
            wordmark,
            gpu: Gpu::new(cc.wgpu_render_state.as_ref().expect("wgpu backend").clone())
                .with_context(cc.egui_ctx.clone()),
            logo,
            s: Settings::default(),
            paint: PaintState::default(),
            last: Instant::now(),
            accumulated: 0.0,
            reset: true,
            fps: 0.0,
            samples: 0,
            sample_time: 0.0,
            density_fraction: 1.0,
            density_mode: false,
            fps_input: "60".into(),
            last_render: Instant::now() - std::time::Duration::from_secs(1),
            last_matrix: Mat4::IDENTITY,
            tab: 0,
            presets: read_presets(),
            preset_name: "My setup".into(),
            selected_preset: 0,
            message: String::new(),
            replay_tick: None,
            capture_request: false,
            capture_pending: None,
            capture_path: std::env::current_exe()
                .unwrap_or_default()
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .join("captures/MHSB.png")
                .to_string_lossy()
                .into_owned(),
            capture_width: 0,
            capture_supersample: false,
        }
    }
}
impl eframe::App for Mhsb {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(receiver) = &self.capture_pending
            && let Ok(result) = receiver.try_recv()
        {
            self.capture_pending = None;
            self.message = match result {
                Ok(path) => format!("Saved {path}"),
                Err(error) => format!("Capture failed · {error}"),
            };
        }
        if !ctx.input(|i| i.pointer.primary_down()) {
            self.paint.end(&self.s.groom);
        }
        let _ = self.gpu.rs.device.poll(wgpu::PollType::Poll);
        let elapsed = self.last.elapsed().as_secs_f32();
        let dt = elapsed.min(0.05);
        self.last = Instant::now();
        let minimized = ctx.input(|input| input.viewport().minimized.unwrap_or(false));
        egui::TopBottomPanel::top("studio_toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.strong("MHSB 0.2.1");
                ui.separator();
                if ui
                    .selectable_label(
                        !self.s.tool_grab && self.paint.tool == PaintTool::Off,
                        "Orbit",
                    )
                    .clicked()
                {
                    self.s.tool_grab = false;
                    self.paint.tool = PaintTool::Off;
                    self.s.surface_view = false;
                }
                if ui
                    .selectable_label(self.s.tool_grab, "Grab & release")
                    .clicked()
                {
                    self.s.tool_grab = true;
                    self.paint.tool = PaintTool::Off;
                    self.s.surface_view = false;
                }
                if ui
                    .selectable_label(self.paint.tool != PaintTool::Off, "Paint")
                    .clicked()
                {
                    self.paint.tool = PaintTool::Density;
                    self.s.tool_grab = false;
                    self.tab = 7;
                }
                ui.separator();
                if ui
                    .button(if self.s.paused {
                        "▶ Play"
                    } else {
                        "Ⅱ Pause"
                    })
                    .clicked()
                {
                    self.s.paused = !self.s.paused;
                }
                if ui.button("↻ Reset").clicked() {
                    self.reset = true;
                    self.replay_tick = None;
                }
                if ui
                    .add_enabled(self.s.can_emit(), egui::Button::new("Emit selected gust ↗"))
                    .clicked()
                {
                    self.s.emit_selected();
                    self.s.paused = false;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{:.0} FPS", self.fps));
                });
            });
        });
        egui::SidePanel::left("controls")
            .exact_width(350.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.controls(ui));
            });
        self.sample_time += elapsed;
        if self.sample_time >= 1.0 {
            self.fps = self.samples as f32 / self.sample_time;
            self.samples = 0;
            self.sample_time = 0.0;
            if density_lod_eligible(&self.s, self.gpu.size[1]) {
                if let Some(t) = self.gpu.profiler.milliseconds {
                    let target = 1000.0 / self.s.frame_limit.max(1) as f32 * 0.85;
                    let ms = t.iter().sum::<f64>() as f32;
                    self.density_fraction = (self.density_fraction
                        * (target / ms.max(0.1)).sqrt().clamp(0.85, 1.08))
                    .clamp(0.0625, 1.0);
                }
            } else {
                self.density_fraction = 1.0;
            }
        }
        if self.s.god_opts && self.s.god_density_lod {
            self.gpu.profiler.enabled = true;
        }
        let density_active = density_lod_eligible(&self.s, self.gpu.size[1]);
        if density_active && !self.density_mode {
            let target = 1000.0 / self.s.frame_limit.max(1) as f32 * 0.85;
            self.density_fraction = self
                .gpu
                .profiler
                .milliseconds
                .map(|t| (target / t.iter().sum::<f64>().max(0.1) as f32).clamp(0.0625, 1.0))
                .unwrap_or(0.125);
        }
        self.density_mode = density_active;
        self.s.drawn = if self.s.frame_limit == 0 {
            self.s.fibers
        } else {
            density_lod_count(&self.s, self.gpu.size[1], self.density_fraction)
        };
        if self.reset {
            self.s.time = 0.0;
            self.s.angle = 0.0;
            self.s.stop_gusts();
            self.accumulated = 0.0;
        }
        if !self.s.paused && !minimized {
            self.accumulated = (self.accumulated + dt).min(6.0 * STEP);
        }
        let mut steps = if self.s.paused {
            0
        } else {
            (self.accumulated / STEP).floor() as u32
        };
        if !self.s.paused
            && let Some(tick) = self.replay_tick
        {
            self.s.time = tick as f32 * STEP;
            self.s.angle = if self.s.motion {
                0.26 * (self.s.time * 0.8).sin() * (self.s.time * 0.31).sin()
            } else {
                0.0
            };
            if tick == 120 {
                self.s.emit_all_gusts(1.0);
            }
            steps = 1;
        }
        let focused = ctx.input(|input| input.focused);
        let frame_limit = effective_frame_limit(self.s.frame_limit, focused);
        let interval = if frame_limit == 0 {
            0.0
        } else {
            1.0 / frame_limit as f32
        };
        let mut rendered = false;
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let size = ui.available_size().max(egui::vec2(1.0, 1.0));
                let scale = ctx.pixels_per_point().min(1.5);
                let limit = self.gpu.rs.device.limits().max_texture_dimension_2d as f32;
                if !minimized
                    && self
                        .gpu
                        .in_flight
                        .load(std::sync::atomic::Ordering::Relaxed)
                        == 0
                    && (self.last_render.elapsed().as_secs_f32() >= interval
                        || self.last_matrix == Mat4::IDENTITY)
                {
                    self.gpu.resize([
                        (size.x * scale).round().clamp(1.0, limit) as u32,
                        (size.y * scale).round().clamp(1.0, limit) as u32,
                    ]);

                    if self.reset {
                        steps = 0;
                    }
                    if self.replay_tick.is_none() && !self.s.paused {
                        self.s.time += steps as f32 * STEP;
                        if self.s.motion {
                            self.s.angle =
                                0.26 * (self.s.time * 0.8).sin() * (self.s.time * 0.31).sin();
                        }
                    }
                    self.last_matrix = self.gpu.render(&self.s, steps, self.reset);
                    self.reset = false;
                    self.accumulated = (self.accumulated - steps as f32 * STEP).max(0.0);
                    self.last_render = Instant::now();
                    rendered = true;
                }

                if self.capture_request && self.gpu.size[0] > 0 {
                    self.capture_request = false;

                    let (pending, matrix) = self.gpu.export_png(
                        &self.s,
                        self.capture_path.clone(),
                        self.capture_width,
                        self.capture_supersample,
                    );
                    self.capture_pending = Some(pending);
                    self.last_matrix = matrix;
                    self.message = "Exporting PNG…".into();
                }
                let matrix = self.last_matrix;
                let response = match self.gpu.texture_id {
                    Some(texture) => ui.add(
                        egui::Image::new((texture, size)).sense(egui::Sense::click_and_drag()),
                    ),
                    None => ui.allocate_response(size, egui::Sense::hover()),
                };
                let project = |p: Vec3| {
                    let clip = matrix * p.extend(1.0);
                    if clip.w <= 0.05 {
                        return None;
                    }
                    let p = clip.truncate() / clip.w;
                    Some(egui::pos2(
                        response.rect.left() + (p.x * 0.5 + 0.5) * size.x,
                        response.rect.top() + (0.5 - p.y * 0.5) * size.y,
                    ))
                };

                let painting = self.paint_viewport(ctx, ui, &response, matrix);
                let mut source_drag = painting;
                if self.s.show_light
                    && let Some(point) = project(self.s.studio_position)
                {
                    let handle = ui.interact(
                        egui::Rect::from_center_size(point, egui::vec2(28.0, 28.0)),
                        egui::Id::new("studio_key_handle"),
                        egui::Sense::click_and_drag(),
                    );
                    ui.painter_at(response.rect).circle_stroke(
                        point,
                        9.0,
                        egui::Stroke::new(2.0_f32, COPPER),
                    );
                    ui.painter_at(response.rect).text(
                        point + egui::vec2(12.0, -12.0),
                        egui::Align2::LEFT_CENTER,
                        "KEY",
                        egui::FontId::proportional(11.0),
                        COPPER,
                    );
                    if handle.dragged() {
                        source_drag = true;
                        let delta =
                            ctx.input(|i| i.pointer.delta()) * self.s.distance * 0.73 / size.y;
                        let right = Vec3::new(self.s.yaw.cos(), 0.0, -self.s.yaw.sin());
                        let up = Vec3::new(
                            -self.s.yaw.sin() * self.s.pitch.sin(),
                            self.s.pitch.cos(),
                            -self.s.yaw.cos() * self.s.pitch.sin(),
                        );
                        self.s.studio_position += right * delta.x - up * delta.y;
                        self.s.light_rig = 0;
                    }
                }
                if !painting
                    && self.s.tool_grab
                    && response.drag_started()
                    && !ctx.input(|i| i.modifiers.shift || i.pointer.middle_down())
                    && let Some(pointer) = response.interact_pointer_pos()
                    && let Some(anchor) = self.gpu.pick(matrix, response.rect, pointer)
                {
                    self.s.grab_anchor = anchor;
                    self.s.grab_target = anchor;
                    self.s.grab_token += 1.0;
                    self.s.grab_active = true;
                }
                if self.s.grab_active
                    && response.dragged()
                    && !ctx.input(|i| i.modifiers.shift || i.pointer.middle_down())
                {
                    let d = ctx.input(|i| i.pointer.delta()) * self.s.distance * 0.73 / size.y;
                    let right = Vec3::new(self.s.yaw.cos(), 0.0, -self.s.yaw.sin());
                    let up = Vec3::new(
                        -self.s.yaw.sin() * self.s.pitch.sin(),
                        self.s.pitch.cos(),
                        -self.s.yaw.cos() * self.s.pitch.sin(),
                    );
                    self.s.grab_target += right * d.x - up * d.y;
                    if let Some(point) = project(self.s.grab_target) {
                        ui.painter().circle_stroke(
                            point,
                            18.0,
                            egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(142, 217, 211)),
                        );
                    }
                }
                if !ctx.input(|i| i.pointer.primary_down()) {
                    self.s.grab_active = false;
                }
                if self.s.show_source {
                    let selected = self.s.selected_wind;
                    let right = Vec3::new(self.s.yaw.cos(), 0.0, -self.s.yaw.sin());
                    let up = Vec3::new(
                        -self.s.yaw.sin() * self.s.pitch.sin(),
                        self.s.pitch.cos(),
                        -self.s.yaw.cos() * self.s.pitch.sin(),
                    );
                    let scale = self.s.distance * 0.73 / size.y;
                    let mut choose = None;
                    for (index, wind) in self.s.winds_mut().iter_mut().enumerate() {
                        if let (Some(a), Some(b)) = (
                            project(wind.position),
                            project(wind.position + wind.direction()),
                        ) {
                            let handle = ui.interact(
                                egui::Rect::from_center_size(a, egui::vec2(28.0, 28.0)),
                                egui::Id::new(("wind_handle", wind.id)),
                                egui::Sense::click_and_drag(),
                            );
                            if handle.clicked() || handle.drag_started() {
                                choose = Some(index);
                            }
                            let color = if index == selected {
                                egui::Color32::from_rgb(142, 217, 211)
                            } else {
                                egui::Color32::from_rgb(120, 139, 154)
                            };
                            let painter = ui.painter_at(response.rect);
                            let delta = b - a;
                            if delta.length() > 0.001 {
                                painter.arrow(
                                    a,
                                    delta.normalized() * 48.0,
                                    egui::Stroke::new(1.6_f32, color),
                                );
                            }
                            painter.circle(
                                a,
                                7.0,
                                egui::Color32::from_rgb(24, 34, 36),
                                egui::Stroke::new(1.4_f32, color),
                            );
                            painter.text(
                                a + egui::vec2(12.0, -13.0),
                                egui::Align2::LEFT_CENTER,
                                &wind.name,
                                egui::FontId::proportional(11.0),
                                color,
                            );
                            if handle.dragged() {
                                source_drag = true;
                                let d = ctx.input(|input| input.pointer.delta()) * scale;
                                wind.position = (wind.position + right * d.x - up * d.y)
                                    .clamp(Vec3::new(-4.0, -2.5, -4.0), Vec3::splat(4.0));
                                if wind.track_groom {
                                    wind.aim();
                                }
                            }
                        }
                    }
                    if let Some(index) = choose {
                        self.s.selected_wind = index;
                    }
                }

                if response.dragged_by(egui::PointerButton::Middle)
                    || (response.dragged() && ctx.input(|i| i.modifiers.shift))
                {
                    let d = ctx.input(|i| i.pointer.delta()) * self.s.distance * 0.73 / size.y;
                    let right = Vec3::new(self.s.yaw.cos(), 0.0, -self.s.yaw.sin());
                    let up = Vec3::new(
                        -self.s.yaw.sin() * self.s.pitch.sin(),
                        self.s.pitch.cos(),
                        -self.s.yaw.cos() * self.s.pitch.sin(),
                    );
                    self.s.camera_pan += -right * d.x + up * d.y;
                } else if response.dragged() && !source_drag && !self.s.tool_grab {
                    let d = ctx.input(|i| i.pointer.delta());
                    self.s.yaw -= d.x * 0.007;
                    self.s.pitch = (self.s.pitch + d.y * 0.006).clamp(-1.55, 1.55);
                }
                if response.hovered() {
                    let scroll = ctx.input(|i| {
                        if i.raw_scroll_delta.y != 0.0 {
                            i.raw_scroll_delta.y
                        } else {
                            i.raw_scroll_delta.x
                        }
                    });

                    if ctx.input(|i| i.modifiers.shift) {
                        self.s.camera_pan.y += scroll * self.s.distance * 0.001;
                    } else {
                        self.s.distance =
                            (self.s.distance * (-scroll * 0.001).exp()).clamp(1.0, 14.0);
                    }
                    if ctx.input(|i| i.key_pressed(egui::Key::F)) {
                        self.frame_painted_groom();
                    }
                    if response.double_clicked()
                        && let Some(pointer) = response.interact_pointer_pos()
                        && let Some(point) = self.gpu.pick(matrix, response.rect, pointer)
                    {
                        let base = match self.s.style {
                            3 | 4 => 1.35,
                            5 => 0.95,
                            0 | 1 => -0.05,
                            _ => 0.72,
                        };
                        self.s.camera_pan = point - Vec3::new(0.0, base, 0.0);
                    }
                }
            });
        if rendered
            && steps > 0
            && !self.s.paused
            && let Some(tick) = self.replay_tick
        {
            if tick >= 719 {
                self.replay_tick = None;
                self.s.paused = true;
                self.message = "Replay complete · 720 fixed steps".into();
            } else {
                self.replay_tick = Some(tick + 1);
            }
        }
        if rendered {
            self.samples += 1;
        }
        let delay = if minimized {
            0.25
        } else {
            (interval - self.last_render.elapsed().as_secs_f32()).max(0.0)
        };
        let delay = if self.s.god_opts
            && self
                .gpu
                .in_flight
                .load(std::sync::atomic::Ordering::Relaxed)
                > 0
        {
            delay.max(if frame_limit == 0 {
                1.0 / 240.0
            } else {
                interval
            })
        } else {
            delay.max(0.002)
        };
        ctx.request_repaint_after(std::time::Duration::from_secs_f32(delay));
    }
}
fn main() -> eframe::Result {
    let image = image::load_from_memory(include_bytes!("../../logo.png"))
        .expect("embedded icon")
        .to_rgba8();
    let icon = egui::IconData {
        rgba: image.into_raw(),
        width: 128,
        height: 128,
    };
    let mut setup = egui_wgpu::WgpuSetupCreateNew::default();
    let descriptor = setup.device_descriptor.clone();
    setup.device_descriptor = std::sync::Arc::new(move |adapter| {
        let mut d = descriptor(adapter);
        d.required_features |= adapter.features() & wgpu::Features::TIMESTAMP_QUERY;
        d
    });
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: egui_wgpu::WgpuConfiguration {
            present_mode: wgpu::PresentMode::AutoNoVsync,
            wgpu_setup: egui_wgpu::WgpuSetup::CreateNew(setup),
            ..Default::default()
        },
        viewport: egui::ViewportBuilder::default()
            .with_icon(icon)
            .with_inner_size([1440.0, 960.0])
            .with_min_inner_size([850.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "MHSB 0.2.1 — Making Hair Sims Better",
        options,
        Box::new(move |cc| Ok(Box::new(Mhsb::new(cc)))),
    )
}

include!("profiling.rs");
include!("scheduling.rs");
include!("density_lod.rs");
include!("groom.rs");
include!("groom_seeding.rs");
include!("groom_ui.rs");
include!("environment.rs");
include!("photography.rs");
include!("hairstyles.rs");
include!("features.rs");

include!("wind.rs");
