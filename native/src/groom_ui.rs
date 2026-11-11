impl Mhsb {
    fn frame_painted_groom(&mut self) {
        let mut lo = Vec3::new(-1.77, -0.54, -0.79);
        let mut hi = Vec3::new(1.77, 2.1, 0.79);
        for root in self
            .gpu
            .root_cache
            .records
            .iter()
            .take(self.s.guides as usize)
            .filter(|r| r.parameters[0] > 0.001)
        {
            for p in &root.rest {
                let v = rotate_cpu(Vec3::from_slice(p), self.s.angle * root.position[3]);
                lo = lo.min(v);
                hi = hi.max(v);
            }
        }
        let center = (lo + hi) * 0.5;
        let radius = (hi - lo).length() * 0.5;
        let aspect = self.gpu.size[0] as f32 / self.gpu.size[1].max(1) as f32;
        let half = (self.s.fov.to_radians() * 0.5)
            .tan()
            .min((self.s.fov.to_radians() * 0.5).tan() * aspect)
            .atan();
        self.s.distance = (radius / half.sin() * 1.05).clamp(1.0, 14.0);
        let base = match self.s.style {
            3 | 4 => 1.35,
            5 => 0.95,
            0 | 1 => -0.05,
            _ => 0.72,
        };
        self.s.camera_pan = center - Vec3::new(0.0, base, 0.0);
    }
    fn paint_controls(&mut self, ui: &mut egui::Ui) {
        let before = self.s.groom.clone();
        let mut history_action = false;
        ui.heading("Paint hair");
        ui.small("Paint on any part of the human mesh. New roots are placed under your brush.");
        ui.horizontal_wrapped(|ui| {
            for (tool, label) in [
                (PaintTool::Density, "Grow · G"),
                (PaintTool::Erase, "Erase · E"),
                (PaintTool::Length, "Length · L"),
                (PaintTool::Comb, "Comb · C"),
            ] {
                if ui
                    .selectable_label(self.paint.tool == tool, label)
                    .clicked()
                {
                    self.paint.tool = tool;
                    self.s.tool_grab = false;
                }
            }
        });
        ui.checkbox(
            &mut self.s.surface_view,
            "See skin · hide hair while editing",
        );
        ui.small("Hold Ctrl to erase · [ / ] brush size · Alt-drag orbit · Esc finish");
        egui::ComboBox::from_label("More brushes")
            .selected_text(paint_tool_name(self.paint.tool))
            .show_ui(ui, |ui| {
                for tool in [
                    PaintTool::Off,
                    PaintTool::Density,
                    PaintTool::Erase,
                    PaintTool::Length,
                    PaintTool::Comb,
                    PaintTool::Curl,
                    PaintTool::Clump,
                    PaintTool::Width,
                    PaintTool::Flyaway,
                    PaintTool::Color,
                    PaintTool::Smooth,
                ] {
                    if ui
                        .selectable_value(&mut self.paint.tool, tool, paint_tool_name(tool))
                        .changed()
                    {
                        self.s.tool_grab = false;
                    }
                }
            });
        let mut radius = self.paint.radius / 0.072;
        ui.add(egui::Slider::new(&mut radius, 0.3..=8.0).text("Brush radius · cm"));
        self.paint.radius = radius * 0.072;
        ui.add(egui::Slider::new(&mut self.paint.strength, 0.05..=1.0).text("Strength"));
        ui.add(egui::Slider::new(&mut self.paint.softness, 0.0..=1.0).text("Softness"));
        ui.checkbox(&mut self.paint.symmetry, "X symmetry");
        ui.checkbox(&mut self.paint.scalp_only, "Restrict to scalp");
        ui.checkbox(&mut self.paint.show_roots, "Show nearby roots");
        match self.paint.tool {
            PaintTool::Density | PaintTool::Length => {
                ui.horizontal(|ui| {
                    for (name, cm) in [("Short", 1.0), ("Medium", 15.0), ("Long", 40.0)] {
                        if ui.button(name).clicked() {
                            self.paint.length_cm = cm;
                        }
                    }
                });
                ui.add(
                    egui::Slider::new(&mut self.paint.length_cm, 0.5..=80.0).text("Length · cm"),
                );
            }
            PaintTool::Curl => {
                ui.add(egui::Slider::new(&mut self.paint.curl, 0.0..=2.5).text("Curl multiplier"));
            }
            PaintTool::Clump => {
                ui.add(
                    egui::Slider::new(&mut self.paint.clump, 0.0..=2.0).text("Clump multiplier"),
                );
            }
            PaintTool::Width => {
                ui.add(
                    egui::Slider::new(&mut self.paint.width, 0.2..=2.5)
                        .text("Thickness multiplier"),
                );
            }
            PaintTool::Flyaway => {
                ui.add(
                    egui::Slider::new(&mut self.paint.flyaway, 0.0..=0.3).text("Flyaway fraction"),
                );
            }
            PaintTool::Color => {
                egui::ComboBox::from_label("Paint color")
                    .selected_text(paint_color_name(self.paint.color))
                    .show_ui(ui, |ui| {
                        for i in 0..14 {
                            ui.selectable_value(&mut self.paint.color, i, paint_color_name(i));
                        }
                    });
            }
            PaintTool::Comb => {
                ui.small("Drag in the direction the hair should grow. Direction is stored on the surface, independent of the camera.");
            }
            _ => {}
        }
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!self.paint.undo.is_empty(), egui::Button::new("Undo"))
                .clicked()
            {
                self.paint.undo(&mut self.s.groom);
                history_action = true;
            }
            if ui
                .add_enabled(!self.paint.redo.is_empty(), egui::Button::new("Redo"))
                .clicked()
            {
                self.paint.redo(&mut self.s.groom);
                history_action = true;
            }
        });
        ui.small("Ctrl+Z · Undo     Ctrl+Shift+Z / Ctrl+Y · Redo");
        ui.separator();
        ui.collapsing("Layers",|ui| {
        for i in 0..self.s.groom.layers.len() {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.s.groom.layers[i].enabled, "");
                ui.selectable_value(&mut self.s.groom.selected, i, format!("{}", i + 1));
                ui.add(
                    egui::TextEdit::singleline(&mut self.s.groom.layers[i].name)
                        .desired_width(190.0),
                );
            });
        }
        let selected = self.s.groom.selected;
        ui.add(
            egui::Slider::new(&mut self.s.groom.layers[selected].opacity, 0.0..=1.0)
                .text("Layer opacity"),
        );
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    self.s.groom.layers.len() < 8,
                    egui::Button::new("Add layer"),
                )
                .clicked()
            {
                let i = self.s.groom.layers.len();
                self.s.groom.layers.push(GroomLayer {
                    name: format!("Layer {}", i + 1),
                    enabled: true,
                    opacity: 1.0,
                    base: false,
                    edits: BTreeMap::new(),
                });
                self.s.groom.selected = i;
            }
            if ui
                .add_enabled(
                    self.s.groom.layers.len() > 1,
                    egui::Button::new("Delete layer"),
                )
                .clicked()
            {
                self.s.groom.layers.remove(selected);
                self.s.groom.selected = selected.min(self.s.groom.layers.len() - 1);
            }
            if ui.button("Clear layer").clicked() {
                self.s.groom.layers[self.s.groom.selected].edits.clear();
            }
            if ui
                .add_enabled(selected > 0, egui::Button::new("Move up"))
                .clicked()
            {
                self.s.groom.layers.swap(selected, selected - 1);
                self.s.groom.selected -= 1;
            }
            if ui
                .add_enabled(
                    selected + 1 < self.s.groom.layers.len(),
                    egui::Button::new("Move down"),
                )
                .clicked()
            {
                self.s.groom.layers.swap(selected, selected + 1);
                self.s.groom.selected += 1;
            }
        });
        ui.small("Lower rows composite above earlier rows. Erase cuts hair; clearing a layer reveals the layers below it.");
        ui.horizontal(|ui| {
            if ui.button("Start blank").clicked() {
                self.s.groom = GroomDocument::default();
                self.s.groom.layers[0].base = false;
                self.s.groom.layers[0].name = "Painted groom".into();
            }
            if ui.button("Full scalp").clicked() {
                self.s.groom = GroomDocument::default();
            }
        });
        });
        if !self.paint.status.is_empty() {
            ui.small(&self.paint.status);
        }
        let active = self
            .gpu
            .root_cache
            .records
            .iter()
            .take(self.s.guides as usize)
            .filter(|r| r.parameters[0] > 0.001)
            .count();
        ui.small(format!("{active} / {} guide roots active", self.s.guides));
        if !history_action && before != self.s.groom && self.paint.before.is_none() {
            self.paint.before = Some(before);
            if !ui.input(|i| i.pointer.primary_down()) {
                self.paint.end(&self.s.groom);
            }
        }
        ui.separator();
        ui.collapsing("Groom files", |ui| {
            ui.text_edit_singleline(&mut self.paint.path);
            ui.horizontal(|ui| {
                if ui.button("Save groom").clicked() {
                    self.message = match save_groom_file(&self.paint.path, &self.s) {
                        Ok(()) => "Groom saved".into(),
                        Err(e) => e,
                    };
                }
                if ui.button("Load groom").clicked() {
                    match load_groom_file(&self.paint.path) {
                        Ok(doc) => {
                            apply_groom_project(&mut self.s, doc);
                            self.paint.before = None;
                            self.paint.undo.clear();
                            self.paint.redo.clear();
                            self.message = "Groom loaded".into();
                        }
                        Err(e) => self.message = e,
                    }
                }
            });
            ui.small(
                "Save setup also includes the painted groom. Placed roots are saved with the file.",
            );
        });
    }
    fn paint_viewport(
        &mut self,
        ctx: &egui::Context,
        ui: &egui::Ui,
        response: &egui::Response,
        matrix: Mat4,
    ) -> bool {
        if !ctx.wants_keyboard_input() && !ctx.input(|i| i.modifiers.command || i.modifiers.alt) {
            for (key, tool) in [
                (egui::Key::G, PaintTool::Density),
                (egui::Key::E, PaintTool::Erase),
                (egui::Key::L, PaintTool::Length),
                (egui::Key::C, PaintTool::Comb),
            ] {
                if ctx.input(|i| i.key_pressed(key)) {
                    self.paint.tool = tool;
                    self.s.tool_grab = false;
                    self.tab = 7;
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.paint.end(&self.s.groom);
                self.paint.tool = PaintTool::Off;
                self.s.surface_view = false;
            }
            let shrink = ctx.input(|i| i.key_pressed(egui::Key::OpenBracket));
            let grow = ctx.input(|i| i.key_pressed(egui::Key::CloseBracket));
            if shrink || grow {
                self.paint.radius =
                    (self.paint.radius * if grow { 1.2 } else { 1.0 / 1.2 }).clamp(0.0216, 0.576);
            }
        }
        if !ctx.wants_keyboard_input() {
            let undo = ctx.input(|i| {
                i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::Z)
            });
            let redo = ctx.input(|i| {
                i.modifiers.command
                    && (i.key_pressed(egui::Key::Y)
                        || (i.modifiers.shift && i.key_pressed(egui::Key::Z)))
            });
            if undo {
                self.paint.undo(&mut self.s.groom);
            }
            if redo {
                self.paint.redo(&mut self.s.groom);
            }
        }
        if !ctx.input(|i| i.pointer.primary_down()) {
            self.paint.end(&self.s.groom);
        }
        if self.paint.tool == PaintTool::Off {
            return false;
        }
        let navigating =
            ctx.input(|i| i.modifiers.alt || i.modifiers.shift || i.pointer.middle_down());
        if !response.hovered() || navigating {
            return false;
        }
        let Some(pointer) = ctx.input(|i| i.pointer.hover_pos()) else {
            return false;
        };
        let ndc = Vec3::new(
            (pointer.x - response.rect.left()) / response.rect.width() * 2.0 - 1.0,
            1.0 - (pointer.y - response.rect.top()) / response.rect.height() * 2.0,
            0.0,
        );
        let inverse = matrix.inverse();
        let a = inverse * ndc.extend(1.0);
        let b = inverse * Vec3::new(ndc.x, ndc.y, 1.0).extend(1.0);
        let origin = a.truncate() / a.w;
        let direction = (b.truncate() / b.w - origin).normalize_or_zero();
        let Some(hit) = human_mesh().hit(origin, direction, self.s.angle) else {
            return true;
        };
        let allowed = !self.paint.scalp_only || hit.scalp;
        let clip = matrix * hit.world.extend(1.0);
        let radius = (self.paint.radius * response.rect.height()
            / (self.s.fov.to_radians() * 0.5).tan()
            / clip.w.max(0.01)
            * 0.5)
            .clamp(3.0, 300.0);
        let painter = ui.painter_at(response.rect);
        let color = if allowed {
            COPPER
        } else {
            egui::Color32::from_rgb(220, 90, 90)
        };
        painter.circle_stroke(pointer, radius, egui::Stroke::new(1.6_f32, color));
        painter.circle_stroke(
            pointer,
            radius * (1.0 - self.paint.softness),
            egui::Stroke::new(0.8_f32, color.gamma_multiply(0.45)),
        );
        if self.paint.show_roots {
            for id in 0..self.s.guides as usize {
                let (p, _, _) = self.s.groom.surface(id);
                if p.distance(hit.position) > self.paint.radius {
                    continue;
                }
                let f = self.s.groom.attachment(id);
                let tri = human_mesh().triangles[f.triangle];
                let mut world = Vec3::ZERO;
                for (i, &vertex) in tri.iter().enumerate() {
                    world += rotate_cpu(
                        human_mesh().vertices[vertex],
                        self.s.angle * human_mesh().motion[vertex],
                    ) * f.barycentric[i];
                }
                let q = matrix * world.extend(1.0);
                if q.w <= 0.0 {
                    continue;
                }
                let xy = q.truncate() / q.w;
                let point = egui::pos2(
                    response.rect.left() + (xy.x * 0.5 + 0.5) * response.rect.width(),
                    response.rect.top() + (0.5 - xy.y * 0.5) * response.rect.height(),
                );
                let d = self.s.groom.field(id).density;
                painter.circle_filled(
                    point,
                    1.4,
                    if d > 0.1 {
                        egui::Color32::from_rgb(115, 211, 176)
                    } else {
                        egui::Color32::from_gray(110)
                    },
                );
            }
        }
        if allowed && ctx.input(|i| i.pointer.primary_down()) {
            self.paint.begin(&self.s.groom);
            let previous = self.paint.last_hit;
            let flow = previous
                .map(|p| hit.position - p.position)
                .unwrap_or(Vec3::ZERO);
            if previous.is_none_or(|p| p.position.distance(hit.position) > self.paint.radius * 0.08)
                || self.paint.tool != PaintTool::Comb
            {
                let steps = previous
                    .map(|p| {
                        (p.position.distance(hit.position) / (self.paint.radius * 0.25)).ceil()
                            as usize
                    })
                    .unwrap_or(1)
                    .clamp(1, 8);
                for step in 1..=steps {
                    let mut stamp = hit;
                    if let Some(p) = previous {
                        stamp.position = p.position.lerp(hit.position, step as f32 / steps as f32);
                    }
                    let selected_tool = self.paint.tool;
                    if ctx.input(|i| i.modifiers.ctrl) {
                        self.paint.tool = PaintTool::Erase;
                    }
                    self.paint.dab(
                        &mut self.s.groom,
                        stamp,
                        flow,
                        (self.s.style, self.s.length, self.s.fringe),
                        self.s.guides as usize,
                    );
                    self.paint.tool = selected_tool;
                }
                self.paint.last_hit = Some(hit);
            }
        }
        true
    }
}
fn paint_tool_name(tool: PaintTool) -> &'static str {
    match tool {
        PaintTool::Off => "Navigate",
        PaintTool::Density => "Grow hair",
        PaintTool::Erase => "Erase hair",
        PaintTool::Length => "Length",
        PaintTool::Comb => "Comb direction",
        PaintTool::Curl => "Curl",
        PaintTool::Clump => "Clumping",
        PaintTool::Width => "Thickness",
        PaintTool::Flyaway => "Flyaways",
        PaintTool::Color => "Color",
        PaintTool::Smooth => "Smooth",
    }
}
fn paint_color_name(id: i32) -> &'static str {
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
    ][id.clamp(0, 13) as usize]
}
#[derive(serde::Serialize, serde::Deserialize)]
struct GroomProject {
    version: u32,
    model: String,
    settings: Settings,
}
fn save_groom_file(path: &str, s: &Settings) -> Result<(), String> {
    let path = std::path::Path::new(path);
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let temporary = path.with_extension("mhsbgroom.tmp");
    let file = GroomProject {
        version: 2,
        model: MODEL_ID.into(),
        settings: s.clone(),
    };
    std::fs::write(
        &temporary,
        serde_json::to_vec_pretty(&file).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::rename(temporary, path).map_err(|e| e.to_string())
}
fn load_groom_file(path: &str) -> Result<GroomProject, String> {
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 32 * 1024 * 1024 {
        return Err("Groom file exceeds 32 MiB".into());
    }
    let mut file: GroomProject =
        serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    if file.model != MODEL_ID
        || ![1, 2].contains(&file.version)
        || file.settings.groom.model != MODEL_ID
        || ![1, 2].contains(&file.settings.groom.version)
        || file.settings.groom.layers.is_empty()
        || file.settings.groom.layers.len() > 8
        || file
            .settings
            .groom
            .placements
            .iter()
            .any(|(&id, r)| id >= ROOT_CAPACITY || !r.valid())
    {
        return Err("This groom uses a different model or unsupported version".into());
    }
    file.settings.validate();
    Ok(file)
}
fn apply_groom_project(s: &mut Settings, file: GroomProject) {
    let g = file.settings;
    s.groom = g.groom;
    s.style = g.style;
    s.length = g.length;
    s.curl_radius = g.curl_radius;
    s.curl_tightness = g.curl_tightness;
    s.hair_volume = g.hair_volume;
    s.fringe = g.fringe;
    s.color = g.color;
    s.roughness = g.roughness;
    s.diameter = g.diameter;
    s.flyaways = g.flyaways;
    s.clumping = g.clumping;
    s.clump_width = g.clump_width;
    s.root_density = g.root_density;
    s.memory = g.memory;
}
