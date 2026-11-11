#[derive(Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum WindMode {
    Continuous,
    Gust,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct WindSource {
    id: u32,
    name: String,
    position: Vec3,
    azimuth: f32,
    elevation: f32,
    spread: f32,
    strength: f32,
    duration: f32,
    turbulence: f32,
    seed: u32,
    enabled: bool,
    track_groom: bool,
    mode: WindMode,
    #[serde(skip)]
    started: Option<f32>,
}
impl Default for WindSource {
    fn default() -> Self {
        Self {
            id: 1,
            name: "Wind 1".into(),
            position: Vec3::new(-1.3, 1.4, 1.2),
            azimuth: 133.0,
            elevation: -13.0,
            spread: 45.0,
            strength: 6.0,
            duration: 1.2,
            turbulence: 0.25,
            seed: 1,
            enabled: true,
            track_groom: true,
            mode: WindMode::Gust,
            started: None,
        }
    }
}
impl WindSource {
    fn direction(&self) -> Vec3 {
        let a = self.azimuth.to_radians();
        let e = self.elevation.to_radians();
        Vec3::new(a.sin() * e.cos(), e.sin(), a.cos() * e.cos())
    }
    fn aim(&mut self) {
        let delta = Vec3::new(0.0, 1.0, 0.0) - self.position;
        self.azimuth = delta.x.atan2(delta.z).to_degrees();
        self.elevation = delta
            .y
            .atan2(delta.x.hypot(delta.z))
            .to_degrees()
            .clamp(-80.0, 80.0);
    }
    fn intensity(&self, time: f32) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        let elapsed = time - self.started.unwrap_or(0.0);
        let envelope = match self.mode {
            WindMode::Continuous => 1.0,
            WindMode::Gust => {
                if self.started.is_none() || elapsed < 0.0 || elapsed >= self.duration {
                    return 0.0;
                }
                gust_strength(self.duration - elapsed, self.duration, 1.0)
            }
        };
        let seed = mix_seed(self.seed);
        let phase = (seed & 65535) as f32 * 0.013 + (seed >> 16) as f32 * 0.071;
        let variation = 1.0 + self.turbulence * 0.3 * (elapsed * 1.7 + phase).sin();
        self.strength * envelope * variation
    }
    fn validate(&mut self) {
        self.position = self
            .position
            .clamp(Vec3::new(-4.0, -2.5, -4.0), Vec3::splat(4.0));
        self.azimuth = self.azimuth.clamp(-180.0, 180.0);
        self.elevation = self.elevation.clamp(-80.0, 80.0);
        self.spread = self.spread.clamp(5.0, 90.0);
        self.strength = self.strength.clamp(0.0, 20.0);
        self.duration = self.duration.clamp(0.2, 4.0);
        self.turbulence = self.turbulence.clamp(0.0, 1.0);
        self.name = self.name.chars().take(60).collect();
        self.started = None;
    }
}

