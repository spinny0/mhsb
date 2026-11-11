use std::collections::BTreeMap;
const ROOT_CAPACITY: usize = 8192;
const MODEL_ID: &str = "makehuman-cc0-bust-v1";
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct SurfaceRoot {
    triangle: usize,
    barycentric: [f32; 3],
}
#[derive(serde::Deserialize)]
struct HumanMesh {
    id: String,
    vertices: Vec<Vec3>,
    normals: Vec<Vec3>,
    motion: Vec<f32>,
    triangles: Vec<[usize; 3]>,
    #[serde(rename = "scalpFaces")]
    scalp_faces: Vec<usize>,
    roots: Vec<SurfaceRoot>,
}
fn human_mesh() -> &'static HumanMesh {
    static MESH: std::sync::OnceLock<HumanMesh> = std::sync::OnceLock::new();
    MESH.get_or_init(|| {
        let mesh: HumanMesh =
            serde_json::from_str(include_str!("../../model/human.json")).expect("Human mesh");
        assert_eq!(mesh.id, MODEL_ID);
        mesh
    })
}
impl HumanMesh {

    fn attached_surface(&self, r: &SurfaceRoot) -> (Vec3, Vec3, f32) {
        let f = self.triangles[r.triangle];
        let p = (0..3).map(|i| self.vertices[f[i]] * r.barycentric[i]).sum();
        let n = (0..3)
            .map(|i| self.normals[f[i]] * r.barycentric[i])
            .sum::<Vec3>()
            .normalize_or_zero();
        let movement = (0..3).map(|i| self.motion[f[i]] * r.barycentric[i]).sum();
        (p, n, movement)
    }
    fn scalp(&self, id: usize) -> bool {
        self.scalp_faces
            .binary_search(&self.roots[id].triangle)
            .is_ok()
    }
    fn hit(&self, origin: Vec3, direction: Vec3, angle: f32) -> Option<SurfaceHit> {
        let mut nearest = f32::INFINITY;
        let mut hit = None;
        for (index, f) in self.triangles.iter().enumerate() {
            let world = f.map(|i| rotate_cpu(self.vertices[i], angle * self.motion[i]));
            let e1 = world[1] - world[0];
            let e2 = world[2] - world[0];
            let h = direction.cross(e2);
            let det = e1.dot(h);
            if det <= 0.000001 {
                continue;
            }
            let inv = 1.0 / det;
            let delta = origin - world[0];
            let u = delta.dot(h) * inv;
            if !(0.0..=1.0).contains(&u) {
                continue;
            }
            let q = delta.cross(e1);
            let v = direction.dot(q) * inv;
            if v < 0.0 || u + v > 1.0 {
                continue;
            }
            let t = e2.dot(q) * inv;
            if t <= 0.0 || t >= nearest {
                continue;
            }
            let b = [1.0 - u - v, u, v];
            let p = (0..3).map(|i| self.vertices[f[i]] * b[i]).sum();
            let n = (0..3)
                .map(|i| self.normals[f[i]] * b[i])
                .sum::<Vec3>()
                .normalize_or_zero();
            nearest = t;
            hit = Some(SurfaceHit {
                position: p,
                normal: n,
                world: origin + direction * t,
                scalp: self.scalp_faces.binary_search(&index).is_ok(),
            });
        }
        hit
    }
}
#[derive(Clone, Copy)]
struct SurfaceHit {
    position: Vec3,
    normal: Vec3,
    world: Vec3,
    scalp: bool,
}
fn rotate_cpu(p: Vec3, a: f32) -> Vec3 {
    Vec3::new(
        a.cos() * p.x + a.sin() * p.z,
        p.y,
        -a.sin() * p.x + a.cos() * p.z,
    )
}
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct RootPaint {
    mask: f32,
    density: f32,
    length_cm: f32,
    curl: f32,
    clump: f32,
    width: f32,
    flyaway: f32,
    color: i32,
    comb: Vec3,
}
impl Default for RootPaint {
    fn default() -> Self {
        Self {
            mask: 1.0,
            density: 1.0,
            length_cm: 0.0,
            curl: 1.0,
            clump: 1.0,
            width: 1.0,
            flyaway: -1.0,
            color: -1,
            comb: Vec3::ZERO,
        }
    }
}
impl RootPaint {
    fn validate(&mut self) {
        for v in [
            &mut self.mask,
            &mut self.density,
            &mut self.length_cm,
            &mut self.curl,
            &mut self.clump,
            &mut self.width,
            &mut self.flyaway,
        ] {
            if !v.is_finite() {
                *v = 0.0;
            }
        }
        self.mask = self.mask.clamp(0.0, 1.0);
        self.density = self.density.clamp(0.0, 1.0);
        self.length_cm = self.length_cm.clamp(0.0, 80.0);
        self.curl = self.curl.clamp(0.0, 2.5);
        self.clump = self.clump.clamp(0.0, 2.0);
        self.width = self.width.clamp(0.2, 2.5);
        self.flyaway = self.flyaway.clamp(-1.0, 0.3);
        self.color = self.color.clamp(-1, 13);
        if !self.comb.is_finite() {
            self.comb = Vec3::ZERO;
        }
        self.comb = self.comb.clamp_length_max(1.0);
    }
}
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct GroomLayer {
    name: String,
    enabled: bool,
    opacity: f32,
    base: bool,
    edits: BTreeMap<usize, RootPaint>,
}
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct GroomDocument {
    version: u32,
    model: String,
    layers: Vec<GroomLayer>,
    selected: usize,
    placements: BTreeMap<usize, SurfaceRoot>,
}
impl Default for GroomDocument {
    fn default() -> Self {
        Self {
            version: 2,
            model: MODEL_ID.into(),
            layers: vec![GroomLayer {
                name: "Base groom".into(),
                enabled: true,
                opacity: 1.0,
                base: true,
                edits: BTreeMap::new(),
            }],
            selected: 0,
            placements: BTreeMap::new(),
        }
    }
}
impl GroomDocument {
    fn validate(&mut self) {
        if ![1, 2].contains(&self.version) || self.model != MODEL_ID || self.layers.is_empty() {
            *self = Self::default();
            return;
        }
        self.version = 2;
        self.placements
            .retain(|id, root| *id < ROOT_CAPACITY && root.valid());
        self.layers.truncate(8);
        self.selected = self.selected.min(self.layers.len() - 1);
        for layer in &mut self.layers {
            layer.name = layer.name.chars().take(48).collect();
            if !layer.opacity.is_finite() {
                layer.opacity = 1.0;
            }
            layer.opacity = layer.opacity.clamp(0.0, 1.0);
            layer.edits.retain(|id, _| *id < ROOT_CAPACITY);
            for p in layer.edits.values_mut() {
                p.validate();
            }
        }
    }
    fn field(&self, id: usize) -> RootPaint {
        let mut value = RootPaint {
            density: 0.0,
            ..RootPaint::default()
        };
        for layer in &self.layers {
            if !layer.enabled {
                continue;
            }
            if layer.base {
                value = RootPaint {
                    density: if !self.placements.contains_key(&id) && human_mesh().scalp(id) {
                        1.0
                    } else {
                        0.0
                    },
                    ..RootPaint::default()
                };
            }
            if let Some(p) = layer.edits.get(&id) {
                let w = p.mask * if layer.base { 1.0 } else { layer.opacity };
                value.density = lerp(value.density, p.density, w);
                value.length_cm = lerp(value.length_cm, p.length_cm, w);
                value.curl = lerp(value.curl, p.curl, w);
                value.clump = lerp(value.clump, p.clump, w);
                value.width = lerp(value.width, p.width, w);
                value.flyaway = lerp(value.flyaway, p.flyaway, w);
                value.comb = value.comb.lerp(p.comb, w);
                if w >= 0.5 {
                    value.color = p.color;
                }
            }
            if layer.base {
                value.density *= layer.opacity;
            }
        }
        value
    }
}
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
fn noise(n: f32) -> f32 {
    let x = (n * 127.1 + 31.7).sin() * 43758.547;
    x - x.floor()
}
fn frame_cpu(t: Vec3, h: Vec3) -> Vec3 {
    let p = h - t * t.dot(h);
    if p.length_squared() > 0.000001 {
        p.normalize()
    } else {
        t.cross(if t.y.abs() > 0.9 { Vec3::X } else { Vec3::Y })
            .normalize_or_zero()
    }
}
fn preset_length(style: u32) -> f32 {
    match style {
        0 => 55.0,
        1 => 65.0,
        2 => 42.0,
        3 => 28.0,
        4 => 36.0,
        _ => 26.0,
    }
}
fn nominal_length(doc: &GroomDocument, id: usize, style: u32, fringe: f32) -> f32 {
    let (p, n, _) = doc.surface(id);
    let r = p + n * 0.002;
    if style == 5 && r.z > 0.12 && r.y > 1.5 && r.x.abs() < 0.58 {
        ((r.y - 1.30) / 0.072).clamp(2.0, 11.0) * fringe
    } else {
        preset_length(style)
    }
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RootGpu {
    position: [f32; 4],
    normal: [f32; 4],
    parameters: [f32; 4],
    variation: [f32; 4],
    triangle: [[f32; 4]; 3],
    barycentric: [f32; 4],
    rest: [[f32; 4]; 25],
}
fn build_root(id: usize, s: &Settings, revision: f32) -> RootGpu {
    let mesh = human_mesh();
    let (surface, n, movement) = s.groom.surface(id);
    let r = surface + n * 0.002;
    let field = s.groom.field(id);
    let default_length = nominal_length(&s.groom, id, s.style, s.fringe);
    let target = if field.length_cm > 0.0 {
        field.length_cm
    } else {
        default_length
    } * s.length
        * 0.072;
    let mut points = [Vec3::ZERO; 25];
    let radial = Vec3::new(r.x, 0.0, r.z).normalize_or_zero();
    let front = smooth(-0.1, 0.65, radial.z);
    let side = if r.x >= 0.0 { 1.0 } else { -1.0 };
    let seed = ((r.z.atan2(r.x) + std::f32::consts::PI) / std::f32::consts::TAU * 24.0).floor()
        + ((2.1 - r.y) * 1.5).clamp(0.0, 2.0).floor() * 24.0;
    let phase = noise(seed + 41.0) * std::f32::consts::TAU;
    let variation = 0.9 + 0.16 * noise(seed) + 0.04 * noise(id as f32 + 9.0);
    for (j, p) in points.iter_mut().enumerate() {
        let t = (j as f32 / 24.0).powf(1.3);
        let mut dest = radial * 0.81;
        dest = dest.lerp(Vec3::new(side * 0.89, 0.0, -0.06), front);
        let mut bottom = -0.72;
        if s.style > 0 {
            dest *= 1.08;
            bottom = -1.0;
        }
        if s.style > 1 {
            dest *= 1.06;
            bottom = -0.5;
        }
        if s.style > 4 {
            dest *= 1.10;
            bottom = 0.35 + 0.16 * noise(seed);
        }
        let sweep = (dest - Vec3::new(r.x, 0.0, r.z)) * (1.0 - (-t * 9.0).exp());
        let mut center = r + Vec3::new(sweep.x, -(r.y - bottom) * t * variation, sweep.z);
        let (mut radius, mut turns) = match s.style {
            0 => (0.0, 1.0),
            1 => (0.12, 2.15),
            2 => (0.105, 3.2),
            3 | 4 => (0.055, 3.6),
            _ => (0.10, 2.15),
        };
        if s.style == 3 || s.style == 4 {
            let size = if s.style == 4 { 0.92 } else { 0.63 };
            let frontal = r.z > 0.0 && r.x.abs() < 0.62;
            let reach = size * s.hair_volume * if frontal { 0.25 } else { 1.0 };
            let growth = (n + if frontal { Vec3::Y * 0.4 } else { Vec3::ZERO }).normalize_or_zero();
            center = r + growth * reach * t.powf(0.65) * variation;
            center.y -= if frontal { 0.1 } else { 0.25 } * t * t;
            if frontal {
                radius = 0.01;
                turns = 2.0;
            }
        }
        if s.style == 5 && r.z > 0.12 && r.y > 1.50 && r.x.abs() < 0.58 {
            center = r + Vec3::new(
                (-r.x * 0.25 + phase.sin() * 0.04) * t,
                -0.5 * t,
                0.13 * (t * std::f32::consts::FRAC_PI_2).sin(),
            );
            radius = 0.02;
            turns = 1.2;
        }
        if !s.groom.scalp(id) {
            center = r + n * (0.20 * t * (1.0 - t)) + Vec3::NEG_Y * t;
        }
        center += field.comb * (t * t * 0.8);
        radius *= s.curl_radius * field.curl;
        turns *= s.curl_tightness;
        let axis = (center - r + Vec3::new(0.00001, -0.0001, 0.0)).normalize_or_zero();
        let x = frame_cpu(axis, Vec3::X);
        let y = axis.cross(x);
        let a = phase + t * turns * std::f32::consts::TAU;
        *p = center
            + (x * (a.sin() - phase.sin()) + y * (a.cos() - phase.cos()))
                * radius
                * smooth(0.0, 0.20, t);
    }
    let arc = points
        .windows(2)
        .map(|p| p[0].distance(p[1]))
        .sum::<f32>()
        .max(0.001);
    for p in &mut points {
        *p = r + (*p - r) * target / arc;
    }
    points[0] = r;
    points[1] = r + n * 0.012_f32.min(target * 0.1);
    points[2] = r + n * 0.024_f32.min(target * 0.2);
    let attachment = s.groom.attachment(id);
    let f = mesh.triangles[attachment.triangle];
    RootGpu {
        position: [r.x, r.y, r.z, movement],
        normal: [n.x, n.y, n.z, 0.0],
        parameters: [field.density, field.curl, revision, target],
        variation: [field.width, field.flyaway, field.color as f32, field.clump],
        triangle: f.map(|i| {
            [
                mesh.vertices[i].x,
                mesh.vertices[i].y,
                mesh.vertices[i].z,
                mesh.motion[i],
            ]
        }),
        barycentric: [
            attachment.barycentric[0],
            attachment.barycentric[1],
            attachment.barycentric[2],
            attachment.triangle as f32,
        ],
        rest: points.map(|p| [p.x, p.y, p.z, 0.0]),
    }
}
#[derive(Default)]
struct RootCache {
    records: Vec<RootGpu>,
    document: Option<GroomDocument>,
    shape: Vec<f32>,
    revision: f32,
    fields: Vec<RootPaint>,
}
impl RootCache {
    fn update(&mut self, s: &Settings, queue: &wgpu::Queue, buffer: &wgpu::Buffer) -> bool {
        let shape = vec![
            s.style as f32,
            s.length,
            s.curl_radius,
            s.curl_tightness,
            s.hair_volume,
            s.fringe,
        ];
        if self.document.as_ref() == Some(&s.groom) && self.shape == shape {
            return false;
        }
        let first = self.records.is_empty();
        self.revision += 1.0;
        if first {
            self.records
                .resize(ROOT_CAPACITY, <RootGpu as bytemuck::Zeroable>::zeroed());
            self.fields.resize(ROOT_CAPACITY, RootPaint::default());
        }
        let mut changed = false;
        for id in 0..ROOT_CAPACITY {
            let field = s.groom.field(id);
            if !first
                && self.shape == shape
                && self.fields[id] == field
                && self
                    .document
                    .as_ref()
                    .is_some_and(|doc| doc.attachment(id) == s.groom.attachment(id))
            {
                continue;
            }
            self.fields[id] = field;
            let old = self.records[id];
            let mut next = build_root(id, s, old.parameters[2]);
            if next.triangle != old.triangle
                || next.barycentric != old.barycentric
                || next.rest != old.rest
                || (old.parameters[0] <= 0.001 && next.parameters[0] > 0.001)
            {
                next.parameters[2] = self.revision;
            }
            if first || bytemuck::bytes_of(&next) != bytemuck::bytes_of(&old) {
                self.records[id] = next;
                if !first {
                    queue.write_buffer(
                        buffer,
                        (id * std::mem::size_of::<RootGpu>()) as u64,
                        bytemuck::bytes_of(&next),
                    );
                }
                changed = true;
            }
        }
        if first {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.records));
        }
        self.document = Some(s.groom.clone());
        self.shape = shape;
        changed
    }
}
#[derive(Clone, Copy, PartialEq)]
enum PaintTool {
    Off,
    Density,
    Erase,
    Length,
    Comb,
    Curl,
    Clump,
    Width,
    Flyaway,
    Color,
    Smooth,
}
struct PaintState {
    tool: PaintTool,
    radius: f32,
    strength: f32,
    softness: f32,
    length_cm: f32,
    curl: f32,
    clump: f32,
    width: f32,
    flyaway: f32,
    color: i32,
    symmetry: bool,
    scalp_only: bool,
    show_roots: bool,
    before: Option<GroomDocument>,
    last_hit: Option<SurfaceHit>,
    undo: Vec<GroomDocument>,
    redo: Vec<GroomDocument>,
    path: String,
    status: String,
}
impl Default for PaintState {
    fn default() -> Self {
        Self {
            tool: PaintTool::Off,
            radius: 0.20,
            strength: 1.0,
            softness: 0.7,
            length_cm: 5.0,
            curl: 1.0,
            clump: 1.0,
            width: 1.0,
            flyaway: 0.025,
            color: 0,
            symmetry: false,
            scalp_only: false,
            show_roots: false,
            before: None,
            last_hit: None,
            undo: vec![],
            redo: vec![],
            status: String::new(),
            path: std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.join("grooms/My groom.mhsbgroom")))
                .unwrap_or_else(|| std::path::PathBuf::from("My groom.mhsbgroom"))
                .to_string_lossy()
                .into_owned(),
        }
    }
}
impl PaintState {
    fn begin(&mut self, doc: &GroomDocument) {
        if self.before.is_none() {
            self.before = Some(doc.clone());
        }
    }
    fn end(&mut self, doc: &GroomDocument) {
        if let Some(before) = self.before.take()
            && before != *doc
        {
            self.undo.push(before);
            if self.undo.len() > 16 {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
        self.last_hit = None;
    }
    fn undo(&mut self, doc: &mut GroomDocument) {
        self.end(doc);
        if let Some(before) = self.undo.pop() {
            self.redo.push(doc.clone());
            *doc = before;
        }
    }
    fn redo(&mut self, doc: &mut GroomDocument) {
        if let Some(after) = self.redo.pop() {
            self.undo.push(doc.clone());
            *doc = after;
        }
    }
    fn dab(
        &mut self,
        doc: &mut GroomDocument,
        hit: SurfaceHit,
        flow: Vec3,
        shape: (u32, f32, f32),
        guide_count: usize,
    ) -> usize {
        let (style, global_length, fringe) = shape;
        if self.scalp_only && !hit.scalp {
            return 0;
        }
        if self.tool == PaintTool::Off {
            return 0;
        }
        let layer = &mut doc.layers[doc.selected];
        layer.enabled = true;
        if layer.opacity < 0.001 {
            layer.opacity = 1.0;
        }
        let added = if self.tool == PaintTool::Density {
            doc.seed_patch(
                hit,
                self.radius,
                self.symmetry,
                self.scalp_only,
                guide_count,
            )
        } else {
            0
        };
        let mut ids = Vec::new();
        for id in 0..guide_count.min(ROOT_CAPACITY) {
            if self.scalp_only && !doc.scalp(id) {
                continue;
            }
            let (p, n, _) = doc.surface(id);
            let mut d = p.distance(hit.position);
            let mut compatible = n.dot(hit.normal) > 0.1;
            if self.symmetry {
                let center = Vec3::new(-hit.position.x, hit.position.y, hit.position.z);
                let mirrored = n.dot(Vec3::new(-hit.normal.x, hit.normal.y, hit.normal.z)) > 0.1;
                if mirrored && p.distance(center) < d {
                    d = p.distance(center);
                    compatible = true;
                }
            }
            if compatible && d < self.radius {
                ids.push((
                    id,
                    1.0 - smooth(self.radius * (1.0 - self.softness), self.radius, d),
                ));
            }
        }
        let fields: Vec<_> = ids.iter().map(|&(id, _)| doc.field(id)).collect();
        let mut average = RootPaint {
            density: 0.0,
            length_cm: 0.0,
            curl: 0.0,
            clump: 0.0,
            width: 0.0,
            flyaway: 0.0,
            comb: Vec3::ZERO,
            ..RootPaint::default()
        };
        for (&(id, _), f) in ids.iter().zip(&fields) {
            average.density += f.density;
            average.length_cm += if f.length_cm > 0.0 {
                f.length_cm
            } else {
                nominal_length(doc, id, style, fringe)
            };
            average.curl += f.curl;
            average.clump += f.clump;
            average.width += f.width;
            average.flyaway += f.flyaway;
            average.comb += f.comb;
        }
        let count = fields.len().max(1) as f32;
        average.density /= count;
        average.length_cm /= count;
        average.curl /= count;
        average.clump /= count;
        average.width /= count;
        average.flyaway /= count;
        average.comb /= count;
        let lengths: Vec<_> = ids
            .iter()
            .map(|&(id, _)| nominal_length(doc, id, style, fringe))
            .collect();
        let positions: Vec<_> = ids.iter().map(|&(id, _)| doc.surface(id).0).collect();
        let layer = &mut doc.layers[doc.selected];
        for (index, ((id, w), mut f)) in ids.iter().copied().zip(fields).enumerate() {
            let w = w * self.strength;
            f.mask = 1.0;
            match self.tool {
                PaintTool::Density => {
                    if f.density < 0.001 {
                        f.length_cm = self.length_cm / global_length;
                        f.curl = self.curl;
                    }
                    f.density = lerp(f.density, 1.0, w);
                }
                PaintTool::Erase => f.density = lerp(f.density, 0.0, w),
                PaintTool::Length => {
                    f.length_cm = lerp(
                        if f.length_cm > 0.0 {
                            f.length_cm
                        } else {
                            lengths[index]
                        },
                        self.length_cm / global_length,
                        w,
                    )
                }
                PaintTool::Comb => {
                    let mut direction = flow - hit.normal * flow.dot(hit.normal);
                    if self.symmetry && hit.position.x.signum() != positions[index].x.signum() {
                        direction.x = -direction.x;
                    }
                    if direction.length_squared() > 0.000001 {
                        f.comb = f.comb.lerp(direction.normalize(), w);
                    }
                }
                PaintTool::Curl => f.curl = lerp(f.curl, self.curl, w),
                PaintTool::Clump => f.clump = lerp(f.clump, self.clump, w),
                PaintTool::Width => f.width = lerp(f.width, self.width, w),
                PaintTool::Flyaway => f.flyaway = lerp(f.flyaway, self.flyaway, w),
                PaintTool::Color => f.color = self.color,
                PaintTool::Smooth => {
                    f.density = lerp(f.density, average.density, w);
                    f.length_cm = lerp(
                        if f.length_cm > 0.0 {
                            f.length_cm
                        } else {
                            lengths[index]
                        },
                        average.length_cm,
                        w,
                    );
                    f.curl = lerp(f.curl, average.curl, w);
                    f.comb = f.comb.lerp(average.comb, w);
                }
                PaintTool::Off => continue,
            }
            f.validate();
            layer.edits.insert(id, f);
        }
        self.status = if self.tool == PaintTool::Density && ids.is_empty() {
            "No spare guides. Erase existing hair or increase Physics guides.".into()
        } else {
            format!("{} guides edited · {added} new roots", ids.len())
        };
        ids.len()
    }
}
