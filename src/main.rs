// GBFRER Summon Drop Picker. Re-weights this mod's own summon_lot.tbl so each
// summon drops with a chosen skill and equip bonus. Offline, no game tools needed.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::fs;
use std::path::{Path, PathBuf};

const BASE_SUMMON_LOT: &[u8] = include_bytes!("../base/summon_lot.tbl");
const BASE_SUMMON: &[u8] = include_bytes!("../base/summon.tbl");
const BASE_REWARD: &[u8] = include_bytes!("../base/reward_summon_lot.tbl");
const BASE_CURVE: &[u8] = include_bytes!("../base/summon_curve.tbl");
const MODCONFIG_JSON: &str = include_str!("../dist/ModConfig.json");
const WINDOW_ICON: &[u8] = include_bytes!("../icon_256.rgba");

// Untouched game tables. Writing these lets the loader serve vanilla data on the
// next launch, which is how you clear a forced drop before removing the mod.
const VANILLA: [(&str, &[u8]); 4] = [
    ("summon_lot.tbl", include_bytes!("../vanilla/summon_lot.tbl")),
    ("summon.tbl", include_bytes!("../vanilla/summon.tbl")),
    ("reward_summon_lot.tbl", include_bytes!("../vanilla/reward_summon_lot.tbl")),
    ("summon_curve.tbl", include_bytes!("../vanilla/summon_curve.tbl")),
];

// Astral equip curve. Level 9 gives the astral maximum (100% caps); level 6 gives
// exactly the normal-tier values (50% caps, Stun 15, Attack +2000, Health +2000).
// Both are levels the curve can roll naturally, so either choice stays legit.
const ASTRAL_CURVE_MAX: &str = "4E547493";
const ASTRAL_CURVE_HALF: &str = "5A370B86";

struct Summon {
    name: &'static str,
    astral: bool,
    skill_pool: &'static str,
    equip_pool: &'static str,
    skills: &'static [(&'static str, &'static str)],
}

struct Bonus {
    key: &'static str,
    name: &'static str,
    normal_id: &'static str,
    normal_max: &'static str,
    astral_id: &'static str,
    astral_max: &'static str,
}

const BONUSES: &[Bonus] = &[
    Bonus { key: "normatkcap",  name: "Normal Attack Damage Cap Up", normal_id: "A66241C9", normal_max: "50%",   astral_id: "9245DFA4", astral_max: "100%" },
    Bonus { key: "skilldmgcap", name: "Skill Damage Cap Up",         normal_id: "2FFB509F", normal_max: "50%",   astral_id: "CE70C58A", astral_max: "100%" },
    Bonus { key: "skyartcap",   name: "Skybound Art Damage Cap Up",  normal_id: "A3539FBB", normal_max: "50%",   astral_id: "5A1D2C89", astral_max: "100%" },
    Bonus { key: "healcap",     name: "Healing Cap Up",              normal_id: "2270BC40", normal_max: "50%",   astral_id: "2EA9CA80", astral_max: "75%"  },
    Bonus { key: "stun",        name: "Stun Power Up",               normal_id: "F7B0316F", normal_max: "15",    astral_id: "F0F77BC1", astral_max: "20"   },
    Bonus { key: "atk",         name: "Attack Power Up",             normal_id: "A8900C80", normal_max: "+2000", astral_id: "A3E537B1", astral_max: "+3000"},
    Bonus { key: "hp",          name: "Health Up",                   normal_id: "BC4E92CB", normal_max: "+2000", astral_id: "F2256E44", astral_max: "+5000"},
    Bonus { key: "crit",        name: "Critical Hit Rate Up",        normal_id: "00D171E0", normal_max: "20%",   astral_id: "A074A967", astral_max: "30%"  },
    Bonus { key: "skilldmg",    name: "Skill Damage Up",             normal_id: "664E8E32", normal_max: "20%",   astral_id: "AF39A3FB", astral_max: "30%"  },
    Bonus { key: "skyart",      name: "Skybound Art Damage Up",      normal_id: "75138D21", normal_max: "20%",   astral_id: "33C0D50C", astral_max: "30%"  },
    Bonus { key: "chainburst",  name: "Chain Burst Damage Up",       normal_id: "5A39D81B", normal_max: "50%",   astral_id: "54B09A37", astral_max: "100%" },
];

