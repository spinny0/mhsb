fn density_lod_eligible(s: &Settings, height: u32) -> bool {
    height > 0
        && s.frame_limit > 0
        && s.god_opts
        && s.god_density_lod
        && !s.paused
        && !s.grab_active
        && s.distance >= 3.2
        && s.diameter * 0.001 * height as f32 / (s.distance - 2.0).max(0.5) < 0.25
}
fn density_lod_count(s: &Settings, height: u32, fraction: f32) -> u32 {
    if !density_lod_eligible(s, height) {
        return s.fibers;
    }
    let count =
        ((s.fibers as f32 * fraction.clamp(0.0625, 1.0)) as u32 / s.guides).max(4) * s.guides;
    count.min(s.fibers)
}
#[cfg(test)]
mod density_lod_tests {
    use super::*;
    #[test]
    fn lod_is_explicit_bounded_and_leaves_physics_settings_unchanged() {
        let mut s = Settings {
            fibers: 200000,
            guides: 4096,
            distance: 6.0,
            god_opts: true,
            god_density_lod: true,
            ..Settings::default()
        };
        let count = density_lod_count(&s, 1080, 0.25);
        assert!(count >= s.fibers / 5 && count < s.fibers);
        assert_eq!(s.guides, 4096);
        assert_eq!(s.iterations, 24);
        s.god_density_lod = false;
        assert_eq!(density_lod_count(&s, 1080, 0.25), s.fibers);
        s.god_density_lod = true;
        s.distance = 2.0;
        assert_eq!(density_lod_count(&s, 1080, 0.25), s.fibers);
        s.distance = 6.0;
        s.paused = true;
        assert_eq!(density_lod_count(&s, 1080, 0.25), s.fibers);
    }
}

fn curve_segments(s: &Settings, height: u32) -> u32 {
    if density_lod_eligible(s, height) {
        match s.style {
            0 => 16,
            1 => 32,
            3 | 4 => 64,
            _ => 48,
        }
    } else {
        match s.style {
            0 => 48,
            1 => 64,
            3 | 4 => 96,
            _ => 80,
        }
    }
}
