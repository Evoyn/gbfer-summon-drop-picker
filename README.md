# GBFRER Summon Drop Picker

Source code for the small Windows GUI app bundled with the **GBFRER Summon Drop Picker**
mod for *Granblue Fantasy: Relink* (Endless Ragnarok). It's published here so you can
read exactly what the `.exe` does before you run it.

Mod page: https://www.nexusmods.com/granbluefantasyrelink/mods/677

The app is a helper for the [Reloaded-II](https://github.com/Reloaded-Project/Reloaded-II)
data mod. It lets you force all eight Infinity-boss summons (Behemoth III, Wee Pincer III,
Albacore III, Furycane Nihilla, Lucilius, Beelzebub, Rolan and Lilith) to drop at once,
each with the skill and equip bonus you choose, by re-weighting the game's own
`summon_lot.tbl`. Astral summons can also be capped to normal-tier values (50% instead
of 100%) if the astral maximum is more than you want.

## Is it safe?

- **100% offline.** There is no networking anywhere in this code - no HTTP client, no
  sockets, no telemetry, no auto-update. The only dependencies are `eframe`/`egui`
  (the GUI), `rfd` (the native "Browse for file" dialog), `serde_json` (reads a local
  settings file) and `winresource` (embeds the icon at build time). None of them are
  used to reach the internet here. Search the source for `reqwest`, `http`, `TcpStream`
  or `std::net` - there are no hits.
- **What it actually touches:** it reads and writes files only inside its own mod folder
  (`summon_lot.tbl` and the three other tables it ships, plus a small
  `picker_settings.json`), and - only when you press **Apply & Run Game** - it starts
  `Reloaded-II.exe` to launch the game. That's the whole of it. No memory editing, no
  code injection, no background processes.
- **Nothing to install.** The Visual C++ runtime is statically linked (see
  `.cargo/config.toml`, which sets `+crt-static`), so the `.exe` runs on a clean
  Windows 10/11 - no VC++ Redistributable, no .NET, no other dependencies. Every DLL it
  imports (`kernel32`, `user32`, `gdi32`, `opengl32`, `shell32`, ...) ships with Windows.
- **Unsigned binary.** The released `.exe` isn't code-signed, so Windows SmartScreen may
  warn on first run. If you'd rather not trust a prebuilt binary, build it yourself
  (below) - you get the same app.

## Build it yourself

Install a [Rust toolchain](https://rustup.rs/) (built and tested with 1.96, MSVC target
on Windows), then:

```sh
cargo build --release
```

The binary lands at `target/release/gbfrer-summon-drop-picker.exe` (distributed in the mod as
`GBFRER Summon Picker.exe`). Exact crate versions are pinned in `Cargo.lock`.

The released binary is built with `--remap-path-prefix` on top of that, so the build
machine's own directories (the Cargo registry, the checkout path) don't end up baked
into the executable as strings. That's the only difference from a plain
`cargo build --release`, which gives you the same app with your own paths in it.

## How it works

`summon_lot.tbl` is a fixed-layout table: an 8-byte header (row count) followed by
20-byte rows - `Key`, `SkillId`/`BaseParamId`, `CurveId`, `Weight`, `Unk`, all
little-endian. To "force" a choice, the app sets the `Weight` of every other row in a
pool to `1` and leaves the chosen row untouched - the same fair bias the official table
tools produce, done here as a direct byte patch so nothing extra has to be installed.
The base tables (with each summon's equip pool split out so the bonuses can be chosen
independently) are embedded into the exe via `include_bytes!`.

For the astral summons, the half-cap option repoints their equip pool at a different
level curve. The astral curve spans levels 5-9, so level 9 gives the astral maximum
(100%) and level 6 gives exactly the normal-tier values (50%). Both are levels the game
rolls naturally.

The app can also install itself: run as a lone `.exe`, it writes its own `ModConfig.json`
and default tables next to itself so Reloaded-II recognises it as a mod. It also has a
"Restore vanilla tables" button, which writes the untouched game tables back so a forced
drop can be cleared before removing the mod.

## What's in here

| Path | What it is |
|------|------------|
| `src/data.rs` | The summon and bonus tables (ids, hashes, values). |
| `src/patch.rs` | The embedded `.tbl` bytes and the byte patcher. |
| `src/reloaded.rs` | Reloaded-II detection, dependency check, launching. |
| `src/app.rs` | State and the actions (apply, restore, run). |
| `src/ui.rs` | The egui layout and theme. |
| `base/*.tbl` | The mod's base game tables, embedded into the exe. |
| `vanilla/*.tbl` | The untouched game tables, for the restore option. |
| `dist/ModConfig.json` | The Reloaded-II mod manifest (also embedded, for self-install). |
| `build.rs`, `assets/` | The window and executable icon. |

## Credits

- **Nenkai** and all contributors - [GBFRDataTools](https://github.com/Nenkai/GBFRDataTools), the table tools and hashing.
- **Nenkai** and **WistfulHopes** - `gbfrelink.utility.manager`, the mod loader this mod depends on.
- **Sewer56 / Reloaded-Project** - Reloaded-II.
- Built with **egui / eframe**.

## License

MIT. Do what you want with the code, no warranty. The `base/*.tbl` and `vanilla/*.tbl`
files are *Granblue Fantasy: Relink* data, included only for interoperability; all game
assets remain (c) their respective owners.