// Skill ids are the raw hashes stored in summon_lot.tbl. Signature skills (the ones
// on the single-level curve) are listed first, then the rest of that summon's pool.
const SUMMONS: &[Summon] = &[
    Summon { name: "Behemoth III", astral: false, skill_pool: "DF902143", equip_pool: "393EF1D8", skills: &[
        ("Stout Heart", "A1A8E39D"), ("Supplementary DMG", "57AB5B10"), ("Uplift", "B5FF9FD3"),
        ("Celestial Ventus", "73220725"), ("Less Is More", "82CE278D"), ("Critical Hit Rate", "8D78A19B") ] },
    Summon { name: "Wee Pincer III", astral: false, skill_pool: "2340DCED", equip_pool: "C39F7144", skills: &[
        ("Crabvestment Returns", "1B0D9897"), ("DMG Cap", "DC584F60"), ("Cascade", "05F2ECDC"), ("Steel Nerves", "1470F860") ] },
    Summon { name: "Albacore III", astral: false, skill_pool: "1250EFD2", equip_pool: "E31F7BAD", skills: &[
        ("Natural Defenses", "0EAD65E0"), ("Path to Mastery", "5E422AE5"), ("Rupie Tycoon", "C86F3082"), ("Fast Learner", "F687C5EF") ] },
    Summon { name: "Furycane Nihilla", astral: false, skill_pool: "384C3333", equip_pool: "FD5DACF0", skills: &[
        ("Celestial Aqua", "A898E283"), ("Fatebreaker", "D029FE08"), ("Potion Hoarder", "24883AF3"),
        ("Guts", "E69A4694"), ("Blight Resistance", "9702860F") ] },
    Summon { name: "Lucilius", astral: true, skill_pool: "CC0B0EF1", equip_pool: "F63793E4", skills: &[
        ("Alpha", "DBE1D775"), ("Beta", "8D2ADB6E"), ("Gamma", "5C862E13"), ("Berserker Echo", "EE85CD1F"),
        ("Tyranny", "71F11A9B"), ("Celestial Terra", "9232DC17") ] },
    Summon { name: "Beelzebub", astral: true, skill_pool: "BD452CC9", equip_pool: "B7D9379B", skills: &[
        ("Spartan Echo", "3D8153A1"), ("Supplementary DMG", "57AB5B10"), ("DMG Cap", "DC584F60"),
        ("Drain", "7CCFF74F"), ("Celestial Lumen", "A7726190"), ("Improved Guard", "0AA20846") ] },
    Summon { name: "Rolan", astral: true, skill_pool: "26428274", equip_pool: "A35D7A5C", skills: &[
        ("War Elemental", "4C588C27"), ("Uplift", "B5FF9FD3"), ("Autorevive", "95F3FA86"),
        ("Quick Cooldown", "318D12E9"), ("Drain", "7CCFF74F"), ("Aegis", "E0ABFDFE") ] },
    Summon { name: "Lilith", astral: true, skill_pool: "05205336", equip_pool: "DCB2B22B", skills: &[
        ("War Elemental", "4C588C27"), ("Uplift", "B5FF9FD3"), ("Potion Hoarder", "24883AF3"),
        ("Tyranny", "71F11A9B"), ("Linked Together", "3FEC5F80"), ("Improved Healing", "9389CC06") ] },
];

// summon_lot.tbl: 8-byte header (int32 rowCount), then 20-byte rows:
//   +0 Key  +4 SkillId/BaseParamId  +8 CurveId  +12 Weight(i32)  +16 Unk5, all little-endian.
fn rows(bytes: &[u8]) -> usize {
    i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize
}

fn field(bytes: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]])
}

fn hex(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap_or(0)
}

/// Every row of `pool` keeps its weight only if its id is `keep`, otherwise drops to 1.
fn force_pool(bytes: &mut [u8], pool: &str, keep: &str) {
    let (pool, keep) = (hex(pool), hex(keep));
    for r in 0..rows(bytes) {
        let off = 8 + r * 20;
        if off + 20 > bytes.len() { break; }
        if field(bytes, off) == pool && field(bytes, off + 4) != keep {
            bytes[off + 12..off + 16].copy_from_slice(&1i32.to_le_bytes());
        }
    }
}

