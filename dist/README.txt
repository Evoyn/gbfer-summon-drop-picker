========================================================================
  GBFRER Summon Drop Picker - GUI edition           by Nebu
  Granblue Fantasy: Relink  (Endless Ragnarok, v2.0.2)         v2.1.0
========================================================================

  Mod page: https://www.nexusmods.com/granbluefantasyrelink/mods/677

A small Windows app (egui) that forces ALL EIGHT boss summons to drop at once,
each from its own quest, and lets you set every summon's SKILL and EQUIP BONUS
independently, then hit Apply (and optionally Run Game). Astral summons can be
capped to 50% instead of 100% if the max is more than you want.

  Summons: Behemoth III, Wee Pincer III, Albacore III, Furycane Nihilla,
           Lucilius, Beelzebub, Rolan, Lilith.

  * No install, nothing bundled to set up - it's a single .exe.
  * 100% OFFLINE. The app never touches the internet.
  * It only edits this mod's own table file (summon_lot.tbl) in place.

>>> SOLO / OFFLINE ONLY. Do NOT use in public multiplayer matchmaking. <<<

------------------------------------------------------------------------
REQUIREMENTS
------------------------------------------------------------------------
  * Reloaded-II ................. https://github.com/Reloaded-Project/Reloaded-II
  * gbfrelink.utility.manager ... https://www.nexusmods.com/granbluefantasyrelink/mods/526
  * Game version 2.0.2 (Endless Ragnarok).

------------------------------------------------------------------------
INSTALL  (two ways - either works)
------------------------------------------------------------------------
  A) Full folder / zip:
     1. Copy the whole "gbfrelink.summon.picker" folder into your Reloaded-II
        "Mods" folder (or drag this zip into Reloaded-II).
     2. In Reloaded-II tick "GBFRER Summon Drop Picker" and
        "gbfrelink.utility.manager". Untick any other summon-drop mod.

  B) Just the .exe:
     The app installs itself. Make a new empty folder inside Reloaded-II's
     "Mods" folder (any name), drop "GBFRER Summon Picker.exe" in it, and run
     it once. It creates its own ModConfig.json + tables next to itself. Then
     in Reloaded-II click the refresh/reload button, tick the mod (and
     "gbfrelink.utility.manager"), and you're set.

------------------------------------------------------------------------
HOW TO USE
------------------------------------------------------------------------
  1. Double-click  "GBFRER Summon Picker.exe"  (inside this mod folder).
  2. For each summon choose a Skill and an Equip Bonus (values are each
     summon's legit max).
  3. Click "Apply Picks".  (Or "Apply & Run Game" to apply and launch the
     game through Reloaded-II in one step.)
  4. If you had the game open, restart it - tables are read at startup.

  The Run Game button auto-detects Reloaded-II and the game from Reloaded's
  own settings. If it can't find them, open the "Reloaded-II / game paths"
  section and Browse to them once (it remembers). If Reloaded still isn't
  found, your picks are still saved - just launch the game via Reloaded-II
  yourself.

------------------------------------------------------------------------
REMOVING THE MOD
------------------------------------------------------------------------
  Just disabling the mod can leave the forced drops in place. In the app,
  open "Removing the mod", click "Restore vanilla tables", then start the
  game once with the mod still ticked. After that, untick or delete the mod.

------------------------------------------------------------------------
"Windows protected your PC" / antivirus warning
------------------------------------------------------------------------
  The .exe is not code-signed, so Windows SmartScreen may warn the first
  time ("unknown publisher"). Click "More info" -> "Run anyway". It's a tiny
  offline app that only rewrites this mod's summon_lot.tbl and (optionally)
  starts Reloaded-II.

  The full source code is public - read it or build it yourself:
      https://github.com/Evoyn/gbfer-summon-drop-picker

------------------------------------------------------------------------
CREDITS
------------------------------------------------------------------------
  * Nenkai ....... GBFRDataTools (table tools + hashing) & Relink modding docs.
  * WistfulHopes . gbfrelink.utility.manager (the loader).
  * Sewer56 / Reloaded-Project ... Reloaded-II.
  * Built with egui/eframe (Emil Ernerfeldt & contributors).

  Data-file edits only - no memory editing, no code injection, no network.