impl Settings {
    fn winds(&self) -> &[WindSource] {
        self.wind_sources.as_deref().unwrap_or(&[])
    }
    fn winds_mut(&mut self) -> &mut Vec<WindSource> {
        self.wind_sources.get_or_insert_with(Vec::new)
    }
    fn normalize_winds(&mut self) {
        if self.wind_sources.is_none() {
            self.wind_sources = Some(vec![WindSource {
                position: self.source,
                azimuth: self.azimuth,
                elevation: self.elevation,
                spread: self.spread,
                strength: if self.wind > 0.0 {
                    self.wind * 4.0
                } else {
                    self.strength
                },
                duration: self.duration,
                track_groom: false,
                mode: if self.wind > 0.0 {
                    WindMode::Continuous
                } else {
                    WindMode::Gust
                },
                ..Default::default()
            }]);
        }
        self.winds_mut().truncate(16);
        for source in self.winds_mut() {
            source.validate();
        }
        self.selected_wind = self.selected_wind.min(self.winds().len().saturating_sub(1));
    }
    fn add_wind(&mut self) {
        if self.winds().len() >= 16 {
            return;
        }
        let id = self
            .winds()
            .iter()
            .map(|source| source.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        let angle = id as f32 * 2.3999631;
        let mut source = WindSource {
            position: Vec3::new(
                angle.sin() * 1.8,
                1.3 + 0.16 * (id % 4) as f32,
                angle.cos() * 1.8,
            ),
            id,
            name: format!("Wind {id}"),
            seed: id.wrapping_mul(1664525).wrapping_add(1013904223),
            mode: WindMode::Continuous,
            started: Some(self.time),
            ..Default::default()
        };
        source.aim();
        self.winds_mut().push(source);
        self.selected_wind = self.winds().len() - 1;
    }
    fn delete_wind(&mut self) {
        let selected = self.selected_wind;
        if selected < self.winds().len() {
            self.winds_mut().remove(selected);
        }
        self.selected_wind = selected.min(self.winds().len().saturating_sub(1));
    }
    fn can_emit(&self) -> bool {
        self.winds()
            .get(self.selected_wind)
            .is_some_and(|source| source.enabled && source.mode == WindMode::Gust)
    }
    fn emit_selected(&mut self) {
        let index = self.selected_wind;
        let time = self.time;
        if let Some(source) = self.winds_mut().get_mut(index)
            && source.enabled
            && source.mode == WindMode::Gust
        {
            source.started = Some(time);
        }
    }
    fn emit_all_gusts(&mut self, time: f32) {
        for source in self.winds_mut() {
            if source.enabled && source.mode == WindMode::Gust {
                source.started = Some(time);
            }
        }
    }
    fn stop_gusts(&mut self) {
        for source in self.winds_mut() {
            source.started = None;
        }
    }
}

fn mix_seed(mut seed: u32) -> u32 {
    seed = (seed ^ (seed >> 16)).wrapping_mul(0x7feb352d);
    seed = (seed ^ (seed >> 15)).wrapping_mul(0x846ca68b);
    seed ^ (seed >> 16)
}

fn pack_wind_sources(values: &mut [f32; UNIFORM_FLOATS], settings: &Settings, time: f32) {
    values[56..316].fill(0.0);
    let mut active = 0;
    for source in settings.winds().iter().take(16) {
        let intensity = source.intensity(time);
        if intensity <= 0.0 {
            continue;
        }
        let offset = 60 + active * 16;
        let direction = source.direction();
        values[offset..offset + 4].copy_from_slice(&[
            source.position.x,
            source.position.y,
            source.position.z,
            source.spread.to_radians().cos(),
        ]);
        values[offset + 4..offset + 8].copy_from_slice(&[
            direction.x,
            direction.y,
            direction.z,
            intensity,
        ]);
        let seed = mix_seed(source.seed);
        values[offset + 8..offset + 12].copy_from_slice(&[
            (seed & 65535) as f32,
            (seed >> 16) as f32,
            source.turbulence,
            (time - source.started.unwrap_or(0.0)).max(0.0),
        ]);
        active += 1;
    }
    values[56] = active as f32;
}

impl Mhsb {
    fn wind_controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Build your airflow");
        ui.small("Each source has its own mode, position, and repeatable seed.");
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(self.s.winds().len() < 16, egui::Button::new("+ Add source"))
                .clicked()
            {
                self.s.add_wind();
                self.s.paused = false;
            }
            if ui
                .add_enabled(
                    !self.s.winds().is_empty(),
                    egui::Button::new("Delete selected"),
                )
                .clicked()
            {
                self.s.delete_wind();
            }
        });
        egui::ComboBox::from_id_salt("wind_sources")
            .selected_text(
                self.s
                    .winds()
                    .get(self.s.selected_wind)
                    .map(|source| source.name.as_str())
                    .unwrap_or("No sources"),
            )
            .show_ui(ui, |ui| {
                let mut selection = self.s.selected_wind;
                for (index, source) in self.s.winds().iter().enumerate() {
                    ui.selectable_value(&mut selection, index, &source.name);
                }
                self.s.selected_wind = selection;
            });
        ui.checkbox(&mut self.s.show_source, "Show source handles");
        let selected = self.s.selected_wind;
        let time = self.s.time;
        let paused = self.s.paused;
        let mut emitted = false;
        let Some(source) = self.s.winds_mut().get_mut(selected) else {
            ui.small("Add a source to apply wind.");
            return;
        };
        ui.group(|ui| {
            ui.text_edit_singleline(&mut source.name);
            ui.checkbox(&mut source.enabled, "Enabled");
            ui.horizontal(|ui| {
                let changed = ui
                    .selectable_value(&mut source.mode, WindMode::Continuous, "Continuous")
                    .changed();
                let changed = ui
                    .selectable_value(&mut source.mode, WindMode::Gust, "Gust")
                    .changed()
                    || changed;
                if changed {
                    source.started = if source.mode == WindMode::Continuous {
                        Some(time)
                    } else {
                        None
                    };
                }
            });
            let status = if !source.enabled {
                "Disabled"
            } else if paused {
                "Paused"
            } else if source.strength == 0.0 {
                "Strength is zero"
            } else if source.mode == WindMode::Continuous {
                "Continuous · running"
            } else if source.intensity(time) > 0.0 {
                "Gust · active"
            } else {
                "Gust · ready to emit"
            };
            ui.label(egui::RichText::new(status).color(egui::Color32::from_rgb(142, 217, 211)));
            ui.add(egui::Slider::new(&mut source.strength, 0.0..=20.0).text("Strength"));
            if source.mode == WindMode::Gust {
                if ui
                    .add(egui::Slider::new(&mut source.duration, 0.2..=4.0).text("Duration s"))
                    .changed()
                {
                    source.started = None;
                }
                if ui
                    .add_enabled(source.enabled, egui::Button::new("Emit seeded gust ↗"))
                    .clicked()
                {
                    source.started = Some(time);
                    emitted = true;
                }
            }
            ui.horizontal(|ui| {
                ui.label("Seed");
                ui.add(egui::DragValue::new(&mut source.seed).range(0..=u32::MAX));
                if ui.button("New seed").clicked() {
                    source.seed = source.seed.wrapping_mul(1664525).wrapping_add(1013904223);
                }
            });
            ui.add(egui::Slider::new(&mut source.turbulence, 0.0..=1.0).text("Variation"));
            ui.small("The same seed and replay settings produce the same wind pattern.");
        });
        ui.add_space(12.0);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                for (name, position) in [
                    ("Left", [-1.5, 1.4, 0.8]),
                    ("Right", [1.5, 1.4, 0.8]),
                    ("Above", [0.0, 2.4, 0.8]),
                    ("Back", [0.0, 1.4, -2.0]),
                ] {
                    if ui.button(name).clicked() {
                        source.position = position.into();
                        source.aim();
                    }
                }
            });
            let mut moved = ui
                .add(egui::Slider::new(&mut source.position.x, -4.0..=4.0).text("Source X"))
                .changed();
            moved |= ui
                .add(egui::Slider::new(&mut source.position.y, -2.5..=4.0).text("Source Y"))
                .changed();
            moved |= ui
                .add(egui::Slider::new(&mut source.position.z, -4.0..=4.0).text("Source Z"))
                .changed();
            moved |= ui
                .checkbox(&mut source.track_groom, "Track groom while moving")
                .changed();
            if moved && source.track_groom {
                source.aim();
            }
            let mut aimed = ui
                .add(egui::Slider::new(&mut source.azimuth, -180.0..=180.0).text("Azimuth"))
                .changed();
            aimed |= ui
                .add(egui::Slider::new(&mut source.elevation, -80.0..=80.0).text("Elevation"))
                .changed();
            if aimed {
                source.track_groom = false;
            }
            ui.add(egui::Slider::new(&mut source.spread, 5.0..=90.0).text("Spread"));
            if ui.button("Aim at hair ↗").clicked() {
                source.aim();
            }
        });
        if emitted {
            self.s.paused = false;
        }
    }
}