/// Repoints a pool's rows at a different level curve, which is how the astral
/// 100% / 50% choice is applied.
fn set_pool_curve(bytes: &mut [u8], pool: &str, curve: &str) {
    let (pool, curve) = (hex(pool), hex(curve));
    for r in 0..rows(bytes) {
        let off = 8 + r * 20;
        if off + 20 > bytes.len() { break; }
        if field(bytes, off) == pool {
            bytes[off + 8..off + 12].copy_from_slice(&curve.to_le_bytes());
        }
    }
}

#[derive(Clone, Copy)]
struct Pick {
    skill: usize,
    bonus: usize,
    half_cap: bool,
}

fn patched_summon_lot(picks: &[Pick]) -> Vec<u8> {
    let mut lot = BASE_SUMMON_LOT.to_vec();
    for (i, s) in SUMMONS.iter().enumerate() {
        let p = picks[i];
        force_pool(&mut lot, s.skill_pool, s.skills[p.skill].1);
        let b = &BONUSES[p.bonus];
        force_pool(&mut lot, s.equip_pool, if s.astral { b.astral_id } else { b.normal_id });
        if s.astral {
            let curve = if p.half_cap { ASTRAL_CURVE_HALF } else { ASTRAL_CURVE_MAX };
            set_pool_curve(&mut lot, s.equip_pool, curve);
        }
    }
    lot
}

struct DepReport {
    ok: bool,
    located: bool,
    items: Vec<(String, bool)>,
}

struct App {
    picks: Vec<Pick>,
    reloaded_path: String,
    game_path: String,
    status: String,
    exe_dir: PathBuf,
    dep: DepReport,
    logo: Option<egui::TextureHandle>,
}

fn read_json(path: &Path) -> Option<serde_json::Value> {
    let txt = fs::read_to_string(path).ok()?;
    serde_json::from_str(txt.trim_start_matches('\u{feff}')).ok()
}

impl App {
    fn new() -> Self {
        let exe_dir = std::env::current_exe().ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        let mut picks = vec![Pick { skill: 0, bonus: 0, half_cap: false }; SUMMONS.len()];
        let (mut reloaded_path, mut game_path) = autodetect();

        if let Some(v) = read_json(&exe_dir.join("picker_settings.json")) {
            if let Some(p) = v.get("reloaded_path").and_then(|x| x.as_str()) { if !p.is_empty() { reloaded_path = p.to_string(); } }
            if let Some(p) = v.get("game_path").and_then(|x| x.as_str()) { if !p.is_empty() { game_path = p.to_string(); } }
            if let Some(arr) = v.get("picks").and_then(|x| x.as_array()) {
                for e in arr {
                    let name = e.get("summon").and_then(|x| x.as_str()).unwrap_or_default();
                    let Some(i) = SUMMONS.iter().position(|s| s.name == name) else { continue };
                    if let Some(sk) = e.get("skill").and_then(|x| x.as_str()) {
                        if let Some(k) = SUMMONS[i].skills.iter().position(|x| x.0 == sk) { picks[i].skill = k; }
                    }
                    if let Some(bk) = e.get("bonus").and_then(|x| x.as_str()) {
                        if let Some(k) = BONUSES.iter().position(|x| x.key == bk) { picks[i].bonus = k; }
                    }
                    picks[i].half_cap = e.get("half_cap").and_then(|x| x.as_bool()).unwrap_or(false);
                }
            }
        }
        ensure_installed(&exe_dir, &picks);
        Self { picks, reloaded_path, game_path, status: "Ready. Set each summon, then Apply.".into(), exe_dir, dep: check_deps(), logo: None }
    }

