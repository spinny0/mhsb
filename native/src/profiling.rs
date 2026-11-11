struct Profiler {
    enabled: bool,
    tag: u64,
    pending_tag: u64,
    samples: std::collections::VecDeque<[f64; 5]>,
    active: bool,
    query: Option<wgpu::QuerySet>,
    resolve_buffer: wgpu::Buffer,
    readback: wgpu::Buffer,
    pending: Option<std::sync::mpsc::Receiver<Result<[f64; 5], String>>>,
    milliseconds: Option<[f64; 5]>,
}
impl Profiler {
    fn new(device: &wgpu::Device) -> Self {
        let query = device
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY)
            .then(|| {
                device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("GPU pass timings"),
                    ty: wgpu::QueryType::Timestamp,
                    count: 10,
                })
            });
        Self {
            enabled: false,
            tag: 0,
            pending_tag: 0,
            samples: std::collections::VecDeque::new(),
            active: false,
            query,
            pending: None,
            milliseconds: None,
            resolve_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Timing resolve"),
                size: 80,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }),
            readback: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Timing readback"),
                size: 80,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
        }
    }
    fn poll(&mut self, device: &wgpu::Device) {
        let _ = device.poll(wgpu::PollType::Poll);
        if let Some(rx) = &self.pending
            && let Ok(result) = rx.try_recv()
        {
            self.pending = None;
            if let Ok(values) = result
                && self.pending_tag == self.tag
            {
                self.samples.push_back(values);
                if self.samples.len() > 32 {
                    self.samples.pop_front();
                }
                self.milliseconds = Some(std::array::from_fn(|i| {
                    self.samples.iter().map(|v| v[i]).sum::<f64>() / self.samples.len() as f64
                }));
            }
        }
    }
    fn set_tag(&mut self, tag: u64) {
        if self.tag != tag {
            self.tag = tag;
            self.samples.clear();
            self.milliseconds = None;
        }
    }
    fn begin(&mut self) {
        self.active = self.enabled && self.query.is_some() && self.pending.is_none();
    }
    fn compute(&self, pass: u32) -> Option<wgpu::ComputePassTimestampWrites<'_>> {
        self.active.then(|| wgpu::ComputePassTimestampWrites {
            query_set: self.query.as_ref().unwrap(),
            beginning_of_pass_write_index: Some(pass * 2),
            end_of_pass_write_index: Some(pass * 2 + 1),
        })
    }
    fn render(&self, pass: u32) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        self.active.then(|| wgpu::RenderPassTimestampWrites {
            query_set: self.query.as_ref().unwrap(),
            beginning_of_pass_write_index: Some(pass * 2),
            end_of_pass_write_index: Some(pass * 2 + 1),
        })
    }
    fn resolve(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.active {
            encoder.resolve_query_set(self.query.as_ref().unwrap(), 0..10, &self.resolve_buffer, 0);
            encoder.copy_buffer_to_buffer(&self.resolve_buffer, 0, &self.readback, 0, 80);
        }
    }
    fn map(&mut self, period: f32) {
        if !self.active {
            return;
        }
        self.pending_tag = self.tag;
        let buffer = self.readback.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.pending = Some(rx);
        self.readback
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let result = result.map_err(|e| e.to_string()).map(|()| {
                    let mapped = buffer.slice(..).get_mapped_range();
                    let ticks: Vec<u64> = mapped
                        .as_chunks::<8>()
                        .0
                        .iter()
                        .map(|b| u64::from_le_bytes(*b))
                        .collect();
                    let values = std::array::from_fn(|i| {
                        ticks[i * 2 + 1].saturating_sub(ticks[i * 2]) as f64 * period as f64
                            / 1_000_000.0
                    });
                    drop(mapped);
                    buffer.unmap();
                    values
                });
                let _ = tx.send(result);
            });
    }
    fn summary(&self) -> String {
        if self.query.is_none() {
            return "GPU timestamps unavailable on this adapter.".into();
        }
        match self.milliseconds {
            Some(t) => format!(
                "GPU average (up to 32 samples) · {:.2} ms total\nPhysics {:.2} · Curves {:.2}\nShadows {:.2} · Hair + scene {:.2} · Output {:.2} ms",
                t.iter().sum::<f64>(),
                t[0],
                t[1],
                t[2],
                t[3],
                t[4]
            ),
            None => "Enable GPU timing to inspect pass costs.".into(),
        }
    }
}
