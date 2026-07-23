use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};

use crate::app::App;
use crate::data::{BONUSES, SUMMONS};

pub const WINDOW_ICON: &[u8] = include_bytes!("../assets/icon_256.rgba");

const ACCENT: Color32 = Color32::from_rgb(0x8b, 0x7c, 0xf0);
const MUTED: Color32 = Color32::from_rgb(0x8b, 0x92, 0xa0);
const QUEST: Color32 = Color32::from_rgb(0x6f, 0xd6, 0xc4);
const CARD: Color32 = Color32::from_rgb(0x1e, 0x20, 0x2a);
const LINE: Color32 = Color32::from_rgb(0x2d, 0x32, 0x41);

enum Act {
    None,
    Apply,
    Run,
    Save,
    Recheck,
    Restore,
}

pub fn setup_style(ctx: &egui::Context) {
    let corner = CornerRadius::same(7);
    ctx.set_theme(egui::ThemePreference::Dark);
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(14.0, 7.0);
        style.spacing.interact_size.y = 30.0;
        style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::new(21.0, egui::FontFamily::Proportional));
        style.text_styles.insert(egui::TextStyle::Body, egui::FontId::new(14.5, egui::FontFamily::Proportional));
        style.text_styles.insert(egui::TextStyle::Button, egui::FontId::new(14.5, egui::FontFamily::Proportional));

        let text = Color32::from_rgb(0xdf, 0xe3, 0xea);
        let mut v = egui::Visuals::dark();
        v.panel_fill = Color32::from_rgb(0x14, 0x15, 0x1c);
        v.window_fill = v.panel_fill;
        v.override_text_color = Some(text);
        v.faint_bg_color = CARD;
        v.extreme_bg_color = Color32::from_rgb(0x0e, 0x0f, 0x15);
        v.window_corner_radius = CornerRadius::same(10);
        v.selection.bg_fill = Color32::from_rgba_unmultiplied(0x8b, 0x7c, 0xf0, 96);
        v.selection.stroke = Stroke::new(1.0, ACCENT);
        v.hyperlink_color = ACCENT;

        for (w, fill) in [
            (&mut v.widgets.noninteractive, CARD),
            (&mut v.widgets.inactive, Color32::from_rgb(0x28, 0x2c, 0x38)),
            (&mut v.widgets.hovered, Color32::from_rgb(0x32, 0x37, 0x46)),
            (&mut v.widgets.active, Color32::from_rgb(0x3b, 0x41, 0x53)),
            (&mut v.widgets.open, Color32::from_rgb(0x32, 0x37, 0x46)),
        ] {
            w.bg_fill = fill;
            w.weak_bg_fill = fill;
            w.bg_stroke = Stroke::new(1.0, LINE);
            w.corner_radius = corner;
            w.fg_stroke = Stroke::new(1.0, text);
        }
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0x6b, 0x5f, 0xc0));
        v.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
        v.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        style.visuals = v;
    });
}

fn card(ui: &egui::Ui, fill: Color32) -> egui::Frame {
    egui::Frame::group(ui.style())
        .fill(fill)
        .stroke(Stroke::new(1.0, LINE))
        .corner_radius(CornerRadius::same(9))
        .inner_margin(Margin::same(11))
}

fn chip(ui: &mut egui::Ui, fill: Color32, text_color: Color32, label: &str) {
    egui::Frame::group(ui.style())
        .fill(fill)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(label).size(11.0).color(text_color).strong());
        });
}