    fn save_config(&self) {
        let picks: Vec<serde_json::Value> = SUMMONS.iter().enumerate().map(|(i, s)| {
            serde_json::json!({
                "summon": s.name,
                "skill": s.skills[self.picks[i].skill].0,
                "bonus": BONUSES[self.picks[i].bonus].key,
                "half_cap": self.picks[i].half_cap,
            })
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

    /// Overwrites the active tables with the untouched game data. Needed because
    /// simply disabling the mod can leave already-deployed tables in place.
    fn restore_vanilla(&mut self) {
        let dir = self.exe_dir.join("GBFR").join("data").join("system").join("table");
        let write = fs::create_dir_all(&dir)
            .and_then(|_| VANILLA.iter().try_for_each(|(n, b)| fs::write(dir.join(n), *b)));
        self.status = match write {
            Ok(()) => "Vanilla tables restored. Launch the game once with the mod still enabled, then disable or delete it.".into(),
            Err(e) => format!("Could not write tables: {e}"),
        };
    }

    fn do_apply(&mut self) {
        self.status = match self.apply() {
            Ok(()) => "Applied. Enable only this mod in Reloaded-II, then relaunch the game.".into(),
            Err(e) => format!("Could not write tables: {e}. Is the game running?"),
        };
    }

    fn run_game(&mut self) {
        if let Err(e) = self.apply() {
            self.status = format!("Could not write tables: {e}");
            return;
        }
        if self.reloaded_path.is_empty() || !Path::new(&self.reloaded_path).exists() {
            self.status = "Picks applied. Reloaded-II not found, set its path below or launch the game yourself.".into();
            return;
        }
        let mut cmd = std::process::Command::new(&self.reloaded_path);
        if !self.game_path.is_empty() && Path::new(&self.game_path).exists() {
            cmd.arg("--launch").arg(&self.game_path);
        }
        self.status = match cmd.spawn() {
            Ok(_) if self.game_path.is_empty() => "Applied. Opened Reloaded-II, press Launch there.".into(),
            Ok(_) => "Applied, launching the game through Reloaded-II.".into(),
            Err(e) => format!("Applied, but could not start Reloaded-II: {e}"),
        };
    }
}

fn mods_dir() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    let v = read_json(&Path::new(&appdata).join("Reloaded-Mod-Loader-II").join("ReloadedII.json"))?;
    v.get("ModConfigDirectory").and_then(|x| x.as_str()).map(PathBuf::from)
}

/// The three loader mods belong to gbfrelink.utility.manager, not to us. We only
/// declare utility.manager and check its chain so a missing piece is visible.
fn check_deps() -> DepReport {
    let mods = mods_dir();
    let has = |id: &str| mods.as_ref().map(|m| m.join(id).join("ModConfig.json").exists()).unwrap_or(false);
    let core = "gbfrelink.utility.manager";
    let mut items = vec![(core.to_string(), has(core))];
    if let Some(m) = &mods {
        if let Some(v) = read_json(&m.join(core).join("ModConfig.json")) {
            if let Some(arr) = v.get("ModDependencies").and_then(|x| x.as_array()) {
                for d in arr.iter().filter_map(|x| x.as_str()) {
                    items.push((d.to_string(), has(d)));
                }
            }
        }
    }
    DepReport { ok: items.iter().all(|(_, f)| *f), located: mods.is_some(), items }
}

/// Lets a bare exe work as a mod: writes ModConfig.json and default tables if absent.
fn ensure_installed(exe_dir: &Path, picks: &[Pick]) {
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
        if let Some(v) = read_json(&Path::new(&appdata).join("Reloaded-Mod-Loader-II").join("ReloadedII.json")) {
            if let Some(lp) = v.get("LauncherPath").and_then(|x| x.as_str()) { reloaded = lp.to_string(); }
            if let Some(acd) = v.get("ApplicationConfigDirectory").and_then(|x| x.as_str()) {
                if let Ok(rd) = fs::read_dir(acd) {
                    for e in rd.flatten() {
                        let Some(j) = read_json(&e.path().join("AppConfig.json")) else { continue };
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
    if reloaded.is_empty() {
        for c in ["F:\\Reloaded-II\\Reloaded-II.exe", "C:\\Reloaded-II\\Reloaded-II.exe", "D:\\Reloaded-II\\Reloaded-II.exe"] {
            if Path::new(c).exists() { reloaded = c.to_string(); break; }
        }
    }
    (reloaded, game)
}

const ACCENT: egui::Color32 = egui::Color32::from_rgb(0x8b, 0x7c, 0xf0);
const MUTED: egui::Color32 = egui::Color32::from_rgb(0x8b, 0x92, 0xa0);
const CARD: egui::Color32 = egui::Color32::from_rgb(0x1e, 0x20, 0x2a);
const LINE: egui::Color32 = egui::Color32::from_rgb(0x2d, 0x32, 0x41);

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
        let hover = Color32::from_rgb(0x32, 0x37, 0x46);
        let active = Color32::from_rgb(0x3b, 0x41, 0x53);
        let text = Color32::from_rgb(0xdf, 0xe3, 0xea);
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

enum Act { None, Apply, Run, Save, Recheck, Restore }

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        use egui::{Color32, CornerRadius, Margin, RichText, Stroke};
        let mut act = Act::None;

        if self.logo.is_none() {
            let img = egui::ColorImage::from_rgba_unmultiplied([256, 256], WINDOW_ICON);
            self.logo = Some(ui.ctx().load_texture("logo", img, egui::TextureOptions::LINEAR));
        }

        // eframe hands us the bare window ui, so the outer margin has to come from here.
        let pad = egui::Frame::group(ui.style())
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE)
            .inner_margin(Margin::symmetric(14, 10));

        pad.show(ui, |ui| {
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

                let (fill, msg) = if !self.dep.located {
                    (Color32::from_rgb(0x40, 0x36, 0x1c), "Could not find Reloaded-II's Mods folder to check requirements.".to_string())
                } else if self.dep.ok {
                    (Color32::from_rgb(0x17, 0x32, 0x26), "Loader ready, gbfrelink.utility.manager and its dependencies are installed.".to_string())
                } else {
                    let miss: Vec<_> = self.dep.items.iter().filter(|(_, f)| !*f).map(|(n, _)| n.clone()).collect();
                    (Color32::from_rgb(0x44, 0x22, 0x24), format!("Missing: {}. Install it or the mod will not load.", miss.join(", ")))
                };
                egui::Frame::group(ui.style()).fill(fill).stroke(Stroke::new(1.0, LINE))
                    .corner_radius(CornerRadius::same(8)).inner_margin(Margin::same(10)).show(ui, |ui| {
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

                for (i, s) in SUMMONS.iter().enumerate() {
                    egui::Frame::group(ui.style()).fill(CARD).stroke(Stroke::new(1.0, LINE))
                        .corner_radius(CornerRadius::same(9)).inner_margin(Margin::same(11)).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(s.name).size(16.0).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let (cf, ct, label) = if s.astral {
                                    (Color32::from_rgb(0x3d, 0x33, 0x5e), Color32::from_rgb(0xcf, 0xc3, 0xf5), "ASTRAL")
                                } else {
                                    (Color32::from_rgb(0x33, 0x3b, 0x4d), Color32::from_rgb(0xba, 0xc4, 0xd7), "NORMAL")
                                };
                                egui::Frame::group(ui.style()).fill(cf).stroke(Stroke::NONE)
                                    .corner_radius(CornerRadius::same(6)).inner_margin(Margin::symmetric(8, 3)).show(ui, |ui| {
                                    ui.label(RichText::new(label).size(11.0).color(ct).strong());
                                });
                            });
                        });
                        ui.add_space(6.0);
                        egui::Grid::new(("g", i)).num_columns(2).spacing([12.0, 8.0]).min_col_width(48.0).show(ui, |ui| {
                            ui.label(RichText::new("Skill").color(MUTED));
                            if s.skills.len() > 1 {
                                let cur = s.skills[self.picks[i].skill].0;
                                egui::ComboBox::from_id_salt(("sk", i)).width(320.0).selected_text(cur).show_ui(ui, |ui| {
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
                            let val = |b: &Bonus| if s.astral && !half { b.astral_max } else { b.normal_max };
                            let bi = self.picks[i].bonus;
                            let cur = format!("{}  {}", BONUSES[bi].name, val(&BONUSES[bi]));
                            egui::ComboBox::from_id_salt(("bn", i)).width(320.0).selected_text(cur).show_ui(ui, |ui| {
                                for (k, b) in BONUSES.iter().enumerate() {
                                    ui.selectable_value(&mut self.picks[i].bonus, k, format!("{}  {}", b.name, val(b)));
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
                    if ui.button("Restore vanilla tables").clicked() { act = Act::Restore; }
                });
                ui.add_space(14.0);

                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("Apply Picks").min_size(egui::vec2(150.0, 34.0))).clicked() { act = Act::Apply; }
                    let run = egui::Button::new(RichText::new("Apply & Run Game").color(Color32::WHITE).strong())
                        .fill(ACCENT).min_size(egui::vec2(220.0, 34.0));
                    if ui.add(run).clicked() { act = Act::Run; }
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
            Act::Recheck => { self.dep = check_deps(); self.status = "Re-checked requirements.".into(); }
            Act::Restore => self.restore_vanilla(),
            Act::None => {}
        }
    }
}

fn main() -> eframe::Result<()> {
    if std::env::args().any(|a| a == "--apply") {
        let mut app = App::new();
        std::process::exit(if app.apply().is_ok() { 0 } else { 1 });
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([660.0, 760.0])
            .with_min_inner_size([560.0, 420.0])
            .with_icon(std::sync::Arc::new(egui::IconData { rgba: WINDOW_ICON.to_vec(), width: 256, height: 256 }))
            .with_title("GBFRER Summon Drop Picker"),
        ..Default::default()
    };
    eframe::run_native("GBFRER Summon Drop Picker", options, Box::new(|cc| {
        setup_style(&cc.egui_ctx);
        Ok(Box::new(App::new()))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool_rows(bytes: &[u8], pool: &str) -> Vec<(u32, u32, i32)> {
        let pool = hex(pool);
        (0..rows(bytes)).filter_map(|r| {
            let off = 8 + r * 20;
            (field(bytes, off) == pool).then(|| (
                field(bytes, off + 4),
                field(bytes, off + 8),
                i32::from_le_bytes([bytes[off + 12], bytes[off + 13], bytes[off + 14], bytes[off + 15]]),
            ))
        }).collect()
    }

    fn default_picks() -> Vec<Pick> {
        vec![Pick { skill: 0, bonus: 0, half_cap: false }; SUMMONS.len()]
    }

    #[test]
    fn base_table_shape() {
        // 726 vanilla rows plus one 11-row equip pool per summon
        let expected = 726 + SUMMONS.len() * 11;
        assert_eq!(rows(BASE_SUMMON_LOT), expected);
        assert_eq!(BASE_SUMMON_LOT.len(), 8 + expected * 20);
    }

    #[test]
    fn vanilla_tables_are_the_unmodified_ones() {
        let vanilla_lot = VANILLA.iter().find(|(n, _)| *n == "summon_lot.tbl").unwrap().1;
        assert_eq!(rows(vanilla_lot), 726);
    }

    #[test]
    fn every_summon_pool_exists_and_holds_its_skills() {
        let lot = BASE_SUMMON_LOT;
        for s in SUMMONS {
            assert_eq!(pool_rows(lot, s.equip_pool).len(), 11, "{} equip pool", s.name);
            let ids: Vec<u32> = pool_rows(lot, s.skill_pool).iter().map(|r| r.0).collect();
            for (n, h) in s.skills {
                assert!(ids.contains(&hex(h)), "{} missing skill {}", s.name, n);
            }
        }
    }

    #[test]
    fn forces_skill_and_bonus_per_summon() {
        let rolan = SUMMONS.iter().position(|s| s.name == "Rolan").unwrap();
        let mut picks = default_picks();
        picks[0] = Pick { skill: 3, bonus: 1, half_cap: false };       // Behemoth, Celestial Ventus, Skill Dmg Cap
        picks[rolan] = Pick { skill: 5, bonus: 4, half_cap: false };   // Rolan, Aegis, Stun
        let lot = patched_summon_lot(&picks);

        let kept = |pool: &str| -> Vec<u32> {
            pool_rows(&lot, pool).iter().filter(|r| r.2 > 1).map(|r| r.0).collect()
        };
        assert_eq!(kept("DF902143"), vec![hex("73220725")]);
        assert_eq!(kept("393EF1D8"), vec![hex("2FFB509F")]);
        assert_eq!(kept("26428274"), vec![hex("E0ABFDFE")]);
        assert_eq!(kept("A35D7A5C"), vec![hex("F0F77BC1")]);
    }

    #[test]
    fn astral_cap_switches_curve_and_normals_are_untouched() {
        let lucilius = SUMMONS.iter().position(|s| s.name == "Lucilius").unwrap();
        let mut picks = default_picks();
        picks[lucilius].half_cap = true;
        let lot = patched_summon_lot(&picks);

        for r in pool_rows(&lot, "F63793E4") {
            assert_eq!(r.1, hex(ASTRAL_CURVE_HALF));
        }
        for r in pool_rows(&lot, "B7D9379B") {
            assert_eq!(r.1, hex(ASTRAL_CURVE_MAX), "Beelzebub should stay at max");
        }
        // normal-tier summons keep the normal curve regardless
        for r in pool_rows(&lot, "393EF1D8") {
            assert_eq!(r.1, hex("2E2C5483"));
        }
    }
}
