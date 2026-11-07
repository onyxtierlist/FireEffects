use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveTool {
    Select,
    Hand,
    Text,
}

impl ActiveTool {
    fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Hand => "Hand",
            Self::Text => "Text",
        }
    }
}

pub struct FireEffectsApp {
    active_tool: ActiveTool,
    timeline_zoom: f32,
    track_count: usize,
    show_about: bool,
}

impl FireEffectsApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_theme(&cc.egui_ctx);

        Self {
            active_tool: ActiveTool::Select,
            timeline_zoom: 1.0,
            track_count: 3,
            show_about: false,
        }
    }
}

impl eframe::App for FireEffectsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw_menu_bar(ctx);
        self.draw_toolbar(ctx);
        self.draw_media_panel(ctx);
        self.draw_inspector(ctx);
        self.draw_timeline(ctx);
        self.draw_preview(ctx);

        if self.show_about {
            egui::Window::new("About FireEffects")
                .collapsible(false)
                .resizable(false)
                .default_width(320.0)
                .open(&mut self.show_about)
                .show(ctx, |ui| {
                    ui.heading("FireEffects");
                    ui.label("Version 0.1.0");
                    ui.add_space(6.0);
                    ui.label("An independent editor for video, compositing, and motion graphics.");
                    ui.add_space(8.0);
                    ui.label("Built with Rust and egui.");
                });
        }
    }
}

impl FireEffectsApp {
    fn draw_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar")
            .exact_height(34.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(egui::RichText::new("FireEffects").strong().color(egui::Color32::from_rgb(111, 170, 255)));
                    ui.separator();

                    ui.menu_button("File", |ui| {
                        ui.add_enabled(false, egui::Button::new("New Project"));
                        ui.add_enabled(false, egui::Button::new("Open Project…"));
                        ui.add_enabled(false, egui::Button::new("Save"));
                        ui.add_enabled(false, egui::Button::new("Save As…"));
                        ui.separator();
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });

                    ui.menu_button("Edit", |ui| {
                        ui.add_enabled(false, egui::Button::new("Undo"));
                        ui.add_enabled(false, egui::Button::new("Redo"));
                        ui.separator();
                        ui.add_enabled(false, egui::Button::new("Preferences"));
                    });

                    ui.menu_button("View", |ui| {
                        ui.label("Workspace panels are visible in this preview build.");
                    });

                    ui.menu_button("Help", |ui| {
                        if ui.button("About FireEffects").clicked() {
                            self.show_about = true;
                            ui.close_menu();
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("EARLY DEVELOPMENT BUILD").small().color(egui::Color32::from_rgb(150, 160, 178)));
                    });
                });
            });
    }

    fn draw_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar")
            .exact_height(42.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(egui::RichText::new("TOOLS").small().weak());
                    ui.separator();

                    for tool in [ActiveTool::Select, ActiveTool::Hand, ActiveTool::Text] {
                        if ui.selectable_label(self.active_tool == tool, tool.label()).clicked() {
                            self.active_tool = tool;
                        }
                    }

                    ui.separator();
                    ui.add_enabled(false, egui::Button::new("Import Media"));
                    ui.add_enabled(false, egui::Button::new("Export"));
                    ui.separator();
                    ui.label(format!("Active tool: {}", self.active_tool.label()));
                });
            });
    }

    fn draw_media_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("media_panel")
            .default_width(218.0)
            .min_width(175.0)
            .resizable(true)
            .show(ctx, |ui| {
                panel_heading(ui, "PROJECT");
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Untitled Project").strong());
                ui.label(egui::RichText::new("No media imported").small().weak());
                ui.add_space(12.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("MEDIA").strong().small());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_enabled(false, egui::Button::new("+"));
                    });
                });
                ui.add_space(6.0);
                ui.vertical_centered(|ui| {
                    ui.add_space(22.0);
                    ui.label(egui::RichText::new("▧").size(28.0).color(egui::Color32::from_rgb(105, 120, 145)));
                    ui.label(egui::RichText::new("Your media will appear here").small().weak());
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("Media importing is coming in Stage 2").small().weak());
                });
            });
    }

    fn draw_inspector(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("inspector")
            .default_width(250.0)
            .min_width(205.0)
            .resizable(true)
            .show(ctx, |ui| {
                panel_heading(ui, "INSPECTOR");
                ui.add_space(12.0);
                ui.label(egui::RichText::new("Nothing selected").strong());
                ui.label(egui::RichText::new("Select a clip or layer to inspect its properties.").small().weak());
                ui.add_space(14.0);
                ui.separator();
                ui.label(egui::RichText::new("TRANSFORM").small().strong().weak());
                ui.add_space(6.0);
                for property in ["Position", "Scale", "Rotation", "Opacity"] {
                    ui.horizontal(|ui| {
                        ui.label(property);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new("—").weak());
                        });
                    });
                }
                ui.add_space(12.0);
                ui.separator();
                ui.label(egui::RichText::new("EFFECTS").small().strong().weak());
                ui.label(egui::RichText::new("No effects on selected item").small().weak());
            });
    }

    fn draw_timeline(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("timeline")
            .default_height(238.0)
            .min_height(155.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    panel_heading(ui, "TIMELINE");
                    ui.separator();
                    ui.label(egui::RichText::new("00:00:00:00").monospace().color(egui::Color32::from_rgb(130, 185, 255)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(egui::Slider::new(&mut self.timeline_zoom, 0.5..=3.0).text("Zoom").show_value(false));
                        ui.label("Timeline zoom");
                    });
                });

                ui.add_space(6.0);
                ui.separator();
                egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                    ui.set_min_width(ui.available_width().max(500.0));
                    draw_time_ruler(ui, self.timeline_zoom);
                    ui.add_space(4.0);

                    for index in 0..self.track_count {
                        ui.horizontal(|ui| {
                            let track_name = format!("Track {}", index + 1);
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(108.0, 34.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(rect, 3.0, egui::Color32::from_rgb(42, 49, 64));
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                track_name,
                                egui::FontId::proportional(11.0),
                                egui::Color32::from_rgb(190, 199, 215),
                            );

                            let width = (ui.available_width() * self.timeline_zoom).max(320.0);
                            let (lane, _) = ui.allocate_exact_size(
                                egui::vec2(width, 34.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(lane, 3.0, egui::Color32::from_rgb(28, 34, 46));
                            ui.painter().rect_stroke(
                                lane,
                                3.0,
                                egui::Stroke::new(1.0, egui::Color32::from_rgb(48, 57, 73)),
                                egui::StrokeKind::Inside,
                            );
                        });
                        ui.add_space(3.0);
                    }
                });

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.button("+ Add Track").clicked() {
                        self.track_count = (self.track_count + 1).min(32);
                    }
                    ui.label(egui::RichText::new("Empty timeline · Editing tools arrive in Stage 3").small().weak());
                });
            });
    }

    fn draw_preview(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    panel_heading(ui, "PREVIEW");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("Viewer · No composition").small().weak());
                    });
                });
                ui.add_space(8.0);

                let available = ui.available_size();
                let (rect, _) = ui.allocate_exact_size(available, egui::Sense::hover());
                ui.painter().rect_filled(rect, 6.0, egui::Color32::from_rgb(12, 15, 22));

                let canvas_size = egui::vec2(
                    (rect.width() * 0.78).min(rect.height() * 16.0 / 9.0),
                    (rect.height() * 0.78).min(rect.width() * 9.0 / 16.0),
                );
                let canvas = egui::Rect::from_center_size(rect.center(), canvas_size);
                ui.painter().rect_filled(canvas, 2.0, egui::Color32::from_rgb(20, 25, 36));
                ui.painter().rect_stroke(
                    canvas,
                    2.0,
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(57, 69, 91)),
                    egui::StrokeKind::Inside,
                );

                ui.painter().text(
                    canvas.center() - egui::vec2(0.0, 12.0),
                    egui::Align2::CENTER_CENTER,
                    "FireEffects",
                    egui::FontId::proportional(28.0),
                    egui::Color32::from_rgb(205, 216, 235),
                );
                ui.painter().text(
                    canvas.center() + egui::vec2(0.0, 20.0),
                    egui::Align2::CENTER_CENTER,
                    "Your composition preview will appear here",
                    egui::FontId::proportional(12.0),
                    egui::Color32::from_rgb(119, 133, 156),
                );
            });
        });
    }
}

