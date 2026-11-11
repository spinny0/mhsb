impl Gpu {
    fn export_png(
        &mut self,
        s: &Settings,
        path: String,
        width: u32,
        supersample: bool,
    ) -> (std::sync::mpsc::Receiver<Result<String, String>>, Mat4) {
        let original = self.size;
        let requested_width = if width == 0 { original[0] } else { width }.min(4096);
        let requested_height = ((requested_width as f32 * original[1] as f32 / original[0] as f32)
            .round() as u32)
            .max(1);
        let fit = (4096.0 / requested_width.max(requested_height) as f32).min(1.0);
        let output_width = (requested_width as f32 * fit).round() as u32;
        let output_height = (requested_height as f32 * fit).round() as u32;
        let factor: f32 = if supersample { 2.0 } else { 1.0 };
        let scale = factor.min(4096.0 / output_width.max(output_height) as f32);
        let rendered = [
            (output_width as f32 * scale).round() as u32,
            (output_height as f32 * scale).round() as u32,
        ];
        self.resize(rendered);
        let mut capture = s.clone();
        capture.drawn = capture.fibers;
        capture.god_density_lod = false;
        self.render(&capture, 0, false);
        let stride = (rendered[0] * 4).div_ceil(256) * 256;
        let buffer = self.rs.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PNG readback"),
            size: (stride * rendered[1]) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self.rs.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: self.target.as_ref().unwrap(),
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride),
                    rows_per_image: Some(rendered[1]),
                },
            },
            wgpu::Extent3d {
                width: rendered[0],
                height: rendered[1],
                depth_or_array_layers: 1,
            },
        );
        self.rs.queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        let device = self.rs.device.clone();
        std::thread::spawn(move || {
            let result = (|| -> Result<String, String> {
                let (mapped_tx, mapped_rx) = std::sync::mpsc::channel();
                buffer
                    .slice(..)
                    .map_async(wgpu::MapMode::Read, move |result| {
                        let _ = mapped_tx.send(result);
                    });
                device
                    .poll(wgpu::PollType::Wait {
                        submission_index: None,
                        timeout: Some(std::time::Duration::from_secs(30)),
                    })
                    .map_err(|e| e.to_string())?;
                mapped_rx
                    .recv_timeout(std::time::Duration::from_secs(1))
                    .map_err(|e| e.to_string())?
                    .map_err(|e| e.to_string())?;
                let mapped = buffer.slice(..).get_mapped_range();
                let mut pixels = Vec::with_capacity((rendered[0] * rendered[1] * 4) as usize);
                for row in mapped.chunks(stride as usize) {
                    pixels.extend_from_slice(&row[..rendered[0] as usize * 4]);
                }
                drop(mapped);
                buffer.unmap();
                let image = image::RgbaImage::from_raw(rendered[0], rendered[1], pixels)
                    .ok_or("Invalid capture size")?;

                let image = if rendered == [output_width, output_height] {
                    image
                } else {
                    let mut linear = image::Rgba32FImage::new(rendered[0], rendered[1]);
                    for (source, destination) in image.pixels().zip(linear.pixels_mut()) {
                        for c in 0..3 {
                            let v = source[c] as f32 / 255.0;
                            destination[c] = if v <= 0.04045 {
                                v / 12.92
                            } else {
                                ((v + 0.055) / 1.055).powf(2.4)
                            };
                        }
                        destination[3] = 1.0;
                    }
                    let filtered = image::imageops::resize(
                        &linear,
                        output_width,
                        output_height,
                        image::imageops::FilterType::Lanczos3,
                    );
                    image::RgbaImage::from_fn(output_width, output_height, |x, y| {
                        let p = filtered.get_pixel(x, y);
                        let mut rgba = [0u8; 4];
                        for c in 0..3 {
                            let v = p[c].clamp(0.0, 1.0);
                            rgba[c] = ((if v <= 0.0031308 {
                                v * 12.92
                            } else {
                                1.055 * v.powf(1.0 / 2.4) - 0.055
                            }) * 255.0)
                                .round() as u8;
                        }
                        rgba[3] = 255;
                        image::Rgba(rgba)
                    })
                };
                let destination = std::path::Path::new(&path);
                if let Some(parent) = destination.parent()
                    && !parent.as_os_str().is_empty()
                {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let temporary =
                    destination.with_extension(format!("{}.png.tmp", std::process::id()));
                image
                    .save_with_format(&temporary, image::ImageFormat::Png)
                    .map_err(|e| e.to_string())?;
                std::fs::rename(&temporary, destination).map_err(|e| e.to_string())?;
                Ok(path)
            })();
            let _ = tx.send(result);
        });
        self.resize(original);
        let matrix = self.render(s, 0, false);
        (rx, matrix)
    }
}
