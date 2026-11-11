#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct SavedPreset {
    name: String,
    settings: Settings,
}

fn preset_path() -> std::path::PathBuf {
    let directory = std::env::var_os("MHSB_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("LOCALAPPDATA")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(std::env::temp_dir)
                .join("MHSB")
        });
    directory.join("presets.json")
}

fn read_presets() -> Vec<SavedPreset> {
    std::fs::read(preset_path())
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

impl Settings {
    fn validate(&mut self) {
        if ![1024, 4096, 8192].contains(&self.guides) {
            self.guides = 4096;
        }
        self.fibers = self.fibers.clamp(1000, 1_000_000);
        self.style = self.style.min(5);
        self.groom.validate();
        self.color = self.color.min(13);
        self.wind = self.wind.clamp(0.0, 3.0);
        self.memory = self.memory.clamp(0.0, 1.0);
        self.length = self.length.clamp(0.5, 1.4);
        self.interaction = self.interaction.clamp(0.0, 1.0);
        self.friction = self.friction.clamp(0.0, 1.0);
        self.iterations = self.iterations.clamp(16, 64);
        self.source = self
            .source
            .clamp(Vec3::new(-4.0, -1.0, -4.0), Vec3::splat(4.0));
        self.azimuth = self.azimuth.clamp(-180.0, 180.0);
        self.elevation = self.elevation.clamp(-80.0, 80.0);
        self.spread = self.spread.clamp(10.0, 90.0);
        self.strength = self.strength.clamp(0.0, 20.0);
        self.duration = self.duration.clamp(0.2, 4.0);
        self.distance = self.distance.clamp(1.0, 14.0);
        self.pitch = self.pitch.clamp(-1.55, 1.55);
        if self.frame_limit > 1000 {
            self.frame_limit = 1000;
        }

        self.light_rig = self.light_rig.min(3);
        self.quality = self.quality.min(3);
        self.clumping = self.clumping.clamp(0.0, 1.0);
        self.exposure = self.exposure.clamp(-3.0, 3.0);
        self.roughness = self.roughness.clamp(0.12, 0.6);
        self.diameter = self.diameter.clamp(0.2, 1.2);
        self.flyaways = self.flyaways.clamp(0.0, 0.15);
        self.optical_density = self.optical_density.clamp(0.0, 3.0);
        self.debug_lobe = self.debug_lobe.min(4);

        self.env_mode = self.env_mode.min(4);
        self.sky_quality = self.sky_quality.min(4);
        self.lighting_quality = self.lighting_quality.min(4);
        self.fov = self.fov.clamp(15.0, 90.0);
        self.camera_pan = self.camera_pan.clamp(Vec3::splat(-10.0), Vec3::splat(10.0));
        self.sky_rotation = self.sky_rotation.clamp(-180.0, 180.0);
        self.sky_exposure = self.sky_exposure.clamp(-5.0, 5.0);
        self.sun_azimuth = self.sun_azimuth.clamp(-180.0, 180.0);
        self.sun_elevation = self.sun_elevation.clamp(-90.0, 90.0);
        self.sun_strength = self.sun_strength.clamp(0.0, 10.0);
        self.moon_strength = self.moon_strength.clamp(0.0, 1.0);
        self.moon_phase = self.moon_phase.clamp(0.0, 1.0);
        self.environment_strength = self.environment_strength.clamp(0.0, 4.0);
        self.studio_strength = self.studio_strength.clamp(0.0, 3.0);
        self.time_of_day = self.time_of_day.clamp(0.0, 24.0);
        self.curl_radius = self.curl_radius.clamp(0.3, 1.8);
        self.curl_tightness = self.curl_tightness.clamp(0.5, 1.5);
        self.hair_volume = self.hair_volume.clamp(0.5, 1.5);
        self.fringe = self.fringe.clamp(0.5, 1.5);
        self.root_density = self.root_density.clamp(0.3, 2.0);
        self.clump_width = self.clump_width.clamp(0.5, 2.0);
        self.light_size = self.light_size.clamp(0.01, 0.5);
        self.sun_size = self.sun_size.clamp(0.002, 0.03);
        self.studio_position = self
            .studio_position
            .clamp(Vec3::splat(-8.0), Vec3::splat(8.0));
        self.normalize_winds();
        self.time = 0.0;
        self.angle = 0.0;
        self.pulse = 0.0;
        self.paused = false;
        self.grab_active = false;
    }
}

impl Mhsb {
    fn save_preset(&mut self) -> Result<(), String> {
        let name = self.preset_name.trim();
        if name.is_empty() {
            return Err("Give this setup a name first.".into());
        }
        let preset = SavedPreset {
            name: name.chars().take(60).collect(),
            settings: self.s.clone(),
        };
        let mut presets = self.presets.clone();
        let index = presets
            .iter()
            .position(|p| p.name == preset.name)
            .unwrap_or(presets.len());
        if index == presets.len() {
            presets.push(preset);
        } else {
            presets[index] = preset;
        }
        let path = preset_path();
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let bytes = serde_json::to_vec_pretty(&presets).map_err(|e| e.to_string())?;
        let temporary = path.with_extension("json.tmp");
        std::fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
        std::fs::rename(&temporary, &path).map_err(|e| e.to_string())?;
        self.presets = presets;
        self.selected_preset = index;
        Ok(())
    }
    fn load_preset(&mut self) {
        if let Some(preset) = self.presets.get(self.selected_preset) {
            self.s = preset.settings.clone();
            self.s.validate();
            self.paint.before=None;
            self.paint.undo.clear();
            self.paint.redo.clear();
            if self.s.frame_limit > 0 {
                self.fps_input = self.s.frame_limit.to_string();
            }
            self.preset_name = preset.name.clone();
            self.reset = true;
            self.replay_tick = None;
            self.message = format!("Loaded {}", preset.name);
        }
    }
    fn replay(&mut self) {
        self.reset = true;
        self.s.paused = false;
        self.s.grab_active = false;
        self.replay_tick = Some(0);
        self.message = "Replay · settle, gust, recover".into();
    }
    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.image((self.logo.id(), egui::vec2(44.0, 44.0)));
            ui.image((self.wordmark.id(), egui::vec2(125.0, 48.0)));
        });
        ui.label(
            egui::RichText::new("HAIR SIMULATION")
                .size(10.0)
                .color(egui::Color32::from_rgb(142, 172, 189)),
        );
        ui.add_space(14.0);
        ui.horizontal_wrapped(|ui| {
            for (i, name) in [
                "Groom", "Physics", "Wind", "Presets", "Display", "Camera", "Sky", "Paint",
            ]
            .iter()
            .enumerate()
            {
                ui.selectable_value(&mut self.tab, i as u8, *name);
            }
        });
        ui.add_space(16.0);
        match self.tab {
            0 => {
                ui.heading("Hair");
                ui.collapsing("Shape adjustments", |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.s.root_density, 0.3..=2.0).text("Root density"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.s.clump_width, 0.5..=2.0).text("Clump width"),
                    );
                    let mut rebuild = false;
                    rebuild |= ui
                        .add(
                            egui::Slider::new(&mut self.s.curl_radius, 0.3..=1.8)
                                .text("Curl radius"),
                        )
                        .changed();
                    rebuild |= ui
                        .add(
                            egui::Slider::new(&mut self.s.curl_tightness, 0.5..=1.5)
                                .text("Curl tightness"),
                        )
                        .changed();
                    rebuild |= ui
                        .add(
                            egui::Slider::new(&mut self.s.hair_volume, 0.5..=1.5)
                                .text("Afro volume"),
                        )
                        .changed();
                    rebuild |= ui
                        .add(egui::Slider::new(&mut self.s.fringe, 0.5..=1.5).text("Fringe length"))
                        .changed();
                    if rebuild {
                        self.reset = true;
                    }
                });
                ui.small("Select a preset, then adjust its shape and material.");
                ui.add_space(12.0);
                ui.group(|ui| {
                    let mut chosen = None;
                    egui::ComboBox::from_label("Hairstyle preset")
                        .selected_text(&hairstyles()[self.s.style as usize].name)
                        .show_ui(ui, |ui| {
                            for (index, preset) in hairstyles().iter().enumerate() {
                                ui.horizontal(|ui| {
                                    let image = ui.add(
                                        egui::Image::new((
                                            self.groom_thumbnails[index].id(),
                                            egui::vec2(48.0, 48.0),
                                        ))
                                        .sense(egui::Sense::click()),
                                    );
                                    if image.clicked()
                                        || ui
                                            .selectable_label(
                                                self.s.style == index as u32,
                                                &preset.name,
                                            )
                                            .clicked()
                                    {
                                        chosen = Some(index);
                                        ui.close();
                                    }
                                });
                            }
                        });
                    if let Some(index) = chosen {
                        self.s.apply_hairstyle(index);
                        self.reset = true;
                    }
                    if ui
                        .add(egui::Slider::new(&mut self.s.length, 0.5..=1.4).text("Length"))
                        .changed()
                    {
                        self.reset = true;
                    }
                    egui::ComboBox::from_label("Color")
                        .selected_text(
                            [
                                "Espresso",
                                "Chestnut",
                                "Honey",
                                "Silver",
                                "Black",
                                "Dark blonde",
                                "Auburn",
                                "Platinum",
                                "Ginger",
                                "Medium brown",
                                "Strawberry blonde",
                                "Salt & pepper",
                                "Chocolate",
                                "Warm brown",
                            ][self.s.color as usize],
                        )
                        .show_ui(ui, |ui| {
                            for (i, name) in [
                                "Espresso",
                                "Chestnut",
                                "Honey",
                                "Silver",
                                "Black",
                                "Dark blonde",
                                "Auburn",
                                "Platinum",
                                "Ginger",
                                "Medium brown",
                                "Strawberry blonde",
                                "Salt & pepper",
                                "Chocolate",
                                "Warm brown",
                            ]
                            .iter()
                            .enumerate()
                            {
                                ui.selectable_value(&mut self.s.color, i as u32, *name);
                            }
                        });
                });
                ui.add_space(14.0);
                ui.group(|ui| {
                    ui.strong("Density");
                    egui::ComboBox::from_label("Fiber budget")
                        .selected_text(self.s.fibers.to_string())
                        .show_ui(ui, |ui| {
                            for n in [10000, 100000, 200000, 250000, 300000, 500000, 1000000] {
                                ui.selectable_value(&mut self.s.fibers, n, n.to_string());
                            }
                        });
                    egui::ComboBox::from_label("Physics guides")
                        .selected_text(self.s.guides.to_string())
                        .show_ui(ui, |ui| {
                            for n in [1024, 4096, 8192] {
                                ui.selectable_value(&mut self.s.guides, n, n.to_string());
                            }
                        });
                    ui.small("Painted density controls which fibers are visible.");
                });
            }
            1 => {
                ui.heading("Give it weight");
                ui.add(egui::Slider::new(&mut self.s.clumping, 0.0..=1.0).text("Clumping"));
                ui.small("Volume pressure separates crowded guides. Friction shares their motion.");
                ui.add_space(12.0);
                ui.group(|ui| {
                    let changed = ui
                        .add(egui::Slider::new(&mut self.s.interaction, 0.0..=1.0).text("Volume"))
                        .changed();
                    let changed = ui
                        .add(egui::Slider::new(&mut self.s.friction, 0.0..=1.0).text("Friction"))
                        .changed()
                        || changed;
                    let changed = ui
                        .add(egui::Slider::new(&mut self.s.memory, 0.0..=1.0).text("Shape memory"))
                        .changed()
                        || changed;
                    if changed {
                        self.reset = true;
                    }
                    ui.add(egui::Slider::new(&mut self.s.iterations, 16..=64).text("Iterations"));
                    ui.checkbox(&mut self.s.motion, "Animate head");
                });
                ui.add_space(14.0);
                ui.group(|ui|{
                    ui.strong("Touch the hair");ui.checkbox(&mut self.s.tool_grab,"Grab & release tool");
                    ui.small("Drag a section in the viewport. Release to let it settle. Switch to Orbit to rotate the view.");
                });
            }
            2 => self.wind_controls(ui),
            3 => {
                ui.heading("Keep your experiments");
                ui.small("Save groom, physics, wind, camera, and display settings.");
                ui.add_space(12.0);
                ui.group(|ui| {
                    ui.label("Setup name");
                    ui.text_edit_singleline(&mut self.preset_name);
                    if ui.button("Save setup").clicked() {
                        self.message = match self.save_preset() {
                            Ok(()) => "Setup saved".into(),
                            Err(error) => format!("Save failed · {error}"),
                        };
                    }
                    egui::ComboBox::from_id_salt("saved_setups")
                        .selected_text(
                            self.presets
                                .get(self.selected_preset)
                                .map(|p| p.name.as_str())
                                .unwrap_or("No saved setups"),
                        )
                        .show_ui(ui, |ui| {
                            for (i, preset) in self.presets.iter().enumerate() {
                                ui.selectable_value(&mut self.selected_preset, i, &preset.name);
                            }
                        });
                    if ui.button("Load setup").clicked() {
                        self.load_preset();
                    }
                });
                ui.add_space(14.0);
                ui.group(|ui|{
                    ui.strong("Repeatable gust test");ui.small("720 fixed simulation steps: settle, emit the chosen gust, recover. The camera stays adjustable.");
                    if ui.button("▶ Replay test").clicked(){self.replay();}
                    if let Some(tick)=self.replay_tick{ui.add(egui::ProgressBar::new(tick as f32/720.0).show_percentage());}
                });
            }
            4 => {
                ui.heading("Graphics");
                ui.group(|ui| {
                    ui.checkbox(&mut self.s.god_opts,"GOD OPTS · Experimental");
                    ui.small("Shared strand frames and cached material shading. Same selected quality. Compare components separately.");
                });
                ui.collapsing("GOD OPTS components", |ui| {
                    ui.checkbox(&mut self.s.god_curves, "Reuse curve samples");
                    ui.checkbox(&mut self.s.god_frames, "Reuse strand frames");
                    ui.checkbox(&mut self.s.god_materials, "Cache material factors");
                    ui.checkbox(&mut self.s.god_coverage, "Reject zero-coverage shading");
                    ui.checkbox(&mut self.s.god_density_lod,"Optional screen-space LOD");
                    ui.small("LOD changes strand sampling and curve tessellation below 0.25 px to target the FPS cap. Close-ups and physics stay full detail. Off by default.");
                });
                ui.checkbox(&mut self.gpu.profiler.enabled, "Measure GPU time");
                ui.small(format!(
                    "{} × {} · {} fibers drawn · 60 FPS: 16.67 ms · 144 FPS: 6.94 ms",
                    self.gpu.size[0], self.gpu.size[1], self.s.drawn
                ));
                ui.small(self.gpu.profiler.summary());
                ui.small(format!(
                    "Guides: {:.1} MiB · HDR map: {:.1} MiB",
                    self.gpu.strands.size() as f32 / 1048576.0,
                    self.gpu.environment.width as f32
                        * (self.gpu.environment.width as f32 / 2.0)
                        * 8.0
                        * 4.0
                        / 3.0
                        / 1048576.0
                ));
                egui::ComboBox::from_label("Studio lighting")
                    .selected_text(
                        ["Neutral", "Portrait", "Backlight", "Mixed"][self.s.light_rig as usize],
                    )
                    .show_ui(ui, |ui| {
                        for (i, name) in ["Neutral", "Portrait", "Backlight", "Mixed"]
                            .iter()
                            .enumerate()
                        {
                            ui.selectable_value(&mut self.s.light_rig, i as u32, *name);
                        }
                    });

                egui::ComboBox::from_label("Graphics quality")
                    .selected_text(
                        ["Performance", "Balanced", "Cinematic", "Reference"]
                            [self.s.quality as usize],
                    )
                    .show_ui(ui, |ui| {
                        for (i, name) in ["Performance", "Balanced", "Cinematic", "Reference"]
                            .iter()
                            .enumerate()
                        {
                            ui.selectable_value(&mut self.s.quality, i as u32, *name);
                        }
                    });
                ui.add(egui::Slider::new(&mut self.s.exposure, -3.0..=3.0).text("Exposure"));
                ui.add(
                    egui::Slider::new(&mut self.s.optical_density, 0.0..=3.0)
                        .text("Shadow density"),
                );
                ui.add(egui::Slider::new(&mut self.s.roughness, 0.12..=0.6).text("Roughness"));
                ui.add(egui::Slider::new(&mut self.s.diameter, 0.2..=1.2).text("Fiber width"));
                ui.add(egui::Slider::new(&mut self.s.flyaways, 0.0..=0.15).text("Flyaways"));
                egui::ComboBox::from_label("Lighting view")
                    .selected_text(
                        [
                            "Beauty",
                            "Reflection R",
                            "Transmission TT",
                            "Internal TRT",
                            "Shadows",
                        ][self.s.debug_lobe as usize],
                    )
                    .show_ui(ui, |ui| {
                        for (i, name) in [
                            "Beauty",
                            "Reflection R",
                            "Transmission TT",
                            "Internal TRT",
                            "Shadows",
                        ]
                        .iter()
                        .enumerate()
                        {
                            ui.selectable_value(&mut self.s.debug_lobe, i as u32, *name);
                        }
                    });

                ui.add_space(12.0);
                ui.small("The physics step stays fixed while rendering runs at your chosen rate.");
                ui.add_space(12.0);
                ui.group(|ui|{
                    ui.label("Max FPS · 1 to 1000");
                    let input=ui.add(egui::TextEdit::singleline(&mut self.fps_input).desired_width(100.0));
                    let apply=input.lost_focus() && ui.input(|input|input.key_pressed(egui::Key::Enter));
                    if ui.button("Apply FPS limit").clicked() || apply {
                        match parse_frame_limit(&self.fps_input){
                            Ok(limit)=>{self.s.frame_limit=limit;self.fps_input=limit.to_string();self.message=format!("Max FPS set to {limit}");}
                            Err(error)=>self.message=error,
                        }
                    }
                    let mut uncapped=self.s.frame_limit==0;
                    if ui.checkbox(&mut uncapped,"Uncapped FPS").changed(){
                        self.s.frame_limit=if uncapped{0}else{parse_frame_limit(&self.fps_input).unwrap_or(60)};
                    }
                    ui.small("Starts at 60 FPS. Background rendering is limited to 15 FPS. Minimizing suspends the hair renderer.");
                    ui.small("Quality is fixed. Painted density controls coverage.");
                });
            }

            5 => {
                ui.heading("Camera");
                ui.small("Drag to orbit · Middle drag to pan · Shift + wheel to pan vertically · Wheel to zoom");
                ui.small("Double click hair to set the pivot. Hover the viewport and press F to frame the groom.");
                ui.add(egui::Slider::new(&mut self.s.fov, 15.0..=90.0).text("Field of view"));
                ui.add(egui::Slider::new(&mut self.s.distance, 1.0..=14.0).text("Distance"));
                ui.add(egui::Slider::new(&mut self.s.camera_pan.x, -4.0..=4.0).text("Pan X"));
                ui.add(egui::Slider::new(&mut self.s.camera_pan.y, -4.0..=4.0).text("Pan Y"));
                ui.add(egui::Slider::new(&mut self.s.camera_pan.z, -4.0..=4.0).text("Pan Z"));
                ui.horizontal_wrapped(|ui| {
                    for (label, yaw, pitch) in [
                        ("Front", 0.0, 0.0),
                        ("Side", std::f32::consts::FRAC_PI_2, 0.0),
                        ("Back", std::f32::consts::PI, 0.0),
                        ("Top", 0.0, 1.55),
                    ] {
                        if ui.button(label).clicked() {
                            self.s.yaw = yaw;
                            self.s.pitch = pitch;
                        }
                    }
                    if ui.button("Frame groom").clicked() {
                        self.frame_painted_groom();
                    }
                });

                ui.small("Save setup includes camera position, field of view, and pivot.");
                ui.separator();
                ui.strong("Screenshot");
                egui::ComboBox::from_label("Output width")
                    .selected_text(if self.capture_width == 0 {
                        "Viewport".into()
                    } else {
                        format!("{} px", self.capture_width)
                    })
                    .show_ui(ui, |ui| {
                        for (width, name) in [
                            (0, "Viewport"),
                            (1920, "1920 px"),
                            (2560, "2560 px"),
                            (3840, "3840 px"),
                        ] {
                            ui.selectable_value(&mut self.capture_width, width, name);
                        }
                    });
                ui.checkbox(
                    &mut self.capture_supersample,
                    "Supersample · 4096 px render cap",
                );
                ui.label("PNG destination");
                ui.text_edit_singleline(&mut self.capture_path);
                if ui
                    .add_enabled(
                        self.capture_pending.is_none() && self.gpu.environment.pending.is_none(),
                        egui::Button::new("Export PNG"),
                    )
                    .clicked()
                {
                    self.capture_request = true;
                }
            }
            6 => {
                ui.heading("Sky and environment");
                egui::ComboBox::from_label("Environment")
                    .selected_text(
                        [
                            "Studio",
                            "Mud Road PureSky",
                            "Citrus Orchard PureSky",
                            "Procedural sky",
                            "Custom HDR/EXR",
                        ][self.s.env_mode as usize],
                    )
                    .show_ui(ui, |ui| {
                        for (i, name) in [
                            "Studio",
                            "Mud Road PureSky",
                            "Citrus Orchard PureSky",
                            "Procedural sky",
                        ]
                        .iter()
                        .enumerate()
                        {
                            ui.selectable_value(&mut self.s.env_mode, i as u32, *name);
                        }
                    });
                for (label, value) in [
                    ("Background quality", &mut self.s.sky_quality),
                    ("Lighting quality", &mut self.s.lighting_quality),
                ] {
                    egui::ComboBox::from_label(label)
                        .selected_text(
                            ["Auto", "Performance", "Balanced", "Cinematic", "Reference"]
                                [*value as usize],
                        )
                        .show_ui(ui, |ui| {
                            for (i, name) in
                                ["Auto", "Performance", "Balanced", "Cinematic", "Reference"]
                                    .iter()
                                    .enumerate()
                            {
                                ui.selectable_value(value, i as u32, *name);
                            }
                        });
                }

                ui.label("Custom HDR / EXR panorama path");
                ui.text_edit_singleline(&mut self.s.custom_environment);
                if ui.button("Load custom environment").clicked() {
                    self.s.env_mode = 4;
                    self.s.custom_generation = self.s.custom_generation.wrapping_add(1);
                }
                ui.checkbox(&mut self.s.show_sky, "Show sky background");
                ui.add(
                    egui::Slider::new(&mut self.s.sky_rotation, -180.0..=180.0)
                        .text("Sky rotation"),
                );
                ui.add(
                    egui::Slider::new(&mut self.s.sky_exposure, -5.0..=5.0).text("Sky exposure"),
                );
                ui.add(
                    egui::Slider::new(&mut self.s.environment_strength, 0.0..=4.0)
                        .text("Environment light"),
                );
                ui.add(
                    egui::Slider::new(&mut self.s.studio_strength, 0.0..=3.0).text("Studio light"),
                );

                ui.checkbox(&mut self.s.show_light, "Show draggable key light");
                let key_x = ui
                    .add(egui::Slider::new(&mut self.s.studio_position.x, -8.0..=8.0).text("Key X"))
                    .changed();
                let key_y = ui
                    .add(egui::Slider::new(&mut self.s.studio_position.y, -4.0..=8.0).text("Key Y"))
                    .changed();
                let key_z = ui
                    .add(egui::Slider::new(&mut self.s.studio_position.z, -8.0..=8.0).text("Key Z"))
                    .changed();
                if key_x || key_y || key_z {
                    self.s.light_rig = 0;
                }
                ui.add(
                    egui::Slider::new(&mut self.s.light_size, 0.01..=0.5).text("Key light size"),
                );
                ui.label(&self.gpu.environment.status);
                ui.horizontal(|ui| {
                    for (name, hour, azimuth, elevation, sun, moon) in [
                        ("Day", 12.0, -160.0, 45.0, 1.5, 0.0),
                        ("Sunset", 18.0, -160.0, 4.0, 1.2, 0.0),
                        ("Moonlit", 0.0, 20.0, -12.0, 0.0, 0.25),
                    ] {
                        if ui.button(name).clicked() {
                            self.s.env_mode = 3;
                            self.s.studio_strength = 0.0;
                            self.s.time_of_day = hour;
                            self.s.sun_azimuth = azimuth;
                            self.s.sun_elevation = elevation;
                            self.s.sun_strength = sun;
                            self.s.moon_strength = moon;
                        }
                    }
                });
                ui.add(egui::Slider::new(&mut self.s.sun_size, 0.002..=0.03).text("Sun disc size"));
                ui.separator();
                ui.strong("Sun and moon");
                if ui
                    .add(egui::Slider::new(&mut self.s.time_of_day, 0.0..=24.0).text("Time of day"))
                    .changed()
                {
                    self.s.env_mode = 3;
                    self.s.studio_strength = 0.0;
                    self.s.sun_azimuth = (self.s.time_of_day - 12.0) * 15.0;
                    self.s.sun_elevation =
                        ((self.s.time_of_day - 6.0) * std::f32::consts::PI / 12.0).sin() * 70.0;
                    self.s.sun_strength = if self.s.sun_elevation > 0.0 { 1.5 } else { 0.0 };
                    self.s.moon_strength = if self.s.sun_elevation < 0.0 { 0.2 } else { 0.0 };
                }
                ui.add(
                    egui::Slider::new(&mut self.s.sun_azimuth, -180.0..=180.0).text("Sun azimuth"),
                );
                ui.add(
                    egui::Slider::new(&mut self.s.sun_elevation, -90.0..=90.0)
                        .text("Sun elevation"),
                );
                ui.add(
                    egui::Slider::new(&mut self.s.sun_strength, 0.0..=10.0).text("Sun brightness"),
                );
                ui.add(
                    egui::Slider::new(&mut self.s.moon_strength, 0.0..=1.0).text("Moon brightness"),
                );
                ui.add(egui::Slider::new(&mut self.s.moon_phase, 0.0..=1.0).text("Moon phase"));
                ui.small("HDR skies retain their photographed illumination. Added sun and moon lights are independent.");
            }
            7 => self.paint_controls(ui),
            _ => {}
        }
        ui.add_space(20.0);
        ui.separator();
        ui.label(format!(
            "{:.0} FPS   ·   {} fibers submitted",
            self.fps, self.s.drawn
        ));
        ui.small(format!(
            "{} guides  ·  {:.2} MiB strand storage",
            self.s.guides,
            self.s.guides as f32 * 25.0 * 32.0 / 1048576.0
        ));
        if !self.message.is_empty() {
            ui.add_space(8.0);
            ui.label(&self.message);
        }
        ui.add_space(12.0);
        ui.small("Drag to orbit · Scroll to zoom");
    }
}

