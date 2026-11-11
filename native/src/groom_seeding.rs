impl SurfaceRoot {
    fn valid(&self) -> bool {
        self.triangle < human_mesh().triangles.len()
            && self
                .barycentric
                .iter()
                .all(|v| v.is_finite() && *v >= 0.0 && *v <= 1.0)
            && (self.barycentric.iter().sum::<f32>() - 1.0).abs() < 0.001
    }
}
impl GroomDocument {
    fn attachment(&self, id: usize) -> SurfaceRoot {
        *self.placements.get(&id).unwrap_or(&human_mesh().roots[id])
    }
    fn surface(&self, id: usize) -> (Vec3, Vec3, f32) {
        human_mesh().attached_surface(&self.attachment(id))
    }
    fn scalp(&self, id: usize) -> bool {
        human_mesh()
            .scalp_faces
            .binary_search(&self.attachment(id).triangle)
            .is_ok()
    }
    fn can_relocate(&self, id: usize) -> bool {
        if !self.placements.contains_key(&id)
            && human_mesh().scalp(id)
            && self.layers.iter().any(|l| l.base)
        {
            return false;
        }
        self.layers
            .iter()
            .all(|l| l.edits.get(&id).is_none_or(|p| p.density <= 0.001))
    }
    fn seed_patch(
        &mut self,
        hit: SurfaceHit,
        radius: f32,
        symmetry: bool,
        scalp_only: bool,
        guide_count: usize,
    ) -> usize {
        let capacity = guide_count.min(ROOT_CAPACITY);
        let mut total = 0;
        let mut reserved = std::collections::BTreeSet::new();
        for mirror in 0..if symmetry { 2 } else { 1 } {
            let center = if mirror == 0 {
                hit.position
            } else {
                Vec3::new(-hit.position.x, hit.position.y, hit.position.z)
            };
            let normal = if mirror == 0 {
                hit.normal
            } else {
                Vec3::new(-hit.normal.x, hit.normal.y, hit.normal.z)
            };
            let mut neighbors = Vec::new();
            let mut used = vec![false; capacity];
            for (id, occupied) in used.iter_mut().enumerate() {
                let (p, n, _) = self.surface(id);
                if p.distance(center) < radius && n.dot(normal) > 0.1 {
                    neighbors.push(p);
                    *occupied = true;
                }
            }
            let wanted = ((std::f32::consts::PI * radius * radius / (0.055 * 0.055)).ceil()
                as usize)
                .clamp(4, 32);
            if neighbors.len() >= wanted {
                continue;
            }
            let mut free = (0..capacity)
                .filter(|&id| !used[id] && !reserved.contains(&id) && self.can_relocate(id))
                .collect::<Vec<_>>()
                .into_iter();
            let mesh = human_mesh();
            let mut patch = Vec::new();
            let mut area = 0.0;
            let mut closest = None;
            let mut best = radius * radius;
            for (id, f) in mesh.triangles.iter().enumerate() {
                if scalp_only && mesh.scalp_faces.binary_search(&id).is_err() {
                    continue;
                }
                let [a, b, c] = f.map(|i| mesh.vertices[i]);
                let cross = (b - a).cross(c - a);
                let size = cross.length();
                if size < 0.000001 || cross.dot(normal) / size < 0.1 {
                    continue;
                }
                let bounds = a.min(b).min(c);
                let upper = a.max(b).max(c);
                if center.distance_squared(center.clamp(bounds, upper)) > radius * radius {
                    continue;
                }
                area += size * 0.5;
                patch.push((area, id));
                let bary = closest_barycentric(center, a, b, c);
                let p = a * bary[0] + b * bary[1] + c * bary[2];
                let d = p.distance_squared(center);
                if d < best {
                    best = d;
                    closest = Some(SurfaceRoot {
                        triangle: id,
                        barycentric: bary,
                    });
                }
            }
            if patch.is_empty() {
                continue;
            }
            let mut random = (self.placements.len() as u32 + 1).wrapping_mul(2654435761)
                ^ ((center.x * 1000.0).floor() as i32 as u32)
                ^ ((center.y * 1000.0).floor() as i32 as u32).rotate_left(16);
            let mut rnd = || {
                random ^= random << 13;
                random ^= random >> 17;
                random ^= random << 5;
                (random as f64 / 4294967296.0) as f32
            };
            let spacing = 0.045_f32.min(radius * 0.32);
            for attempt in 0..wanted * 40 {
                if neighbors.len() >= wanted {
                    break;
                }
                let sample = if attempt == 0 {
                    closest.unwrap()
                } else {
                    let choice = rnd() * area;
                    let index = patch
                        .partition_point(|&(sum, _)| sum < choice)
                        .min(patch.len() - 1);
                    let triangle = patch[index].1;
                    let a = rnd().sqrt();
                    let b = rnd();
                    SurfaceRoot {
                        triangle,
                        barycentric: [1.0 - a, a * (1.0 - b), a * b],
                    }
                };
                let (p, n, _) = mesh.attached_surface(&sample);
                if p.distance(center) >= radius * 0.94
                    || n.dot(normal) < 0.1
                    || neighbors.iter().any(|q| p.distance(*q) < spacing)
                {
                    continue;
                }
                let Some(id) = free.next() else {
                    break;
                };
                self.placements.insert(id, sample);
                reserved.insert(id);
                neighbors.push(p);
                total += 1;
            }
        }
        total
    }
}
fn closest_barycentric(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> [f32; 3] {
    let mut bary = closest_barycentric_raw(p, a, b, c);
    for v in &mut bary {
        *v = v.clamp(0.0, 1.0);
    }
    let sum = bary.iter().sum::<f32>().max(0.000001);
    bary.map(|v| v / sum)
}
fn closest_barycentric_raw(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> [f32; 3] {
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return [1.0, 0.0, 0.0];
    }
    let bp = p - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return [0.0, 1.0, 0.0];
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [1.0 - v, v, 0.0];
    }
    let cp = p - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return [0.0, 0.0, 1.0];
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return [1.0 - w, 0.0, w];
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return [0.0, 1.0 - w, w];
    }
    let inv = 1.0 / (va + vb + vc);
    let v = vb * inv;
    let w = vc * inv;
    [1.0 - v - w, v, w]
}
