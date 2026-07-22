# GBFRER Summon Drop Picker

Source code for the small Windows GUI app bundled with the **GBFRER Summon Drop Picker**
mod for *Granblue Fantasy: Relink* (Endless Ragnarok). It's published here so you can
read exactly what the `.exe` does before you run it.

The app is a helper for the [Reloaded-II](https://github.com/Reloaded-Project/Reloaded-II)
data mod. It lets you force all six Infinity-boss summons — Behemoth III, Wee Pincer III,
Lucilius, Beelzebub, Rolan and Lilith — to drop at once, each with the skill and equip
bonus you choose, by re-weighting the game's own `summon_lot.tbl`.

## Is it safe?

- **100% offline.** There is no networking anywhere in this code — no HTTP client, no
  sockets, no telemetry, no auto-update. The only dependencies are `eframe`/`egui`
  (the GUI), `rfd` (the native "Browse for file" dialog), `serde_json` (reads a local
  settings file) and `winresource` (embeds the icon at build time). None of them are
  used to reach the internet here. Search the source for `reqwest`, `http`, `TcpStream`
  or `std::net` — there are no hits.
- **What it actually touches:** it reads and writes files only inside its own mod folder
  (`summon_lot.tbl` and the three other tables it ships, plus a small
  `picker_settings.json`), and — only when you press **Apply & Run Game** — it starts
  `Reloaded-II.exe` to launch the game. That's the whole of it. No memory editing, no
  code injection, no background processes.
- **Unsigned binary.** The released `.exe` isn't code-signed, so Windows SmartScreen may
  warn on first run. If you'd rather not trust a prebuilt binary, build it yourself
  (below) — you get the same app.

## Build it yourself

Install a [Rust toolchain](https://rustup.rs/) (built and tested with 1.96, MSVC target
on Windows), then:

```sh
cargo build --release
```

The binary lands at `target/release/egui_picker.exe` (distributed in the mod as
`GBFRER Summon Picker.exe`). Exact crate versions are pinned in `Cargo.lock`.

## How it works

`summon_lot.tbl` is a fixed-layout table: an 8-byte header (row count) followed by
20-byte rows — `Key`, `SkillId`/`BaseParamId`, `CurveId`, `Weight`, `Unk`, all
little-endian. To "force" a choice, the app sets the `Weight` of every other row in a
pool to `1` and leaves the chosen row untouched — the same fair bias the official table
tools produce, done here as a direct byte patch so nothing extra has to be installed.
The base tables (with each summon's equip pool split out so the bonuses can be chosen
independently) are embedded into the exe via `include_bytes!`.

The app can also install itself: if it's run as a lone `.exe`, it writes its own
`ModConfig.json` and default tables next to itself so Reloaded-II recognises it as a mod.

## What's in here

| Path | What it is |
|------|------------|
| `src/main.rs` | The entire app — UI, the table patcher, Reloaded-II detection & launch. |
| `base/*.tbl` | The mod's base game tables, embedded into the exe. |
| `dist/ModConfig.json` | The Reloaded-II mod manifest (also embedded, for self-install). |
| `build.rs`, `icon.ico`, `icon_256.rgba` | The window / executable icon. |

## Credits

- **Nenkai** — GBFRDataTools and the Relink modding documentation / hashing.
- **WistfulHopes** — `gbfrelink.utility.manager`, the mod loader this mod depends on.
- **Sewer56 / Reloaded-Project** — Reloaded-II.
- Built with **egui / eframe**.

## License

The code is released under the [MIT License](LICENSE). The `base/*.tbl` files are
modified *Granblue Fantasy: Relink* data, included only for interoperability; all game
assets remain © their respective owners.
