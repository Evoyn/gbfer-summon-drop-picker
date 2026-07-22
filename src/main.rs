// GBFRER Summon Drop Picker - native GUI (egui/eframe)
// Fully offline: no network calls anywhere. Edits this mod's own summon_lot.tbl
// in place (same byte-patch the PowerShell picker uses) and can launch the game
// through Reloaded-II.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::fs;
use std::path::{Path, PathBuf};

// ---------- embedded pristine (split-pool) base tables ----------
const BASE_SUMMON_LOT: &[u8] = include_bytes!("../base/summon_lot.tbl");
const BASE_SUMMON:     &[u8] = include_bytes!("../base/summon.tbl");
const BASE_REWARD:     &[u8] = include_bytes!("../base/reward_summon_lot.tbl");
const BASE_CURVE:      &[u8] = include_bytes!("../base/summon_curve.tbl");
const MODCONFIG_JSON:  &str = include_str!("../dist/ModConfig.json");   // so a bare exe can install itself
const WINDOW_ICON:     &[u8] = include_bytes!("../icon_256.rgba");      // 256x256 RGBA window/taskbar icon

// ---------- data ----------
struct SummonDef {
    name: &'static str,
    normal: bool,                 // true = normal tier (Behemoth / Wee Pincer), false = astral
    skill_pool: &'static str,
    equip_pool: &'static str,
    skills: &'static [(&'static str, &'static str)], // (display name, raw hash id)
}

// 11 equip bonuses: (key, name, normal_id, normal_max, astral_id, astral_max)
const BONUSES: &[(&str, &str, &str, &str, &str, &str)] = &[
    ("normatkcap", "Normal Attack Damage Cap Up", "A66241C9", "50%",   "9245DFA4", "100%"),
    ("skilldmgcap","Skill Damage Cap Up",         "2FFB509F", "50%",   "CE70C58A", "100%"),
    ("skyartcap",  "Skybound Art Damage Cap Up",  "A3539FBB", "50%",   "5A1D2C89", "100%"),
    ("healcap",    "Healing Cap Up",              "2270BC40", "50%",   "2EA9CA80", "75%"),
    ("stun",       "Stun Power Up",               "F7B0316F", "15",    "F0F77BC1", "20"),
    ("atk",        "Attack Power Up",             "A8900C80", "+2000", "A3E537B1", "+3000"),
    ("hp",         "Health Up",                   "BC4E92CB", "+2000", "F2256E44", "+5000"),
    ("crit",       "Critical Hit Rate Up",        "00D171E0", "20%",   "A074A967", "30%"),
    ("skilldmg",   "Skill Damage Up",             "664E8E32", "20%",   "AF39A3FB", "30%"),
    ("skyart",     "Skybound Art Damage Up",      "75138D21", "20%",   "33C0D50C", "30%"),
    ("chainburst", "Chain Burst Damage Up",       "5A39D81B", "50%",   "54B09A37", "100%"),
];

const SUMMONS: &[SummonDef] = &[
    SummonDef { name: "Behemoth III", normal: true, skill_pool: "DF902143", equip_pool: "393EF1D8", skills: &[
        ("Stout Heart", "A1A8E39D"), ("Supplementary DMG", "57AB5B10"), ("Uplift", "B5FF9FD3"),
        ("Celestial Ventus", "73220725"), ("Less Is More", "82CE278D"), ("Critical Hit Rate", "8D78A19B") ] },
    SummonDef { name: "Wee Pincer III", normal: true, skill_pool: "2340DCED", equip_pool: "C39F7144", skills: &[
        ("Crabvestment Returns", "1B0D9897"), ("DMG Cap", "DC584F60"), ("Cascade", "05F2ECDC"), ("Steel Nerves", "1470F860") ] },
    SummonDef { name: "Lucilius", normal: false, skill_pool: "CC0B0EF1", equip_pool: "F63793E4", skills: &[
        ("Alpha", "DBE1D775"), ("Beta", "8D2ADB6E"), ("Gamma", "5C862E13"), ("Berserker Echo", "EE85CD1F"),
        ("Tyranny", "71F11A9B"), ("Celestial Terra", "9232DC17") ] },
    SummonDef { name: "Beelzebub", normal: false, skill_pool: "BD452CC9", equip_pool: "B7D9379B", skills: &[
        ("Spartan Echo", "3D8153A1"), ("Supplementary DMG", "57AB5B10"), ("DMG Cap", "DC584F60"),
        ("Drain", "7CCFF74F"), ("Celestial Lumen", "A7726190"), ("Improved Guard", "0AA20846") ] },
    SummonDef { name: "Rolan", normal: false, skill_pool: "26428274", equip_pool: "A35D7A5C", skills: &[
        ("War Elemental", "4C588C27"), ("Uplift", "B5FF9FD3"), ("Autorevive", "95F3FA86"),
        ("Quick Cooldown", "318D12E9"), ("Drain", "7CCFF74F"), ("Aegis", "E0ABFDFE") ] },
    SummonDef { name: "Lilith", normal: false, skill_pool: "05205336", equip_pool: "DCB2B22B", skills: &[
        ("War Elemental", "4C588C27"), ("Uplift", "B5FF9FD3"), ("Potion Hoarder", "24883AF3"),
        ("Tyranny", "71F11A9B"), ("Linked Together", "3FEC5F80"), ("Improved Healing", "9389CC06") ] },
];