#[cfg(test)]
mod wind_tests {
    use super::*;
    #[test]
    fn seed_repeats_and_different_seeds_change_pattern() {
        let a = WindSource {
            mode: WindMode::Continuous,
            turbulence: 1.0,
            ..Default::default()
        };
        let mut b = a.clone();
        b.seed = 2;
        assert_eq!(a.intensity(2.3), a.clone().intensity(2.3));
        assert_ne!(a.intensity(2.3), b.intensity(2.3));
    }
    #[test]
    fn gust_requires_trigger_and_ends() {
        let mut source = WindSource::default();
        assert_eq!(source.intensity(1.0), 0.0);
        source.started = Some(1.0);
        assert!(source.intensity(1.6) > 0.0);
        assert_eq!(source.intensity(3.0), 0.0);
        source.enabled = false;
        assert_eq!(source.intensity(1.6), 0.0);
    }
    #[test]
    fn deleting_last_source_clears_force_and_roundtrips() {
        let mut settings = Settings::default();
        settings.delete_wind();
        assert!(settings.winds().is_empty());
        let mut values = [0.0; UNIFORM_FLOATS];
        pack_wind_sources(&mut values, &settings, 1.0);
        assert!(values[56..].iter().all(|value| *value == 0.0));
        let mut restored: Settings =
            serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
        restored.validate();
        assert!(restored.winds().is_empty());
    }
    #[test]
    fn creating_selecting_and_deleting_sources_is_bounded() {
        let mut settings = Settings::default();
        for _ in 0..30 {
            settings.add_wind();
        }
        assert_eq!(settings.winds().len(), 16);
        assert_eq!(settings.selected_wind, 15);
        let removed = settings.winds()[15].id;
        settings.delete_wind();
        assert_eq!(settings.winds().len(), 15);
        assert!(!settings.winds().iter().any(|source| source.id == removed));
    }
    #[test]
    fn old_single_source_presets_migrate() {
        let mut settings: Settings =
            serde_json::from_str(r#"{"source":[2.0,1.0,0.0],"wind":1.5}"#).unwrap();
        settings.validate();
        assert_eq!(settings.winds().len(), 1);
        assert_eq!(settings.winds()[0].position.x, 2.0);
        assert!(settings.winds()[0].mode == WindMode::Continuous);
    }
    #[test]
    fn source_uniform_layout_is_aligned_and_allows_sixteen() {
        assert_eq!(UNIFORM_FLOATS * 4, 1616);
        assert_eq!(UNIFORM_STRIDE * 4 % 256, 0);
        let mut settings = Settings::default();
        settings.add_wind();
        let mut packed = [0.0; UNIFORM_FLOATS];
        pack_wind_sources(&mut packed, &settings, 1.0);
        assert_eq!(packed[56], 1.0);
        assert!(packed[60 + 7] > 0.0);
    }
}
