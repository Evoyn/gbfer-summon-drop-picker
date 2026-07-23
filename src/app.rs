use std::fs;
use std::path::{Path, PathBuf};

use crate::data::{BONUSES, SUMMONS};
use crate::patch::{self, Pick, BASE_TABLES, VANILLA};
use crate::reloaded::{self, DepReport};

const MODCONFIG_JSON: &str = include_str!("../dist/ModConfig.json");

pub struct App {
    pub picks: Vec<Pick>,
    pub reloaded_path: String,
    pub game_path: String,
    pub status: String,
    pub dep: DepReport,
    pub logo: Option<eframe::egui::TextureHandle>,
    pub restored: bool,
    exe_dir: PathBuf,
}

impl App {
    pub fn new() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| PathBuf::from("."));

        let mut picks = vec![Pick::default(); SUMMONS.len()];
        let (mut reloaded_path, mut game_path) = reloaded::autodetect();

        if let Some(v) = reloaded::read_json(&exe_dir.join("picker_settings.json")) {
            if let Some(p) = v.get("reloaded_path").and_then(|x| x.as_str()).filter(|p| !p.is_empty()) {
                reloaded_path = p.to_string();
            }
            if let Some(p) = v.get("game_path").and_then(|x| x.as_str()).filter(|p| !p.is_empty()) {
                game_path = p.to_string();
            }
            // matched by name, not index, so adding or removing summons
            // doesn't scramble someone's saved picks
            if let Some(saved) = v.get("picks").and_then(|x| x.as_array()) {
                for e in saved {
                    let name = e.get("summon").and_then(|x| x.as_str()).unwrap_or_default();
                    let Some(i) = SUMMONS.iter().position(|s| s.name == name) else { continue };
                    if let Some(sk) = e.get("skill").and_then(|x| x.as_str()) {
                        if let Some(k) = SUMMONS[i].skills.iter().position(|x| x.0 == sk) {
                            picks[i].skill = k;
                        }
                    }
                    if let Some(bk) = e.get("bonus").and_then(|x| x.as_str()) {
                        if let Some(k) = BONUSES.iter().position(|b| b.key == bk) {
                            picks[i].bonus = k;
                        }
                    }
                    picks[i].half_cap = e.get("half_cap").and_then(|x| x.as_bool()).unwrap_or(false);
                }
            }
        }

        install_if_missing(&exe_dir, &picks);

        App {
            picks,
            reloaded_path,
            game_path,
            status: "Ready. Set each summon, then Apply.".into(),
            dep: reloaded::check_deps(),
            logo: None,
            restored: false,
            exe_dir,
        }
    }

    fn table_dir(&self) -> PathBuf {
        self.exe_dir.join("GBFR").join("data").join("system").join("table")
    }

    pub fn save_config(&self) {
        let picks: Vec<_> = SUMMONS
            .iter()
            .zip(&self.picks)
            .map(|(s, p)| serde_json::json!({
                "summon": s.name,
                "skill": s.skills[p.skill].0,
                "bonus": BONUSES[p.bonus].key,
                "half_cap": p.half_cap,
            }))
            .collect();

        let v = serde_json::json!({
            "reloaded_path": self.reloaded_path,
            "game_path": self.game_path,
            "picks": picks,
        });
        let _ = fs::write(
            self.exe_dir.join("picker_settings.json"),
            serde_json::to_string_pretty(&v).unwrap_or_default(),
        );
    }

    pub fn apply(&mut self) -> std::io::Result<()> {
        let dir = self.table_dir();
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("summon_lot.tbl"), patch::patched_summon_lot(&self.picks))?;
        for (name, bytes) in BASE_TABLES {
            fs::write(dir.join(name), bytes)?;
        }
        self.save_config();
        Ok(())
    }

    pub fn do_apply(&mut self) {
        self.restored = false;
        self.status = match self.apply() {
            Ok(()) => "Applied. Enable only this mod in Reloaded-II, then relaunch the game.".into(),
            Err(e) => format!("Could not write tables: {e}. Is the game running?"),
        };
    }

    // just disabling the mod doesn't undo an already deployed table,
    // so overwrite with the untouched ones and launch once
    pub fn restore_vanilla(&mut self) {
        let dir = self.table_dir();
        let write = fs::create_dir_all(&dir)
            .and_then(|_| VANILLA.iter().try_for_each(|(n, b)| fs::write(dir.join(n), *b)));

        match write {
            Ok(()) => {
                self.restored = true;
                self.status = "Vanilla restored. Click the green button to start the game and finish.".into();
            }
            Err(e) => self.status = format!("Could not write tables: {e}"),
        }
    }

    // launches the game without re-applying, so the loader deploys the vanilla tables
    pub fn finish_restore(&mut self) {
        if self.reloaded_path.is_empty() || !Path::new(&self.reloaded_path).exists() {
            self.status = "Start the game yourself with the mod still ticked, load a save and quit, then untick or delete the mod.".into();
            return;
        }
        self.status = match reloaded::launch(&self.reloaded_path, &self.game_path) {
            Ok(()) => "Game starting with vanilla data. Once you load a save and quit, untick or delete the mod.".into(),
            Err(e) => format!("Could not start Reloaded-II: {e}. Start the game yourself, then remove the mod."),
        };
    }

    pub fn run_game(&mut self) {
        self.restored = false;
        if let Err(e) = self.apply() {
            self.status = format!("Could not write tables: {e}");
            return;
        }
        if self.reloaded_path.is_empty() || !Path::new(&self.reloaded_path).exists() {
            self.status = "Picks applied. Reloaded-II not found, set its path below or launch the game yourself.".into();
            return;
        }
        self.status = match reloaded::launch(&self.reloaded_path, &self.game_path) {
            Ok(()) if self.game_path.is_empty() => "Applied. Opened Reloaded-II, press Launch there.".into(),
            Ok(()) => "Applied, launching the game through Reloaded-II.".into(),
            Err(e) => format!("Applied, but could not start Reloaded-II: {e}"),
        };
    }

    pub fn recheck_deps(&mut self) {
        self.dep = reloaded::check_deps();
        self.status = "Re-checked requirements.".into();
    }
}

// lone exe drops its own manifest + tables so reloaded sees a mod
fn install_if_missing(exe_dir: &Path, picks: &[Pick]) {
    let manifest = exe_dir.join("ModConfig.json");
    if !manifest.exists() {
        let _ = fs::write(&manifest, MODCONFIG_JSON);
    }

    let dir = exe_dir.join("GBFR").join("data").join("system").join("table");
    if dir.join("summon_lot.tbl").exists() || fs::create_dir_all(&dir).is_err() {
        return;
    }
    let _ = fs::write(dir.join("summon_lot.tbl"), patch::patched_summon_lot(picks));
    for (name, bytes) in BASE_TABLES {
        let _ = fs::write(dir.join(name), bytes);
    }
}