// ---------- the byte patcher (identical rule to the PowerShell picker) ----------
// summon_lot.tbl: 8-byte header (int32 rowCount) then 20-byte rows:
//   +0 Key  +4 SkillId/BaseParamId  +8 CurveId  +12 Weight(i32)  +16 Unk5   (LE)
fn force_pool(bytes: &mut [u8], pool_hex: &str, keep_hex: &str) {
    let pool = u32::from_str_radix(pool_hex, 16).unwrap_or(0);
    let keep = u32::from_str_radix(keep_hex, 16).unwrap_or(0);
    let n = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    for r in 0..n {
        let off = 8 + r * 20;
        if off + 20 > bytes.len() { break; }
        let key = u32::from_le_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]]);
        if key == pool {
            let sid = u32::from_le_bytes([bytes[off + 4], bytes[off + 5], bytes[off + 6], bytes[off + 7]]);
            if sid != keep {
                bytes[off + 12..off + 16].copy_from_slice(&1i32.to_le_bytes());
            }
        }
    }
}

fn patched_summon_lot(picks: &[(usize, usize)]) -> Vec<u8> {
    let mut lot = BASE_SUMMON_LOT.to_vec();
    for (i, s) in SUMMONS.iter().enumerate() {
        let (skill_idx, bonus_idx) = picks[i];
        force_pool(&mut lot, s.skill_pool, s.skills[skill_idx].1);
        let b = &BONUSES[bonus_idx];
        let bonus_id = if s.normal { b.2 } else { b.4 };
        force_pool(&mut lot, s.equip_pool, bonus_id);
    }
    lot
}

// ---------- app ----------
struct DepReport {
    ok: bool,
    located: bool,                 // could we find Reloaded's Mods folder at all?
    items: Vec<(String, bool)>,    // (mod id, installed)
}

struct App {
    picks: Vec<(usize, usize)>, // (skill_idx, bonus_idx) per summon
    reloaded_path: String,
    game_path: String,
    status: String,
    exe_dir: PathBuf,
    dep: DepReport,
    logo: Option<egui::TextureHandle>,
}

