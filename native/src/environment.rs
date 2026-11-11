use std::io::Read;
type EnvironmentKey = (u32, u32, u32);
struct EnvironmentData {
    width: u32,
    height: u32,
    levels: u32,
    bytes: Vec<u8>,
    sh: [[f32; 4]; 9],
}
#[derive(serde::Deserialize)]
struct EnvironmentInfo {
    sh: [[f32; 4]; 9],
}
struct EnvironmentGpu {
    texture: wgpu::Texture,
    sampler: wgpu::Sampler,
    groups: [wgpu::BindGroup; 3],
    current: EnvironmentKey,
    pending: Option<(
        EnvironmentKey,
        std::sync::mpsc::Receiver<Result<EnvironmentData, String>>,
    )>,
    sh: [[f32; 4]; 9],
    status: String,
    width: u32,
    revision: u32,
}
fn environment_group(
    device: &wgpu::Device,
    pipeline: &wgpu::RenderPipeline,
    texture: &wgpu::Texture,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("HDR environment"),
        layout: &pipeline.get_bind_group_layout(1),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &texture.create_view(&Default::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}
impl EnvironmentGpu {
    fn new(
        rs: &egui_wgpu::RenderState,
        hair: &wgpu::RenderPipeline,
        scene: &wgpu::RenderPipeline,
        post: &wgpu::RenderPipeline,
    ) -> Self {
        let sampler = rs.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Environment filtering"),
            address_mode_u: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let texture = rs.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Environment fallback"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        rs.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&[0x3800u16, 0x3800, 0x3800, 0x3c00]),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(8),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let groups =
            [hair, scene, post].map(|p| environment_group(&rs.device, p, &texture, &sampler));
        Self {
            texture,
            sampler,
            groups,
            current: (0, 0, 0),
            pending: None,
            sh: [[0.0; 4]; 9],
            status: "Studio".into(),
            width: 1,
            revision: 0,
        }
    }
    fn poll(
        &mut self,
        rs: &egui_wgpu::RenderState,
        pipelines: [&wgpu::RenderPipeline; 3],
        s: &Settings,
    ) {
        let background = effective_sky_tier(s.sky_quality, s.quality);
        let lighting = effective_sky_tier(s.lighting_quality, s.quality);
        let tier = background.max(lighting.saturating_sub(1));
        let wanted = (s.env_mode, tier, s.custom_generation);
        if let Some((key, receiver)) = &self.pending
            && let Ok(result) = receiver.try_recv()
        {
            let key = *key;
            self.pending = None;
            if key == wanted {
                self.current = key;
                match result {
                    Ok(data) => {
                        let texture = rs.device.create_texture(&wgpu::TextureDescriptor {
                            label: Some("Linear HDR sky"),
                            size: wgpu::Extent3d {
                                width: data.width,
                                height: data.height,
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: data.levels,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::Rgba16Float,
                            usage: wgpu::TextureUsages::TEXTURE_BINDING
                                | wgpu::TextureUsages::COPY_DST,
                            view_formats: &[],
                        });
                        let mut offset = 0;
                        let (mut w, mut h) = (data.width, data.height);
                        for level in 0..data.levels {
                            let size = (w * h * 8) as usize;
                            rs.queue.write_texture(
                                wgpu::TexelCopyTextureInfo {
                                    texture: &texture,
                                    mip_level: level,
                                    origin: Default::default(),
                                    aspect: wgpu::TextureAspect::All,
                                },
                                &data.bytes[offset..offset + size],
                                wgpu::TexelCopyBufferLayout {
                                    offset: 0,
                                    bytes_per_row: Some(w * 8),
                                    rows_per_image: Some(h),
                                },
                                wgpu::Extent3d {
                                    width: w,
                                    height: h,
                                    depth_or_array_layers: 1,
                                },
                            );
                            offset += size;
                            w = (w / 2).max(1);
                            h = (h / 2).max(1);
                        }
                        self.groups = pipelines
                            .map(|p| environment_group(&rs.device, p, &texture, &self.sampler));
                        self.texture = texture;
                        self.revision = self.revision.wrapping_add(1);
                        self.width = data.width;
                        self.sh = data.sh;
                        self.status = format!(
                            "{} · {} × {} HDR",
                            [
                                "Studio",
                                "Mud Road",
                                "Citrus Orchard",
                                "Procedural sky",
                                "Custom environment"
                            ][s.env_mode as usize],
                            data.width,
                            data.height
                        );
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if ![1, 2, 4].contains(&s.env_mode) {
            self.current = wanted;
            self.status = if s.env_mode == 0 {
                "Studio".into()
            } else {
                "Procedural sky".into()
            };
            return;
        }
        if self.current != wanted && self.pending.is_none() {
            let (tx, rx) = std::sync::mpsc::channel();
            self.pending = Some((wanted, rx));
            self.status = "Loading HDR environment…".into();
            let path = s.custom_environment.clone();
            std::thread::spawn(move || {
                let _ = tx.send(if wanted.0 == 4 {
                    load_custom_environment(&path, wanted.1)
                } else {
                    load_environment(wanted)
                });
            });
        }
    }
}
fn effective_sky_tier(override_tier: u32, quality: u32) -> u32 {
    if override_tier == 0 {
        quality.min(3)
    } else {
        override_tier.saturating_sub(1).min(3)
    }
}
fn load_environment(key: EnvironmentKey) -> Result<EnvironmentData, String> {
    let id = if key.0 == 1 { "mud" } else { "citrus" };
    let name = format!("{id}-{}.env.gz", key.1);
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let installed = executable
        .parent()
        .unwrap()
        .join("environments")
        .join(&name);
    let development = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../environments")
        .join(&name);
    let bytes = std::fs::read(installed)
        .or_else(|_| std::fs::read(development))
        .map_err(|_| "Extract the full release ZIP to load the HDR sky assets.".to_string())?;
    let mut decoded = Vec::new();
    flate2::read::GzDecoder::new(bytes.as_slice())
        .take(400 * 1024 * 1024)
        .read_to_end(&mut decoded)
        .map_err(|e| e.to_string())?;
    if decoded.len() < 16 || &decoded[..4] != b"MENV" {
        return Err("Invalid environment cache".into());
    }
    let read = |i| u32::from_le_bytes(decoded[i..i + 4].try_into().unwrap());
    let (width, height, levels) = (read(4), read(8), read(12));
    if ![1024, 2048, 4096, 8192].contains(&width)
        || height != width / 2
        || levels != width.ilog2() + 1
    {
        return Err("Invalid environment dimensions".into());
    }
    let (mut w, mut h, mut length) = (width, height, 16usize);
    for _ in 0..levels {
        length += (w * h * 8) as usize;
        w = (w / 2).max(1);
        h = (h / 2).max(1);
    }
    if decoded.len() != length {
        return Err("Incomplete HDR environment".into());
    }
    let catalog: Vec<EnvironmentInfo> =
        serde_json::from_str(include_str!("../../environments/catalog.json"))
            .map_err(|e| e.to_string())?;
    decoded.drain(..16);
    Ok(EnvironmentData {
        width,
        height,
        levels,
        bytes: decoded,
        sh: catalog[(key.0 - 1) as usize].sh,
    })
}

fn load_custom_environment(path: &str, tier: u32) -> Result<EnvironmentData, String> {
    let mut reader = image::ImageReader::open(path).map_err(|e| e.to_string())?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(512 * 1024 * 1024);
    reader.limits(limits);
    let width = 1024u32 << tier.min(3);
    let height = width / 2;
    let mut current = {
        let source = reader.decode().map_err(|e| e.to_string())?.into_rgb32f();
        if source.width() != source.height() * 2 {
            return Err("Use a 2:1 latitude-longitude HDR or EXR panorama.".into());
        }
        if source
            .pixels()
            .any(|pixel| pixel.0.iter().any(|v| !v.is_finite()))
        {
            return Err("The environment contains nonfinite radiance.".into());
        }
        if source.width() == width {
            source
        } else {
            resize_radiance(&source, width, height)
        }
    };
    let mut sh = [[0.0f32; 4]; 9];
    let mut total = 0.0;
    for y in 0..128 {
        for x in 0..256 {
            let theta = (y as f32 + 0.5) / 128.0 * std::f32::consts::PI;
            let phi = ((x as f32 + 0.5) / 256.0 - 0.5) * std::f32::consts::TAU;
            let n = Vec3::new(
                theta.sin() * phi.cos(),
                theta.cos(),
                theta.sin() * phi.sin(),
            );
            let b = [
                0.282095,
                0.488603 * n.y,
                0.488603 * n.z,
                0.488603 * n.x,
                1.092548 * n.x * n.y,
                1.092548 * n.y * n.z,
                0.315392 * (3.0 * n.z * n.z - 1.0),
                1.092548 * n.x * n.z,
                0.546274 * (n.x * n.x - n.y * n.y),
            ];
            let pixel = current.get_pixel(x * width / 256, y * height / 128);
            let weight = theta.sin();
            total += weight;
            for (i, coefficient) in sh.iter_mut().enumerate() {
                for (c, value) in coefficient.iter_mut().take(3).enumerate() {
                    *value += pixel[c].clamp(0.0, 65504.0) * b[i] * weight;
                }
            }
        }
    }
    for (i, coefficient) in sh.iter_mut().enumerate() {
        let scale = 4.0 * std::f32::consts::PI / total
            * if i == 0 {
                1.0
            } else if i < 4 {
                2.0 / 3.0
            } else {
                0.25
            };
        for value in coefficient.iter_mut().take(3) {
            *value *= scale;
        }
    }
    let mut bytes = Vec::with_capacity((width * height * 11) as usize);
    let mut levels = 0;
    loop {
        for pixel in current.pixels() {
            for value in pixel.0 {
                bytes.extend_from_slice(
                    &half::f16::from_f32(value.clamp(0.0, 65504.0)).to_le_bytes(),
                );
            }
            bytes.extend_from_slice(&0x3c00u16.to_le_bytes());
        }
        levels += 1;
        if current.width() == 1 && current.height() == 1 {
            break;
        }
        current = resize_radiance(
            &current,
            (current.width() / 2).max(1),
            (current.height() / 2).max(1),
        );
    }
    Ok(EnvironmentData {
        width,
        height,
        levels,
        bytes,
        sh,
    })
}

fn resize_radiance(source: &image::Rgb32FImage, width: u32, height: u32) -> image::Rgb32FImage {
    let sx = source.width() as f32 / width as f32;
    let sy = source.height() as f32 / height as f32;
    image::Rgb32FImage::from_fn(width, height, |x, y| {
        let left = x as f32 * sx;
        let right = (x + 1) as f32 * sx;
        let top = y as f32 * sy;
        let bottom = (y + 1) as f32 * sy;
        let mut radiance = [0.0; 3];
        for yy in top.floor() as u32..(bottom.ceil() as u32).min(source.height()) {
            for xx in left.floor() as u32..(right.ceil() as u32).min(source.width()) {
                let weight = (right.min(xx as f32 + 1.0) - left.max(xx as f32))
                    * (bottom.min(yy as f32 + 1.0) - top.max(yy as f32))
                    / (sx * sy);
                let pixel = source.get_pixel(xx, yy);
                for c in 0..3 {
                    radiance[c] += pixel[c] * weight;
                }
            }
        }
        image::Rgb(radiance)
    })
}