fn configure_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::from_rgb(25, 30, 41);
    visuals.window_fill = egui::Color32::from_rgb(30, 36, 49);
    visuals.extreme_bg_color = egui::Color32::from_rgb(12, 15, 22);
    visuals.faint_bg_color = egui::Color32::from_rgb(35, 42, 56);
    visuals.selection.bg_fill = egui::Color32::from_rgb(48, 91, 151);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(55, 105, 170);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(49, 61, 82);
    ctx.set_visuals(visuals);
}

fn panel_heading(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .size(11.0)
            .strong()
            .color(egui::Color32::from_rgb(151, 169, 195)),
    );
}

fn draw_time_ruler(ui: &mut egui::Ui, zoom: f32) {
    let width = (ui.available_width() * zoom).max(320.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 22.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 2.0, egui::Color32::from_rgb(32, 39, 52));

    for second in 0..=10 {
        let x = rect.left() + rect.width() * second as f32 / 10.0;
        ui.painter().line_segment(
            [egui::pos2(x, rect.bottom() - 7.0), egui::pos2(x, rect.bottom())],
            egui::Stroke::new(1.0, egui::Color32::from_rgb(104, 119, 143)),
        );
        if second % 2 == 0 {
            ui.painter().text(
                egui::pos2(x + 4.0, rect.top() + 2.0),
                egui::Align2::LEFT_TOP,
                format!("{second}s"),
                egui::FontId::proportional(10.0),
                egui::Color32::from_rgb(143, 157, 180),
            );
        }
    }
}