impl App {
    fn new() -> Self {
        let exe_dir = std::env::current_exe().ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        let mut picks = vec![(0usize, 0usize); SUMMONS.len()];
        let (mut reloaded_path, mut game_path) = autodetect();

        // load saved settings next to the exe (overrides autodetect where present)
        let cfg = exe_dir.join("picker_settings.json");
        if let Ok(txt) = fs::read_to_string(&cfg) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(txt.trim_start_matches('\u{feff}')) {
                if let Some(p) = v.get("reloaded_path").and_then(|x| x.as_str()) { if !p.is_empty() { reloaded_path = p.to_string(); } }
                if let Some(p) = v.get("game_path").and_then(|x| x.as_str()) { if !p.is_empty() { game_path = p.to_string(); } }
                if let Some(arr) = v.get("picks").and_then(|x| x.as_array()) {
                    for (i, s) in SUMMONS.iter().enumerate() {
                        if let Some(e) = arr.get(i) {
                            if let Some(sk) = e.get("skill").and_then(|x| x.as_str()) {
                                if let Some(k) = s.skills.iter().position(|x| x.0 == sk) { picks[i].0 = k; }
                            }
                            if let Some(bk) = e.get("bonus").and_then(|x| x.as_str()) {
                                if let Some(k) = BONUSES.iter().position(|x| x.0 == bk) { picks[i].1 = k; }
                            }
                        }
                    }
                }
            }
        }
        ensure_installed(&exe_dir, &picks);   // make a bare exe a valid, working mod on first run
        Self { picks, reloaded_path, game_path, status: "Ready. Set each summon's skill + bonus, then Apply.".into(), exe_dir, dep: check_deps(), logo: None }
    }

    fn save_config(&self) {
        let picks: Vec<serde_json::Value> = SUMMONS.iter().enumerate().map(|(i, s)| {
            serde_json::json!({ "summon": s.name, "skill": s.skills[self.picks[i].0].0, "bonus": BONUSES[self.picks[i].1].0 })
        }).collect();
        let v = serde_json::json!({ "reloaded_path": self.reloaded_path, "game_path": self.game_path, "picks": picks });
        let _ = fs::write(self.exe_dir.join("picker_settings.json"), serde_json::to_string_pretty(&v).unwrap_or_default());
    }

    fn apply(&mut self) -> std::io::Result<()> {
        let dir = self.exe_dir.join("GBFR").join("data").join("system").join("table");
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("summon_lot.tbl"), patched_summon_lot(&self.picks))?;
        fs::write(dir.join("summon.tbl"), BASE_SUMMON)?;
        fs::write(dir.join("reward_summon_lot.tbl"), BASE_REWARD)?;
        fs::write(dir.join("summon_curve.tbl"), BASE_CURVE)?;
        self.save_config();
        Ok(())
    }

    fn do_apply(&mut self) {
        match self.apply() {
            Ok(()) => self.status = "Applied. Enable ONLY this mod in Reloaded-II, then (re)launch the game.".into(),
            Err(e) => self.status = format!("Could not write tables: {e}. Is the game running / folder read-only?"),
        }
    }

    fn run_game(&mut self) {
        if let Err(e) = self.apply() {
            self.status = format!("Could not write tables: {e}");
            return;
        }
        let reloaded_ok = !self.reloaded_path.is_empty() && Path::new(&self.reloaded_path).exists();
        if !reloaded_ok {
            self.status = "Picks applied & saved. Reloaded-II.exe not found - set its path below (Browse), or just launch the game through Reloaded-II yourself.".into();
            return;
        }
        let mut cmd = std::process::Command::new(&self.reloaded_path);
        if !self.game_path.is_empty() && Path::new(&self.game_path).exists() {
            cmd.arg("--launch").arg(&self.game_path);
        }
        match cmd.spawn() {
            Ok(_) => {
                self.status = if self.game_path.is_empty() {
                    "Applied. Opened Reloaded-II - press Launch there.".into()
                } else {
                    "Applied & launching the game through Reloaded-II...".into()
                };
            }
            Err(e) => self.status = format!("Applied, but couldn't start Reloaded-II: {e}. Launch it yourself."),
        }
    }
}

// Reloaded's Mods folder (from its own ReloadedII.json), so we can check deps.
fn mods_dir() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    let cfg = Path::new(&appdata).join("Reloaded-Mod-Loader-II").join("ReloadedII.json");
    let txt = fs::read_to_string(&cfg).ok()?;
    let v: serde_json::Value = serde_json::from_str(txt.trim_start_matches('\u{feff}')).ok()?;
    v.get("ModConfigDirectory").and_then(|x| x.as_str()).map(PathBuf::from)
}