impl Gpu {
    fn pick(&self, matrix: Mat4, rect: egui::Rect, pointer: egui::Pos2) -> Option<Vec3> {
        let staging = self.rs.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Pick hair"),
            size: self.strands.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self.rs.device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(&self.strands, 0, &staging, 0, self.strands.size());
        self.rs.queue.submit([encoder.finish()]);
        let (sender, receiver) = std::sync::mpsc::channel();
        staging
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send(result);
            });
        if self
            .rs
            .device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(std::time::Duration::from_secs(30)),
            })
            .is_err()
            || receiver.recv().ok()?.is_err()
        {
            return None;
        }
        let mapped = staging.slice(..).get_mapped_range();
        let values: &[f32] = bytemuck::cast_slice(&mapped);
        let mut best = 28.0_f32.powi(2);
        let mut point = None;
        for (index, node) in values.as_chunks::<8>().0.iter().enumerate() {
            if index % 25 < 3 {
                continue;
            }
            let p = Vec3::new(node[0], node[1], node[2]);
            let clip = matrix * p.extend(1.0);
            if clip.w <= 0.05 {
                continue;
            }
            let projected = clip.truncate() / clip.w;
            let screen = egui::pos2(
                rect.left() + (projected.x * 0.5 + 0.5) * rect.width(),
                rect.top() + (0.5 - projected.y * 0.5) * rect.height(),
            );
            let distance = screen.distance_sq(pointer);
            if distance < best {
                best = distance;
                point = Some(p);
            }
        }
        drop(mapped);
        staging.unmap();
        point
    }
}

