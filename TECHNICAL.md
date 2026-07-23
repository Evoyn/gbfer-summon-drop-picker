# GBFRER Summon Drop Picker - Technical Reference

Everything about how this mod works: the reverse-engineered table formats, the IDs
and hashes, the patch logic, the verified values, and how the app is built.

For *Granblue Fantasy: Relink* (Endless Ragnarok, game v2.0.2), loaded through
[Reloaded-II](https://github.com/Reloaded-Project/Reloaded-II) +
[gbfrelink.utility.manager](https://www.nexusmods.com/granbluefantasyrelink/mods/526).

- Repo (GUI source): https://github.com/Evoyn/gbfer-summon-drop-picker
- Mod page: https://www.nexusmods.com/granbluefantasyrelink/mods/677

---

## Contents

1. [What it does](#1-what-it-does)
2. [Editions](#2-editions)
3. [Design overview: split pools + weight forcing](#3-design-overview)
4. [The GBFR drop pipeline](#4-the-gbfr-drop-pipeline)
5. [Table formats (reverse-engineered)](#5-table-formats)
6. [The XXHash32Custom hash](#6-the-xxhash32custom-hash)
7. [Reference data (summons, pools, curves)](#7-reference-data)
8. [Equip bonuses (11) + verified values](#8-equip-bonuses)
9. [Full skill pools per summon](#9-full-skill-pools)
10. [Part A - the baked base edits](#10-part-a--baked-base-edits)
11. [Part B - the in-place binary patcher](#11-part-b--the-in-place-binary-patcher)
12. [The GUI app (Rust / egui)](#12-the-gui-app)
13. [Building from source](#13-building-from-source)
14. [How it was verified](#14-how-it-was-verified)
15. [Credits](#15-credits)

---

## 1. What it does

Forces which of the eight Infinity-boss summons drop, and lets you choose **each
summon's skill and equip bonus independently**, all forced at the same time (each
summon from its own quest). It does this purely by editing the game's own data
tables - no memory editing, no code injection, no network access.

Every value it grants is one the game can produce naturally at that summon's max
level; it only biases the RNG toward your choice, it never creates impossible items.

The eight summons:

| Summon | Tier | Notes |
|---|---|---|
| Behemoth III | normal | caps at 50% |
| Wee Pincer III | normal | caps at 50% |
| Lucilius | astral | caps at 100% |
| Beelzebub | astral | caps at 100% |
| Rolan | astral | caps at 100% |
| Lilith | astral | caps at 100% |
| Albacore III | normal | caps at 50% |
| Furycane Nihilla | normal | caps at 50% |

"normal" vs "astral" is the equip-bonus value tier (see section 8).

---

## 2. Editions

All three edit the same tables and produce identical results; they differ only in
the front-end.

| Edition | Front-end | Notes |
|---|---|---|
| **Console** | `pick.bat` -> `pick-summon.ps1` (pure PowerShell menu) | No compiled code; scripts are plain text. |
| **GUI** | `GBFRER Summon Picker.exe` (Rust/egui app) | Dropdowns + "Apply & Run Game". Nothing to install. |
| **Standalone exe** | the same `.exe`, alone | Self-installs its `ModConfig.json` + tables on first run. |

Both front-ends apply the **exact same byte patch** to `summon_lot.tbl` - the GUI
was verified byte-for-byte equal to the PowerShell path.

---

## 3. Design overview

The base game **shares** equip-bonus pools: the four astrals all read pool
`48727C41`, and Behemoth III + Wee Pincer III share `439EA421`. A pool can only
force one bonus, so out of the box every astral would be stuck with the *same*
bonus. Two parts solve this:

- **Part A (baked into the download):** give every summon its **own** equip pool
  (repoint `summon.SummonLotId3`, clone the tier's 11-row pool into `summon_lot`
  with a fresh key), force every summon in its drop pools, and max the level
  curves. This is fixed and identical in every download. See section 10.
- **Part B (per-pick):** the picker re-weights `summon_lot.tbl` in place so each
  summon's chosen skill and bonus win their pool's roll. This is a tiny byte patch,
  no toolchain required. See section 11.

---

## 4. The GBFR drop pipeline

```
quest reward
  \- reward_summon_lot   (Key -> SummonId, Weight)          WHICH summon drops
       \- summon.tbl     (Key -> SummonLotId1..4, SummonParamId)
            |- SummonLotId1 -> summon_lot   (skill pool)     WHICH skill it rolls
            \- SummonLotId3 -> summon_lot   (equip pool)     WHICH equip bonus
                 \- each summon_lot row -> SummonCurveId -> summon_curve   the LEVEL
                      - skills resolve via skill/text tables -> display name
                      - equip bonuses -> summon_base_param -> per-level value
```

Forcing anything = biasing the `Weight` column of the relevant pool.

---

## 5. Table formats

All tables are fixed-width binary. Multi-byte fields are **little-endian**. Hash
fields hold a 4-byte value shown in tools as uppercase hex - e.g. the text
`439EA421` is the `u32` `0x439EA421`, stored as bytes `21 A4 9E 43`.

### 5.1 `summon_lot.tbl` - the one the picker edits

```
header: 8 bytes  ->  int32 rowCount @0x00 , int32 0 @0x04
then rowCount rows of 20 bytes each:

  offset  field
  +0x00   Key                (pool id hash)
  +0x04   SkillId / BaseParamId hash   (skill pools hold skill hashes;
                                        equip pools hold base-param hashes)
  +0x08   SummonCurveId      (hash -> summon_curve)
  +0x0C   Weight             (int32)   <- the only field the picker changes
  +0x10   Unk5               (=0)
```

File length check: `8 + rowCount*20`. The shipped (split-pool) table has 814 rows
-> 15,848 bytes.

### 5.2 `summon.tbl`

Fixed rows. Columns: `SummonLotId1, SummonLotId2, SummonLotId3, SummonLotId4, Key,
SummonParamId, Rarity, SortOrderMaybe, Unk9`. **SummonLotId1 = skill pool**,
**SummonLotId3 = equip pool** (SummonLotId2/4 empty for these summons).

### 5.3 `summon_curve.tbl`

Columns: `Key, SkillOrBaseParamLevel, Weight`. Forcing max level = keep the top
level row, set the others in that curve to `1`.

### 5.4 `reward_summon_lot.tbl`

Columns: `Key, SummonId, Weight, Unk4`. Forcing a summon = keep its row weight, set
the other rows in that pool to `1`.

### 5.5 `summon_base_param.tbl` (the equip-bonus values)

The table converter yields 0 rows for this one - parse it raw:

```
header: 8 bytes (byte0 = row count = 23)
then 23 rows of 64 bytes each:

  +0x00 .. +0x24   Level1Value .. Level10Value   (10 x float32)
  +0x28   Key            (base-param id hash)
  +0x2C   NameHash       (hash of a TXT_ token)
  +0x30   Desc
  +0x34   ParamID
  +0x38   ValueDisplayMultiplier   (int32)
  +0x3C   Unk16
```

Displayed value = `LevelN Value x ValueDisplayMultiplier`. There is an **off-by-one**
between curve level and value index: curve "level N" reads `Level(N+1)Value`. Both
equip curves are forced to level 9 -> they read `Level10Value` -> the **max**. So the
in-game max value for a bonus = `Level10Value x ValueDisplayMultiplier` (see section 8).

---

## 6. The XXHash32Custom hash

Skill/pool ids in the raw tables are `XXHash32Custom` of an ASCII string
(e.g. `hash("SKILL_044_00")`). It's XXHash32 with a **custom seed `0x178A54A4`** and,
for inputs >= 16 bytes, four hardcoded lane seeds instead of the standard ones:

```
v1 = 0x2557311B
v2 = 0x871FB76A
v3 = 0x0133ECF3
v4 = 0x62FC7342
```

Everything else (the rounds, the avalanche) is standard XXHash32. Validation vector:
`hash("RW_408311_100") = 0xD1734A36`.

The picker does **not** need this at runtime - all the skill hashes are precomputed
(section 9), so it only ever compares raw `u32`s.

---

## 7. Reference data

### 7.1 Summons

`orig equip pool` is the shared game pool; `new equip pool` is the per-summon clone
this mod creates (a `XXHash32Custom("GBFRER_EQUIP_<summon>")`, collision-checked).

| Summon | summonId | skill pool (SummonLotId1) | orig equip pool | **new equip pool** | equip curve | drop pools |
|---|---|---|---|---|---|---|
| Behemoth III | `E4B7DCF9` | `DF902143` | `439EA421` | `393EF1D8` | `2E2C5483` | `880B970F,08D88628,2DF630CD,AF0E103A` |
| Wee Pincer III | `F28D0E35` | `2340DCED` | `439EA421` | `C39F7144` | `2E2C5483` | `F5F9C753,6CEBFF62,107EA7DA,9C4BCADB` |
| Lucilius | `6E5968FC` | `CC0B0EF1` | `48727C41` | `F63793E4` | `4E547493` | `32736ED4,362668B5,8588DE4E,C1B77E4B` |
| Beelzebub | `A7EFF558` | `BD452CC9` | `48727C41` | `B7D9379B` | `4E547493` | `C1B6CEFD,E0DA96E4,E94F68CA,F457FE73` |
| Rolan | `0F986ED9` | `26428274` | `48727C41` | `A35D7A5C` | `4E547493` | `24A7E2BD,347FFFE6,3B8629F4,C23CC67F` |
| Lilith | `DFAB70B7` | `05205336` | `48727C41` | `DCB2B22B` | `4E547493` | `8C8F07CF,B44CD878` |
| Albacore III | `B44A8A02` | `1250EFD2` | `439EA421` | `E31F7BAD` | `2E2C5483` | `24276521` |
| Furycane Nihilla | `5C42D57C` | `384C3333` | `439EA421` | `FD5DACF0` | `2E2C5483` | `D322AE46,472F4116,EF6AE9A9,3D4FFD5A` |

No two summons may claim the same drop pool. A pool forces exactly one summon, so
an overlap would make them cancel each other out. The build fails if that happens.

### 7.3 Astral cap curves

Astral caps scale `20,25,30,35,40,45,50,60,80,100` across the value table, and the
astral curve spans levels 5-9, so level 6 lands on exactly the normal-tier maxima
(50% caps, Stun 15, Attack +2000, Health +2000, Crit 20, Healing 50). Both are rolls
the game can produce, so either choice stays legit.

| Curve | Level | Result |
|---|---|---|
| `4E547493` | 9 | astral maximum (100% caps) |
| `5A370B86` | 6 | normal-tier values (50% caps) |

The picker switches a summon between them by writing the curve id into its equip
pool rows (`+0x08`), so only `summon_lot.tbl` is ever patched.

### 7.2 Curves

| Curve | Used for | Levels | Forced to |
|---|---|---|---|
| `2E2C5483` | normal-tier equip | 6-9 | 9 (max) |
| `4E547493` | astral-tier equip | 5-9 | 9 (max) |
| `63E658AB` | signature skill | single level 15 | already max (no edit) |
| `8E4A0E03` | non-signature skills | 11-15 | 15 (max) |

Forcing `8E4A0E03` -> 15 is why even non-signature picks (Lucilius Alpha/Beta/Gamma,
Beelzebub Supplementary DMG, etc.) come in at full level 15.

---

## 8. Equip bonuses

Every summon can take any of these 11. The picker uses the **normal id** for
Behemoth/Wee Pincer and the **astral id** for the four astrals. Values are the
verified in-game maxima (`Level10Value x ValueDisplayMultiplier`, checked against
`summon_base_param.tbl`).

| Bonus | key | Normal id | Normal max | Astral id | Astral max |
|---|---|---|---|---|---|
| Normal Attack Damage Cap Up | `normatkcap` | `A66241C9` | 50% | `9245DFA4` | 100% |
| Skill Damage Cap Up | `skilldmgcap` | `2FFB509F` | 50% | `CE70C58A` | 100% |
| Skybound Art Damage Cap Up | `skyartcap` | `A3539FBB` | 50% | `5A1D2C89` | 100% |
| Healing Cap Up | `healcap` | `2270BC40` | 50% | `2EA9CA80` | 75% |
| Stun Power Up | `stun` | `F7B0316F` | 15 | `F0F77BC1` | 20 |
| Attack Power Up | `atk` | `A8900C80` | +2000 | `A3E537B1` | +3000 |
| Health Up | `hp` | `BC4E92CB` | +2000 | `F2256E44` | +5000 |
| Critical Hit Rate Up | `crit` | `00D171E0` | 20% | `A074A967` | 30% |
| Skill Damage Up | `skilldmg` | `664E8E32` | 20% | `AF39A3FB` | 30% |
| Skybound Art Damage Up | `skyart` | `75138D21` | 20% | `33C0D50C` | 30% |
| Chain Burst Damage Up | `chainburst` | `5A39D81B` | 50% | `54B09A37` | 100% |

Notes:
- **Stun Power Up** is a flat integer, not a percent - its base-param has
  `ValueDisplayMultiplier = 10` (1.5 -> 15, 2.0 -> 20). Everything else is x1.
- **Healing Cap Up** astral is 75%, not 100% - the only cap that isn't 100% on astrals.

---

## 9. Full skill pools

Each entry is `Display name - SKILL_id - picker hash`. * = signature (rolls on curve
`63E658AB`); the rest roll on `8E4A0E03` (forced to level 15). The three "Celestial"
auras are stored in the tables as raw hashes (no `SKILL_` string), reverse-mapped by
brute-forcing `hash("SKILL_nnn_00")`.

**Behemoth III - pool `DF902143`**
- * Stout Heart - SKILL_044_00 - `A1A8E39D`
- Supplementary DMG - SKILL_151_00 - `57AB5B10`
- Uplift - SKILL_072_00 - `B5FF9FD3`
- Celestial Ventus - SKILL_326_00 - `73220725`
- Less Is More - SKILL_152_00 - `82CE278D`
- Critical Hit Rate - SKILL_003_00 - `8D78A19B`

**Wee Pincer III - pool `2340DCED`**
- * Crabvestment Returns - SKILL_141_00 - `1B0D9897`
- DMG Cap - SKILL_020_00 - `DC584F60`
- Cascade - SKILL_070_00 - `05F2ECDC`
- Steel Nerves - SKILL_096_00 - `1470F860`

**Lucilius - pool `CC0B0EF1`**
- Alpha - SKILL_160_00 - `DBE1D775`
- Beta - SKILL_161_00 - `8D2ADB6E`
- Gamma - SKILL_162_00 - `5C862E13`
- * Berserker Echo - SKILL_233_00 - `EE85CD1F`
- Tyranny - SKILL_027_00 - `71F11A9B`
- Celestial Terra - SKILL_322_00 - `9232DC17`

**Beelzebub - pool `BD452CC9`**
- * Spartan Echo - SKILL_234_00 - `3D8153A1`
- Supplementary DMG - SKILL_151_00 - `57AB5B10`
- DMG Cap - SKILL_020_00 - `DC584F60`
- Drain - SKILL_067_00 - `7CCFF74F`
- Celestial Lumen - SKILL_321_00 - `A7726190`
- Improved Guard - SKILL_060_00 - `0AA20846`

**Rolan - pool `26428274`**
- * War Elemental - SKILL_146_00 - `4C588C27`
- Uplift - SKILL_072_00 - `B5FF9FD3`
- Autorevive - SKILL_068_00 - `95F3FA86`
- Quick Cooldown - SKILL_069_00 - `318D12E9`
- Drain - SKILL_067_00 - `7CCFF74F`
- Aegis - SKILL_085_00 - `E0ABFDFE`

**Albacore III - pool `1250EFD2`**
- * Natural Defenses - SKILL_103_00 - `0EAD65E0`
- Path to Mastery - SKILL_147_00 - `5E422AE5`
- Rupie Tycoon - SKILL_080_00 - `C86F3082`
- Fast Learner - SKILL_079_00 - `F687C5EF`

**Furycane Nihilla - pool `384C3333`** (no signature skill, all on the shared curve)
- Celestial Aqua - SKILL_324_00 - `A898E283`
- Fatebreaker - SKILL_325_00 - `D029FE08`
- Potion Hoarder - SKILL_073_00 - `24883AF3`
- Guts - SKILL_045_00 - `E69A4694`
- Blight Resistance - SKILL_088_00 - `9702860F`

**Lilith - pool `05205336`**
- * War Elemental - SKILL_146_00 - `4C588C27`
- Uplift - SKILL_072_00 - `B5FF9FD3`
- Potion Hoarder - SKILL_073_00 - `24883AF3`
- Tyranny - SKILL_027_00 - `71F11A9B`
- Linked Together - SKILL_009_00 - `3FEC5F80`
- Improved Healing - SKILL_065_00 - `9389CC06`

---

## 10. Part A - baked base edits

Done once, identical in every download. Produces the four shipped tables
(`summon.tbl`, `summon_lot.tbl`, `reward_summon_lot.tbl`, `summon_curve.tbl`):

1. **Split the pools** - for each summon, set `summon.SummonLotId3` to its new key
   (section 7.1) and clone the 11 rows of its tier's equip pool into `summon_lot` under that
   new key (88 new rows total, `summon_lot` grows from 726 to 814 rows). The
   original shared pools stay intact for every other summon in the game.
2. **Force the drops** - in all 22 drop pools (`reward_summon_lot`), keep the target
   summon's weight and set competitors to `1`.
3. **Max the curves** (`summon_curve`) - `2E2C5483`->9, `4E547493`->9, `8E4A0E03`->15.

New pool keys are collision-checked against all 224 existing `summon_lot` pools and
against every summon's `SummonLotId1..4`.

---

## 11. Part B - the in-place binary patcher

For each summon the picker calls `Force(pool, keep)` twice - once on its skill pool,
once on its (new) equip pool. The rule: **in every row of `pool`, set `Weight` to 1
unless the row's id equals `keep`** (the chosen row keeps its original weight). This
is byte-for-byte identical to what the official table tool produces from an
`UPDATE ... SET Weight=1 WHERE Key=pool AND id<>keep`.

Rust (from `src/main.rs`):

```rust
// summon_lot.tbl: 8-byte header (int32 rowCount) then 20-byte rows:
//   +0 Key  +4 SkillId/BaseParamId  +8 CurveId  +12 Weight(i32)  +16 Unk5   (LE)
fn force_pool(bytes: &mut [u8], pool_hex: &str, keep_hex: &str) {
    let pool = u32::from_str_radix(pool_hex, 16).unwrap_or(0);
    let keep = u32::from_str_radix(keep_hex, 16).unwrap_or(0);
    let n = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    for r in 0..n {
        let off = 8 + r * 20;
        if off + 20 > bytes.len() { break; }
        let key = u32::from_le_bytes([bytes[off], bytes[off+1], bytes[off+2], bytes[off+3]]);
        if key == pool {
            let sid = u32::from_le_bytes([bytes[off+4], bytes[off+5], bytes[off+6], bytes[off+7]]);
            if sid != keep {
                bytes[off + 12..off + 16].copy_from_slice(&1i32.to_le_bytes());
            }
        }
    }
}
```

The PowerShell edition does the identical thing with `[BitConverter]` on the same
byte layout. Both patch a **pristine** copy of the split-pool `summon_lot.tbl` each
run, so re-picking never compounds.

---

## 12. The GUI app

Rust + [`eframe`/`egui`](https://github.com/emilk/egui) `0.35` (glow/OpenGL backend),
plus `rfd` (native file dialog), `serde_json` (local settings), and `winresource`
(embeds the icon at build time).

Key properties:

- **Fully offline.** No HTTP client, no sockets, no telemetry. Verified at the
  binary level: the exe imports **zero** network DLLs (`ws2_32`/`wininet`/`winhttp`).
- **Nothing to install.** The MSVC C runtime is statically linked
  (`.cargo/config.toml` sets `+crt-static`), so it needs no VC++ Redistributable and
  no .NET - every DLL it imports ships with Windows 10/11.
- **Self-contained.** The four base tables and `ModConfig.json` are embedded via
  `include_bytes!` / `include_str!`. On first run a bare exe writes its own
  `ModConfig.json` + default tables next to itself, so a single `.exe` is a complete
  mod.
- **Self-locating.** It writes `GBFR\data\system\table\*.tbl` next to itself.
- **Requirement check.** It reads Reloaded's `ReloadedII.json`
  (`%APPDATA%\Reloaded-Mod-Loader-II\`) to find the Mods folder and verifies
  `gbfrelink.utility.manager` **and its dependency chain**
  (`Reloaded.Memory.SigScan.ReloadedII`, `reloaded.sharedlib.hooks`,
  `reloaded.universal.redirector`) are installed - shown as a banner. (Those three
  belong to the loader, not to this mod; this mod only declares
  `gbfrelink.utility.manager`.)
- **Run Game.** "Apply & Run Game" launches `Reloaded-II.exe --launch "<game exe>"`,
  auto-detected from the same config; otherwise it saves and tells you to launch via
  Reloaded-II yourself.
- **`--apply` mode.** Run headless with `--apply` to re-apply saved picks without a
  window (used for scripting/tests).

Settings persist to `picker_settings.json` next to the exe. All JSON reads strip a
leading UTF-8 BOM (`\u{feff}`), because .NET-written configs (like Reloaded's) can
carry one and `serde_json` rejects it.

---

## 13. Building from source

Requires a [Rust toolchain](https://rustup.rs/) (built with 1.96, MSVC target on
Windows).

```sh
cargo build --release
```

Output: `target/release/egui_picker.exe` (shipped as `GBFRER Summon Picker.exe`).
`.cargo/config.toml` forces the static CRT, so the build is standalone. Crate
versions are pinned in `Cargo.lock`. Run the logic tests with `cargo test`.

Repo layout:

| Path | Purpose |
|---|---|
| `src/main.rs` | the whole app: UI, `force_pool`, requirement check, Reloaded launch |
| `base/*.tbl` | the mod's base (split-pool) tables, embedded into the exe |
| `dist/ModConfig.json` | Reloaded-II manifest (also embedded for self-install) |
| `build.rs`, `icon.ico`, `icon_256.rgba` | window/exe icon |
| `.cargo/config.toml` | `+crt-static` |

---

## 14. How it was verified

- **Patcher correctness.** The binary `force_pool` output was diffed byte-for-byte
  against the official tool's `tbl-to-sqlite -> UPDATE -> sqlite-to-tbl` result - 0
  bytes differ, for both skill pools and equip pools.
- **Round-trip.** `tbl-to-sqlite -> sqlite-to-tbl` on the base table is byte-identical,
  confirming the fixed-row layout in section 5.1.
- **Values.** Every value in section 8 was recomputed from the raw `summon_base_param.tbl`
  (`Level10Value x ValueDisplayMultiplier`) and matched the picker's display.
- **Skill hashes.** All hashes in section 9 were confirmed present in their pool's raw
  bytes, so a pick can never silently force nothing.
- **End-to-end.** From a freshly-extracted zip, applying an independent config for
  all eight summons produced exactly one kept row per pool (chosen row's weight) with
  every competitor at `1`, for all 12 skill+equip pools.

---

## 15. Credits

- **Nenkai** - GBFRDataTools (archive/table tools + hashing) and the Relink modding
  documentation.
- **WistfulHopes** - `gbfrelink.utility.manager` (the mod loader).
- **Sewer56 / Reloaded-Project** - Reloaded-II.
- **AlphaSatanOmega, SheItoon, Hazelberry** - path-finding / batch-script research.
- Built with **egui / eframe**.

Data-file edits only - no memory editing, no code injection, no network.