fn banner(app: &App) -> (Color32, String) {
    if !app.dep.located {
        (Color32::from_rgb(0x40, 0x36, 0x1c), "Could not find Reloaded-II's Mods folder to check requirements.".into())
    } else if app.dep.ok {
        (Color32::from_rgb(0x17, 0x32, 0x26), "Loader ready, gbfrelink.utility.manager and its dependencies are installed.".into())
    } else {
        let missing: Vec<_> = app.dep.items.iter().filter(|(_, ok)| !ok).map(|(n, _)| n.as_str()).collect();
        (Color32::from_rgb(0x44, 0x22, 0x24), format!("Missing: {}. Install it or the mod will not load.", missing.join(", ")))
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut act = Act::None;

        if self.logo.is_none() {
            let img = egui::ColorImage::from_rgba_unmultiplied([256, 256], WINDOW_ICON);
            self.logo = Some(ui.ctx().load_texture("logo", img, egui::TextureOptions::LINEAR));
        }

        // eframe hands us the bare window ui, no margin on it, so add one here
        egui::Frame::group(ui.style())
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE)
            .inner_margin(Margin::symmetric(14, 10))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if let Some(t) = &self.logo {
                            ui.add(egui::Image::new(egui::load::SizedTexture::new(t.id(), egui::vec2(48.0, 48.0))));
                        }
                        ui.add_space(4.0);
                        ui.vertical(|ui| {
                            ui.heading("GBFRER Summon Drop Picker");
                            ui.label(RichText::new("Every summon below is forced at once, each from its own quest.").color(MUTED));
                        });
                    });
                    ui.add_space(10.0);

                    let (fill, msg) = banner(self);
                    card(ui, fill).corner_radius(CornerRadius::same(8)).inner_margin(Margin::same(10)).show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| ui.label(RichText::new(msg).strong()));
                        ui.collapsing("Requirement details", |ui| {
                            for (name, ok) in &self.dep.items {
                                ui.horizontal(|ui| {
                                    let (c, label) = if *ok {
                                        (Color32::from_rgb(0x7b, 0xd6, 0x9a), "installed")
                                    } else {
                                        (Color32::from_rgb(0xe8, 0x92, 0x92), "MISSING ")
                                    };
                                    ui.label(RichText::new(label).color(c).strong());
                                    ui.label(RichText::new(name).monospace().color(MUTED));
                                });
                            }
                            if ui.button("Re-check").clicked() {
                                act = Act::Recheck;
                            }
                        });
                    });
                    ui.add_space(12.0);

                    for (i, s) in SUMMONS.iter().enumerate() {
                        card(ui, CARD).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(s.name).size(16.0).strong());
                                ui.label(RichText::new(format!("  {}", s.quest)).size(12.5).color(QUEST))
                                    .on_hover_text("recommended quest to farm this summon");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if s.astral {
                                        chip(ui, Color32::from_rgb(0x3d, 0x33, 0x5e), Color32::from_rgb(0xcf, 0xc3, 0xf5), "ASTRAL");
                                    } else {
                                        chip(ui, Color32::from_rgb(0x33, 0x3b, 0x4d), Color32::from_rgb(0xba, 0xc4, 0xd7), "NORMAL");
                                    }
                                });
                            });
                            ui.add_space(6.0);

                            egui::Grid::new(("summon", i)).num_columns(2).spacing([12.0, 8.0]).min_col_width(48.0).show(ui, |ui| {
                                ui.label(RichText::new("Skill").color(MUTED));
                                if s.skills.len() > 1 {
                                    egui::ComboBox::from_id_salt(("skill", i))
                                        .width(320.0)
                                        .selected_text(s.skills[self.picks[i].skill].0)
                                        .show_ui(ui, |ui| {
                                            for (k, sk) in s.skills.iter().enumerate() {
                                                ui.selectable_value(&mut self.picks[i].skill, k, sk.0);
                                            }
                                        });
                                } else {
                                    ui.label(RichText::new(s.skills[0].0).strong());
                                }
                                ui.end_row();

                                ui.label(RichText::new("Bonus").color(MUTED));
                                let half = self.picks[i].half_cap;
                                let selected = &BONUSES[self.picks[i].bonus];
                                egui::ComboBox::from_id_salt(("bonus", i))
                                    .width(320.0)
                                    .selected_text(format!("{}  {}", selected.name, selected.max(s.astral, half)))
                                    .show_ui(ui, |ui| {
                                        for (k, b) in BONUSES.iter().enumerate() {
                                            ui.selectable_value(&mut self.picks[i].bonus, k, format!("{}  {}", b.name, b.max(s.astral, half)));
                                        }
                                    });
                                ui.end_row();

                                if s.astral {
                                    ui.label(RichText::new("Cap").color(MUTED));
                                    ui.horizontal(|ui| {
                                        ui.selectable_value(&mut self.picks[i].half_cap, false, "Astral max");
                                        ui.selectable_value(&mut self.picks[i].half_cap, true, "Half (normal tier)");
                                    });
                                    ui.end_row();
                                }
                            });
                        });
                        ui.add_space(8.0);
                    }

                    ui.collapsing("Reloaded-II / game paths (auto-detected)", |ui| {
                        egui::Grid::new("paths").num_columns(3).spacing([8.0, 8.0]).show(ui, |ui| {
                            ui.label("Reloaded-II.exe");
                            ui.add(egui::TextEdit::singleline(&mut self.reloaded_path).desired_width(320.0));
                            if ui.button("Browse").clicked() {
                                if let Some(p) = rfd::FileDialog::new().add_filter("exe", &["exe"]).pick_file() {
                                    self.reloaded_path = p.display().to_string();
                                    act = Act::Save;
                                }
                            }
                            ui.end_row();

                            ui.label("Game .exe");
                            ui.add(egui::TextEdit::singleline(&mut self.game_path).desired_width(320.0));
                            if ui.button("Browse").clicked() {
                                if let Some(p) = rfd::FileDialog::new().add_filter("exe", &["exe"]).pick_file() {
                                    self.game_path = p.display().to_string();
                                    act = Act::Save;
                                }
                            }
                            ui.end_row();
                        });
                        ui.label(RichText::new("Leave the game path empty to just open Reloaded-II.").color(MUTED));
                    });

                    ui.collapsing("Removing the mod", |ui| {
                        ui.label(RichText::new("Disabling the mod can leave the forced drops in place if the loader already deployed the tables. Restore the vanilla tables, launch the game once with the mod still enabled, then disable or delete it.").color(MUTED));
                        ui.add_space(4.0);
                        if ui.button("Restore vanilla tables").clicked() {
                            act = Act::Restore;
                        }
                    });
                    ui.add_space(14.0);

                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new("Apply Picks").min_size(egui::vec2(150.0, 34.0))).clicked() {
                            act = Act::Apply;
                        }
                        let run = egui::Button::new(RichText::new("Apply & Run Game").color(Color32::WHITE).strong())
                            .fill(ACCENT)
                            .min_size(egui::vec2(220.0, 34.0));
                        if ui.add(run).clicked() {
                            act = Act::Run;
                        }
                    });
                    ui.add_space(10.0);

                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("Status:").color(MUTED));
                        ui.label(RichText::new(&self.status).strong());
                    });
                    ui.add_space(8.0);
                });
            });

        match act {
            Act::Apply => self.do_apply(),
            Act::Run => self.run_game(),
            Act::Save => self.save_config(),
            Act::Recheck => self.recheck_deps(),
            Act::Restore => self.restore_vanilla(),
            Act::None => {}
        }
    }
}
