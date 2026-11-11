#[derive(serde::Deserialize)]
struct HairstylePreset {
    name: String,
    style: u32,
    length: f32,
    memory: f32,
    roughness: f32,
    color: u32,
    clumping: f32,
    flyaways: f32,
    distance: f32,
    fibers: u32,
}
fn hairstyles() -> &'static [HairstylePreset] {
    static PRESETS: std::sync::OnceLock<Vec<HairstylePreset>> = std::sync::OnceLock::new();
    PRESETS.get_or_init(|| {
        serde_json::from_str(include_str!("../../hairstyles.json")).expect("Built-in hairstyles")
    })
}
impl Settings {
    fn apply_hairstyle(&mut self, index: usize) {
        let preset = &hairstyles()[index.min(hairstyles().len() - 1)];
        self.curl_radius = 1.0;
        self.curl_tightness = 1.0;
        self.hair_volume = 1.0;
        self.fringe = 1.0;
        self.camera_pan = Vec3::ZERO;
        self.style = preset.style;
        self.length = preset.length;
        self.memory = preset.memory;
        self.roughness = preset.roughness;
        self.color = preset.color;
        self.clumping = preset.clumping;
        self.flyaways = preset.flyaways;
        self.distance = preset.distance;
        self.fibers = preset.fibers;
    }
}
#[cfg(test)]
mod hairstyle_tests {
    use super::*;
    #[test]
    fn builtins_and_saved_grooms_keep_their_shape_settings() {
        assert_eq!(hairstyles().len(), 6);
        for (index, preset) in hairstyles().iter().enumerate() {
            assert_eq!(preset.style, index as u32);
            let mut settings = Settings::default();
            settings.apply_hairstyle(index);
            settings.validate();
            let loaded: Settings =
                serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
            assert_eq!(loaded.style, preset.style);
            assert_eq!(loaded.memory, preset.memory);
            assert_eq!(loaded.fibers, preset.fibers);
            assert_eq!(loaded.frame_limit, 60);
        }
        for style in 0..3 {
            let mut old = Settings {
                style,
                ..Settings::default()
            };
            old.validate();
            assert_eq!(old.style, style);
        }
    }
}
