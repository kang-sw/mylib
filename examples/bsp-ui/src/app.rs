mod boids;

use egui::RichText;

#[derive(Default)]
pub struct TemplateApp {
    // Example stuff:
    model: boids::Model,
    render_opt: boids::RenderOption,
    spawning_predator: bool,
    once: std::sync::OnceLock<()>,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.once.get_or_init(|| {
            let screen_size = ctx.available_rect().size();
            self.render_opt.offset = [screen_size.x / 2.0, screen_size.y / 2.0];
        });

        ctx.request_repaint();

        // Simulate boids
        self.model.tick();

        egui::CentralPanel::default().show(ctx, |ui| {
            // Render boids / grids
            let resp = self.model.draw(ui, &self.render_opt);

            if resp.dragged_by(egui::PointerButton::Secondary)
                || resp.clicked_by(egui::PointerButton::Secondary)
            {
                let was_click = resp.clicked_by(egui::PointerButton::Secondary);

                let pos = ctx.input(|i| i.pointer.interact_pos().unwrap());
                let pos = boids::to_world(pos, self.render_opt.offset, self.render_opt.zoom);

                self.model.spawn_boids(
                    if self.spawning_predator { 1 } else { 10 } * if was_click { 10 } else { 1 },
                    pos,
                    self.spawning_predator,
                );
            }
            if resp.dragged_by(egui::PointerButton::Primary) {
                let delta = resp.drag_delta();
                self.render_opt.offset[0] += delta.x;
                self.render_opt.offset[1] += delta.y;
            }

            let mut new_zoom = if resp.hovered() {
                let zoom = ctx.input(|i| i.zoom_delta());
                (zoom != 1.).then_some(self.render_opt.zoom * zoom)
            } else {
                None
            };

            if new_zoom.is_none() {
                let delta = ctx.input(|i| i.raw_scroll_delta.y);
                if delta != 0.0 {
                    new_zoom = Some(self.render_opt.zoom.powf(1. + delta / 5000.0));
                }
            }

            if let Some(zoom) = new_zoom {
                let cursor_pos = ctx.input(|i| i.pointer.interact_pos().unwrap_or_default());

                // Cursor position's world position remain same before/after zooming
                let before_world =
                    boids::to_world(cursor_pos, self.render_opt.offset, self.render_opt.zoom);
                let after_world = boids::to_world(cursor_pos, self.render_opt.offset, zoom);

                self.render_opt.offset[0] -= (before_world[0] - after_world[0]) * zoom;
                self.render_opt.offset[1] -= (before_world[1] - after_world[1]) * zoom;
                self.render_opt.zoom = zoom;
            }
        });

        egui::Window::new("Boid").show(ctx, |ui| {
            ui.input(|i| {
                if i.key_pressed(egui::Key::Tab) {
                    self.spawning_predator = !self.spawning_predator;
                } else if i.key_pressed(egui::Key::Space) {
                    self.model.enable_tick = !self.model.enable_tick;
                }
            });

            if ui
                .selectable_label(
                    self.spawning_predator,
                    format!(
                        "Spawning {}",
                        if self.spawning_predator {
                            "Predetor"
                        } else {
                            "Boid"
                        }
                    ),
                )
                .clicked()
            {
                self.spawning_predator = !self.spawning_predator;
            }

            egui::CollapsingHeader::new("Stats")
                .default_open(true)
                .show(ui, |ui| {
                    let stat = self.model.stats().back().unwrap();
                    let label_value_pairs = [
                        ("Count", stat.elem_count.to_string()),
                        ("Tick", format!("{:_>7.3} ms", stat.tick * 1000.)),
                        ("avg.Query", format!("{:_>7.3} µs", stat.avg_query * 1e6)),
                        ("avg.Step", format!("{:_>7.3} µs", stat.avg_step * 1e6)),
                        ("Optimize", format!("{:_>7.3} µs", stat.optimize * 1e6)),
                        ("Verify", format!("{:_>7.3} µs", stat.verify * 1e6)),
                    ];

                    for (label, value) in label_value_pairs.iter() {
                        ui.columns(2, |cols| {
                            cols[0].label(*label);
                            cols[1].monospace(RichText::new(value).color(egui::Color32::WHITE));
                        });
                    }
                });

            egui::CollapsingHeader::new("Boids")
                .default_open(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut self.model.enable_tick, "Enable Simulation?");

                    let mut speed = self.model.tick_delta * 60.0;
                    let label_param_pairs = [
                        ("Simulation Speed", &mut speed),
                        ("Max Speed", &mut self.model.max_speed),
                        ("Predator Avoidance", &mut self.model.predator_avoidance),
                        ("Area Radius", &mut self.model.area_radius),
                        ("View Radius", &mut self.model.view_radius),
                        ("Near Radius", &mut self.model.near_radius),
                        ("Cohesion Force", &mut self.model.cohesion_force),
                        ("Align Force", &mut self.model.align_force),
                        ("Separation Force", &mut self.model.separation_force),
                    ];

                    for (label, param) in label_param_pairs {
                        ui.columns(2, |cols| {
                            cols[0].label(label);
                            cols[1].add(
                                egui::DragValue::new(param)
                                    .speed(0.01)
                                    .clamp_range(0.01..=1e3),
                            );
                        });
                    }

                    self.model.tick_delta = speed / 60.0;
                });

            egui::CollapsingHeader::new("Bsp")
                .default_open(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut self.render_opt.draw_grid, "Draw Grid?");
                });

            guidance(ui);
        });
    }
}

fn guidance(ui: &mut egui::Ui) {
    ui.separator();

    ui.label("Right click and drag to spawn boids.");
    ui.label("Tap to switch boid type.");
    ui.label("Left click and drag to pan.");
    ui.label("Wheel, or control + wheel to zoom.");

    ui.separator();

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Author: ");
        ui.hyperlink_to("kang-sw", "https://github.com/kang-sw");
        ui.label("  (");
        ui.hyperlink_to(
            "source code",
            "https://github.com/kang-sw/mylib/tree/master/examples/bsp-ui",
        );
        ui.label(")");
    });
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