// Check that the required Reloaded mod (gbfrelink.utility.manager) AND its own
// dependency chain are installed. Those 3 (Reloaded.Memory.SigScan.ReloadedII,
// reloaded.sharedlib.hooks, reloaded.universal.redirector) belong to the loader,
// not to us - we only depend on gbfrelink.utility.manager, which pulls them in.
fn check_deps() -> DepReport {
    let mods = mods_dir();
    let has = |id: &str| -> bool {
        mods.as_ref().map(|m| m.join(id).join("ModConfig.json").exists()).unwrap_or(false)
    };
    let core = "gbfrelink.utility.manager";
    let mut items = vec![(core.to_string(), has(core))];
    if let Some(m) = &mods {
        if let Ok(txt) = fs::read_to_string(m.join(core).join("ModConfig.json")) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(txt.trim_start_matches('\u{feff}')) {
                if let Some(arr) = v.get("ModDependencies").and_then(|x| x.as_array()) {
                    for d in arr.iter().filter_map(|x| x.as_str()) {
                        items.push((d.to_string(), has(d)));
                    }
                }
            }
        }
    }
    DepReport { ok: items.iter().all(|(_, f)| *f), located: mods.is_some(), items }
}

// Make a bare exe a valid Reloaded mod: write ModConfig.json if it's missing,
// and materialize default tables so the mod works before the first Apply.
fn ensure_installed(exe_dir: &Path, picks: &[(usize, usize)]) {
    let mc = exe_dir.join("ModConfig.json");
    if !mc.exists() {
        let _ = fs::write(&mc, MODCONFIG_JSON);
    }
    let dir = exe_dir.join("GBFR").join("data").join("system").join("table");
    if !dir.join("summon_lot.tbl").exists() && fs::create_dir_all(&dir).is_ok() {
        let _ = fs::write(dir.join("summon_lot.tbl"), patched_summon_lot(picks));
        let _ = fs::write(dir.join("summon.tbl"), BASE_SUMMON);
        let _ = fs::write(dir.join("reward_summon_lot.tbl"), BASE_REWARD);
        let _ = fs::write(dir.join("summon_curve.tbl"), BASE_CURVE);
    }
}