fn parse_frame_limit(text: &str) -> Result<u32, String> {
    match text.trim().parse::<u32>() {
        Ok(value @ 1..=1000) => Ok(value),
        _ => Err("Enter a whole number from 1 to 1000.".into()),
    }
}

fn effective_frame_limit(requested: u32, focused: bool) -> u32 {
    if focused {
        requested
    } else if requested == 0 {
        15
    } else {
        requested.min(15)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn custom_frame_limits_accept_valid_numbers() {
        for value in [1, 30, 60, 73, 144, 240, 1000] {
            assert_eq!(parse_frame_limit(&value.to_string()).unwrap(), value);
        }
        assert_eq!(parse_frame_limit(" 144 ").unwrap(), 144);
    }
    #[test]
    fn invalid_frame_limits_do_not_apply() {
        for value in ["", "0", "1001", "-1", "60.5", "text"] {
            assert!(parse_frame_limit(value).is_err());
        }
    }
    #[test]
    fn background_pacing_applies_even_when_uncapped() {
        assert_eq!(effective_frame_limit(0, false), 15);
        assert_eq!(effective_frame_limit(240, false), 15);
        assert_eq!(effective_frame_limit(10, false), 10);
        assert_eq!(effective_frame_limit(0, true), 0);
        assert_eq!(effective_frame_limit(73, true), 73);
    }
    #[test]
    fn presets_preserve_custom_frame_limits() {
        let saved = Settings {
            frame_limit: 73,
            ..Default::default()
        };
        let json = serde_json::to_string(&saved).unwrap();
        let mut loaded: Settings = serde_json::from_str(&json).unwrap();
        loaded.validate();
        assert_eq!(loaded.frame_limit, 73);
        let mut legacy: Settings = serde_json::from_str("{}").unwrap();
        legacy.validate();
        assert_eq!(legacy.frame_limit, 60);
    }
}

fn substep_clock(end_time: f32, remaining: f32, active: bool, step: u32, steps: u32) -> (f32, f32) {
    let before = (steps - 1 - step) as f32 * STEP;
    (
        (end_time - before).max(0.0),
        if active { remaining + before } else { 0.0 },
    )
}
fn gust_strength(remaining: f32, duration: f32, strength: f32) -> f32 {
    if remaining > 0.0 && remaining <= duration {
        strength * (std::f32::consts::PI * remaining / duration).sin().max(0.0)
    } else {
        0.0
    }
}
#[cfg(test)]
mod simulation_tests {
    use super::*;
    #[test]
    fn every_substep_has_its_own_time() {
        let samples: Vec<_> = (0..6)
            .map(|step| substep_clock(6.0 * STEP, 0.0, false, step, 6).0)
            .collect();
        for (i, time) in samples.iter().enumerate() {
            assert!((*time - (i + 1) as f32 * STEP).abs() < 0.000001);
        }
    }
    #[test]
    fn inactive_gust_stays_inactive_for_all_substeps() {
        for step in 0..6 {
            assert_eq!(substep_clock(1.0, 0.0, false, step, 6).1, 0.0);
        }
    }
    #[test]
    fn finishing_gust_retains_last_substep_samples() {
        let (_, remaining) = substep_clock(1.0, 0.0, true, 0, 6);
        assert!(remaining > 0.0);
        assert!(gust_strength(remaining, 1.0, 6.0) > 0.0);
        assert_eq!(gust_strength(0.0, 1.0, 6.0), 0.0);
    }
    #[test]
    fn all_shaders_validate_without_a_gpu() {
        for source in [
            include_str!("simulation.wgsl"),
            include_str!("hair.wgsl"),
            include_str!("scene.wgsl"),
            include_str!("post.wgsl"),
        ] {
            let module = naga::front::wgsl::parse_str(source).expect("WGSL syntax");
            naga::valid::Validator::new(
                naga::valid::ValidationFlags::all(),
                naga::valid::Capabilities::all(),
            )
            .validate(&module)
            .expect("WGSL validation");
        }
        assert!(!include_str!("simulation.wgsl").contains("previousQuiet"));
        assert!(!include_str!("simulation.wgsl").contains("idleSteps"));
    }
}
