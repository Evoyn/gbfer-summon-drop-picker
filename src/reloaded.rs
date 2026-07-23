use std::fs;
use std::path::{Path, PathBuf};

pub struct DepReport {
    pub ok: bool,
    pub located: bool,
    pub items: Vec<(String, bool)>,
}

// reloaded's json has a bom, serde chokes on it
pub fn read_json(path: &Path) -> Option<serde_json::Value> {
    let txt = fs::read_to_string(path).ok()?;
    serde_json::from_str(txt.trim_start_matches('\u{feff}')).ok()
}

fn settings() -> Option<serde_json::Value> {
    let appdata = std::env::var("APPDATA").ok()?;
    read_json(&Path::new(&appdata).join("Reloaded-Mod-Loader-II").join("ReloadedII.json"))
}

fn mods_dir() -> Option<PathBuf> {
    settings()?.get("ModConfigDirectory").and_then(|x| x.as_str()).map(PathBuf::from)
}

// the other 3 come from utility.manager, check em anyway
pub fn check_deps() -> DepReport {
    let mods = mods_dir();
    let installed = |id: &str| {
        mods.as_ref().map(|m| m.join(id).join("ModConfig.json").exists()).unwrap_or(false)
    };

    let core = "gbfrelink.utility.manager";
    let mut items = vec![(core.to_string(), installed(core))];
    if let Some(m) = &mods {
        if let Some(v) = read_json(&m.join(core).join("ModConfig.json")) {
            if let Some(deps) = v.get("ModDependencies").and_then(|x| x.as_array()) {
                items.extend(deps.iter().filter_map(|d| d.as_str()).map(|d| (d.to_string(), installed(d))));
            }
        }
    }

    DepReport { ok: items.iter().all(|(_, f)| *f), located: mods.is_some(), items }
}

pub fn autodetect() -> (String, String) {
    let mut launcher = String::new();
    let mut game = String::new();

    if let Some(v) = settings() {
        if let Some(p) = v.get("LauncherPath").and_then(|x| x.as_str()) {
            launcher = p.to_string();
        }
        if let Some(dir) = v.get("ApplicationConfigDirectory").and_then(|x| x.as_str()) {
            if let Ok(entries) = fs::read_dir(dir) {
                for e in entries.flatten() {
                    let Some(app) = read_json(&e.path().join("AppConfig.json")) else { continue };
                    let loc = app.get("AppLocation").and_then(|x| x.as_str()).unwrap_or("");
                    let id = app.get("AppId").and_then(|x| x.as_str()).unwrap_or("");
                    if id.to_lowercase().contains("granblue") || loc.to_lowercase().contains("granblue") {
                        game = loc.to_string();
                        break;
                    }
                }
            }
        }
    }

    if launcher.is_empty() {
        for c in ["F:\\Reloaded-II\\Reloaded-II.exe", "C:\\Reloaded-II\\Reloaded-II.exe", "D:\\Reloaded-II\\Reloaded-II.exe"] {
            if Path::new(c).exists() {
                launcher = c.to_string();
                break;
            }
        }
    }

    (launcher, game)
}

pub fn launch(launcher: &str, game: &str) -> std::io::Result<()> {
    let mut cmd = std::process::Command::new(launcher);
    if !game.is_empty() && Path::new(game).exists() {
        cmd.arg("--launch").arg(game);
    }
    cmd.spawn().map(|_| ())
}