fn autodetect() -> (String, String) {
    let mut reloaded = String::new();
    let mut game = String::new();
    if let Ok(appdata) = std::env::var("APPDATA") {
        let cfg = Path::new(&appdata).join("Reloaded-Mod-Loader-II").join("ReloadedII.json");
        if let Ok(txt) = fs::read_to_string(&cfg) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(txt.trim_start_matches('\u{feff}')) {
                if let Some(lp) = v.get("LauncherPath").and_then(|x| x.as_str()) { reloaded = lp.to_string(); }
                if let Some(acd) = v.get("ApplicationConfigDirectory").and_then(|x| x.as_str()) {
                    if let Ok(rd) = fs::read_dir(acd) {
                        for e in rd.flatten() {
                            let ac = e.path().join("AppConfig.json");
                            if let Ok(t) = fs::read_to_string(&ac) {
                                if let Ok(j) = serde_json::from_str::<serde_json::Value>(t.trim_start_matches('\u{feff}')) {
                                    let loc = j.get("AppLocation").and_then(|x| x.as_str()).unwrap_or("");
                                    let id = j.get("AppId").and_then(|x| x.as_str()).unwrap_or("");
                                    if id.to_lowercase().contains("granblue") || loc.to_lowercase().contains("granblue") {
                                        game = loc.to_string();
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if reloaded.is_empty() {
        for c in ["F:\\Reloaded-II\\Reloaded-II.exe", "C:\\Reloaded-II\\Reloaded-II.exe", "D:\\Reloaded-II\\Reloaded-II.exe"] {
            if Path::new(c).exists() { reloaded = c.to_string(); break; }
        }
    }
    (reloaded, game)
}

// palette (keyed to the crystal icon)
const ACCENT: egui::Color32 = egui::Color32::from_rgb(0x8b, 0x7c, 0xf0);
const MUTED:  egui::Color32 = egui::Color32::from_rgb(0x8b, 0x92, 0xa0);
const CARD:   egui::Color32 = egui::Color32::from_rgb(0x1e, 0x20, 0x2a);
const LINE:   egui::Color32 = egui::Color32::from_rgb(0x2d, 0x32, 0x41);

fn setup_style(ctx: &egui::Context) {
    use egui::{Color32, CornerRadius, Stroke};
    let cr = CornerRadius::same(7);
    ctx.set_theme(egui::ThemePreference::Dark);
    ctx.all_styles_mut(|style| {
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(14.0, 7.0);
    style.spacing.interact_size.y = 30.0;
    style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::new(21.0, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Body, egui::FontId::new(14.5, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Button, egui::FontId::new(14.5, egui::FontFamily::Proportional));

    let mut v = egui::Visuals::dark();
    v.panel_fill = Color32::from_rgb(0x14, 0x15, 0x1c);
    v.window_fill = v.panel_fill;
    v.override_text_color = Some(Color32::from_rgb(0xdf, 0xe3, 0xea));
    v.faint_bg_color = CARD;
    v.extreme_bg_color = Color32::from_rgb(0x0e, 0x0f, 0x15);
    v.window_corner_radius = CornerRadius::same(10);
    v.selection.bg_fill = Color32::from_rgba_unmultiplied(0x8b, 0x7c, 0xf0, 96);
    v.selection.stroke = Stroke::new(1.0, ACCENT);
    v.hyperlink_color = ACCENT;
    let widget = Color32::from_rgb(0x28, 0x2c, 0x38);
    let hover  = Color32::from_rgb(0x32, 0x37, 0x46);
    let active = Color32::from_rgb(0x3b, 0x41, 0x53);
    let text   = Color32::from_rgb(0xdf, 0xe3, 0xea);
    for (w, fill) in [
        (&mut v.widgets.noninteractive, CARD),
        (&mut v.widgets.inactive, widget),
        (&mut v.widgets.hovered, hover),
        (&mut v.widgets.active, active),
        (&mut v.widgets.open, hover),
    ] {
        w.bg_fill = fill;
        w.weak_bg_fill = fill;
        w.bg_stroke = Stroke::new(1.0, LINE);
        w.corner_radius = cr;
        w.fg_stroke = Stroke::new(1.0, text);
    }
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0x6b, 0x5f, 0xc0));
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    v.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    style.visuals = v;
    });
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        use egui::{Color32, CornerRadius, Margin, Stroke, RichText};
        enum Act { None, Apply, Run, Save, Recheck }
        let mut act = Act::None;

        if self.logo.is_none() {
            let img = egui::ColorImage::from_rgba_unmultiplied([256, 256], WINDOW_ICON);
            self.logo = Some(ui.ctx().load_texture("logo", img, egui::TextureOptions::LINEAR));
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(12.0);

            // header
            ui.horizontal(|ui| {
                if let Some(t) = &self.logo {
                    ui.add(egui::Image::new(egui::load::SizedTexture::new(t.id(), egui::vec2(48.0, 48.0))));
                }
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.heading("GBFRER Summon Drop Picker");
                    ui.label(RichText::new("Force all six Infinity-boss summons at once - set each one's skill & bonus.").color(MUTED));
                });
            });
            ui.add_space(10.0);

            // dependency banner
            let (fill, msg) = if !self.dep.located {
                (Color32::from_rgb(0x40, 0x36, 0x1c), "Couldn't find Reloaded-II's Mods folder to check requirements.".to_string())
            } else if self.dep.ok {
                (Color32::from_rgb(0x17, 0x32, 0x26), "Loader ready - gbfrelink.utility.manager and its dependencies are installed.".to_string())
            } else {
                let miss: Vec<_> = self.dep.items.iter().filter(|(_, f)| !*f).map(|(n, _)| n.clone()).collect();
                (Color32::from_rgb(0x44, 0x22, 0x24), format!("Missing: {}  -  install it or the mod won't load.", miss.join(", ")))
            };
            egui::Frame::group(ui.style()).fill(fill).stroke(Stroke::new(1.0, LINE)).corner_radius(CornerRadius::same(8)).inner_margin(Margin::same(10)).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| { ui.label(RichText::new(msg).strong()); });
                ui.collapsing("Requirement details", |ui| {
                    for (n, f) in &self.dep.items {
                        ui.horizontal(|ui| {
                            let (c, m) = if *f { (Color32::from_rgb(0x7b, 0xd6, 0x9a), "installed") } else { (Color32::from_rgb(0xe8, 0x92, 0x92), "MISSING ") };
                            ui.label(RichText::new(m).color(c).strong());
                            ui.label(RichText::new(n).monospace().color(MUTED));
                        });
                    }
                    if ui.button("Re-check").clicked() { act = Act::Recheck; }
                });
            });
            ui.add_space(12.0);

            // summon cards
            for (i, s) in SUMMONS.iter().enumerate() {
                egui::Frame::group(ui.style()).fill(CARD).stroke(Stroke::new(1.0, LINE)).corner_radius(CornerRadius::same(9)).inner_margin(Margin::same(11)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(s.name).size(16.0).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let (cf, ctxt, label) = if s.normal {
                                (Color32::from_rgb(0x33, 0x3b, 0x4d), Color32::from_rgb(0xba, 0xc4, 0xd7), "NORMAL")
                            } else {
                                (Color32::from_rgb(0x3d, 0x33, 0x5e), Color32::from_rgb(0xcf, 0xc3, 0xf5), "ASTRAL")
                            };
                            egui::Frame::group(ui.style()).fill(cf).stroke(Stroke::NONE).corner_radius(CornerRadius::same(6)).inner_margin(Margin::symmetric(8, 3)).show(ui, |ui| {
                                ui.label(RichText::new(label).size(11.0).color(ctxt).strong());
                            });
                        });
                    });
                    ui.add_space(6.0);
                    egui::Grid::new(("grid", i)).num_columns(2).spacing([12.0, 8.0]).min_col_width(48.0).show(ui, |ui| {
                        ui.label(RichText::new("Skill").color(MUTED));
                        if s.skills.len() > 1 {
                            let cur = s.skills[self.picks[i].0].0;
                            egui::ComboBox::from_id_salt(("skill", i)).width(320.0).selected_text(cur).show_ui(ui, |ui| {
                                for (k, sk) in s.skills.iter().enumerate() { ui.selectable_value(&mut self.picks[i].0, k, sk.0); }
                            });
                        } else {
                            ui.label(RichText::new(s.skills[0].0).strong());
                        }
                        ui.end_row();
                        ui.label(RichText::new("Bonus").color(MUTED));
                        let bi = self.picks[i].1;
                        let cur = format!("{}  {}", BONUSES[bi].1, if s.normal { BONUSES[bi].3 } else { BONUSES[bi].5 });
                        egui::ComboBox::from_id_salt(("bonus", i)).width(320.0).selected_text(cur).show_ui(ui, |ui| {
                            for (k, b) in BONUSES.iter().enumerate() {
                                let label = format!("{}  {}", b.1, if s.normal { b.3 } else { b.5 });
                                ui.selectable_value(&mut self.picks[i].1, k, label);
                            }
                        });
                        ui.end_row();
                    });
                });
                ui.add_space(8.0);
            }

            // settings
            ui.collapsing("Reloaded-II / game paths (auto-detected)", |ui| {
                egui::Grid::new("paths").num_columns(3).spacing([8.0, 8.0]).show(ui, |ui| {
                    ui.label("Reloaded-II.exe");
                    ui.add(egui::TextEdit::singleline(&mut self.reloaded_path).desired_width(320.0));
                    if ui.button("Browse").clicked() {
                        if let Some(p) = rfd::FileDialog::new().add_filter("exe", &["exe"]).pick_file() { self.reloaded_path = p.display().to_string(); act = Act::Save; }
                    }
                    ui.end_row();
                    ui.label("Game .exe");
                    ui.add(egui::TextEdit::singleline(&mut self.game_path).desired_width(320.0));
                    if ui.button("Browse").clicked() {
                        if let Some(p) = rfd::FileDialog::new().add_filter("exe", &["exe"]).pick_file() { self.game_path = p.display().to_string(); act = Act::Save; }
                    }
                    ui.end_row();
                });
                ui.label(RichText::new("Leave the game path empty to just open Reloaded-II and press Launch there.").color(MUTED));
            });
            ui.add_space(14.0);

            // action buttons
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("Apply Picks").min_size(egui::vec2(150.0, 34.0))).clicked() { act = Act::Apply; }
                let run = egui::Button::new(RichText::new("Apply & Run Game").color(Color32::WHITE).strong()).fill(ACCENT).min_size(egui::vec2(220.0, 34.0));
                if ui.add(run).clicked() { act = Act::Run; }
            });
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Status:").color(MUTED));
                ui.label(RichText::new(&self.status).strong());
            });
            ui.add_space(8.0);
        });

        match act {
            Act::Apply => self.do_apply(),
            Act::Run => self.run_game(),
            Act::Save => self.save_config(),
            Act::Recheck => { self.dep = check_deps(); self.status = "Re-checked requirements.".into(); }
            Act::None => {}
        }
    }
}

fn main() -> eframe::Result<()> {
    // headless apply mode (no window) - re-applies the saved picks. Handy for
    // automation and testing: `picker.exe --apply`.
    if std::env::args().any(|a| a == "--apply") {
        let mut app = App::new();
        std::process::exit(match app.apply() {
            Ok(()) => 0,
            Err(_) => 1,
        });
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 720.0])
            .with_min_inner_size([520.0, 400.0])
            .with_icon(std::sync::Arc::new(egui::IconData { rgba: WINDOW_ICON.to_vec(), width: 256, height: 256 }))
            .with_title("GBFRER Summon Drop Picker"),
        ..Default::default()
    };
    eframe::run_native("GBFRER Summon Drop Picker", options, Box::new(|cc| {
        setup_style(&cc.egui_ctx);
        Ok(Box::new(App::new()))
    }))
}

// ---------- tests ----------
#[cfg(test)]
mod tests {
    use super::*;

    fn pool_rows(bytes: &[u8], pool_hex: &str) -> Vec<(u32, i32)> {
        let pool = u32::from_str_radix(pool_hex, 16).unwrap();
        let n = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let mut out = vec![];
        for r in 0..n {
            let off = 8 + r * 20;
            let key = u32::from_le_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]]);
            if key == pool {
                let sid = u32::from_le_bytes([bytes[off + 4], bytes[off + 5], bytes[off + 6], bytes[off + 7]]);
                let w = i32::from_le_bytes([bytes[off + 12], bytes[off + 13], bytes[off + 14], bytes[off + 15]]);
                out.push((sid, w));
            }
        }
        out
    }

    #[test]
    fn header_ok() {
        let n = i32::from_le_bytes([BASE_SUMMON_LOT[0], BASE_SUMMON_LOT[1], BASE_SUMMON_LOT[2], BASE_SUMMON_LOT[3]]);
        assert_eq!(n, 792);
        assert_eq!(BASE_SUMMON_LOT.len(), 8 + 792 * 20);
    }

    #[test]
    fn forces_each_summon_independently() {
        // Behemoth Stout Heart + Skill Dmg Cap; Rolan Aegis + Norm Atk Cap; Lilith War Elemental + Stun
        let mut picks = vec![(0usize, 0usize); SUMMONS.len()];
        picks[0] = (0, 1);  // Behemoth: Stout Heart, skilldmgcap
        picks[4] = (5, 0);  // Rolan: Aegis, normatkcap
        picks[5] = (0, 4);  // Lilith: War Elemental, stun
        let lot = patched_summon_lot(&picks);

        let check = |pool: &str, keep_hex: &str| {
            let keep = u32::from_str_radix(keep_hex, 16).unwrap();
            let rows = pool_rows(&lot, pool);
            let kept: Vec<_> = rows.iter().filter(|(_, w)| *w > 1).collect();
            assert_eq!(kept.len(), 1, "pool {pool}: expected exactly one kept row");
            assert_eq!(kept[0].0, keep, "pool {pool}: kept wrong id");
            for (sid, w) in &rows { if *sid != keep { assert_eq!(*w, 1, "pool {pool}: non-kept not 1"); } }
        };
        check("DF902143", "A1A8E39D"); // Behemoth skill = Stout Heart
        check("393EF1D8", "2FFB509F"); // Behemoth equip = Skill Dmg Cap (normal)
        check("26428274", "E0ABFDFE"); // Rolan skill = Aegis
        check("A35D7A5C", "9245DFA4"); // Rolan equip = Norm Atk Cap (astral)
        check("05205336", "4C588C27"); // Lilith skill = War Elemental
        check("DCB2B22B", "F0F77BC1"); // Lilith equip = Stun (astral)
    }
}
