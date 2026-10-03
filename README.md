# bindanalyzer

Cheatsheet and analyzer for Hyprland key binds, in the terminal. Written in Rust.

Meant to live in a Hyprland *special workspace*: start it once, keep it in the
background, and it reloads itself whenever your configuration changes.

[Versión en castellano](README.es.md)

## Supported configurations

| Hyprland | Config file | Support |
|---|---|---|
| 0.55 and later | `hyprland.lua` | Yes. Executed in an embedded Lua interpreter, see below. |
| any | `hyprland.conf` (hyprlang, deprecated since 0.55) | Yes. Kept for people who have not migrated. |

If both files exist `hyprland.lua` wins, exactly as in Hyprland. `-c file`
forces a file and picks the parser by extension. The `hyprctl` integration
works with both.

## What it does

- Reads `hyprland.lua` (Hyprland 0.55+) by running it in an embedded Lua
  interpreter with a fake `hl` table, so variables, loops, `require`,
  `hl.define_submap` and bind options all resolve exactly as Hyprland sees them.
  `hl.dsp.*` dispatchers are rendered as readable actions (`exec firefox`,
  `window.close`, `focus workspace=3`); plain Lua functions show as `lua function`,
  or as their `description` option if given.
- Still reads the legacy `hyprland.conf` and every file it `source`s, resolving
  `$variables`, comments, `submap`, `bind`, `bindm`, `binde`, `bindd`...
  `hyprland.lua` wins when both exist, as in Hyprland; `-c file` picks by extension.
- Queries `hyprctl binds -j` to know what Hyprland has actually loaded, and
  flags binds that are in the file but not loaded (yellow) or loaded but not in
  the file (gray). With a Lua config Hyprland only reports `__lua` callbacks,
  so the action text always comes from the file.
- Views:
  - **Cheatsheet**: table filterable by tags.
  - **Search** (`/`): by text, by application, or by key combination (`super shift t`).
  - **Free keys** (`f`): which keys are still free for each modifier combination.
  - **Free combos** (`l`): for a given key, which modifier combinations are free.
  - **Conflicts** (`c`): repeated combinations in the same submap.
- `--json` dumps everything for scripts, wofi, rofi, etc.
- Interface in English or Spanish (`--lang`, or automatic from `LANG`).

## Tags

Add comments to your config. Binds following a `TAGS:` comment inherit those
tags until the next one. An empty `TAGS:` comment clears them. Binds without
tags show up under `(untagged)`. Files loaded with `require` or `source` keep
their own tags.

```lua
--TAGS: apps main
hl.bind(mainMod .. " + F1", hl.dsp.exec_cmd("firefox"))
hl.bind(mainMod .. " + F2", hl.dsp.exec_cmd("thunderbird"))

--TAGS: workspaces
for i = 1, 9 do
  hl.bind(mainMod .. " + " .. i, hl.dsp.focus({ workspace = tostring(i) }))
end
--TAGS:
```

In the legacy format the marker is `#TAGS:`:

```
#TAGS: apps main
bind = $mainMod, F1, exec, firefox
#TAGS:
```

Note on keycode binds (`code:36`): with a Lua config, Hyprland 0.56 reports
them through `hyprctl` with an empty key, although they work. bindanalyzer
pairs them with the file by registration order.

## Build and install

Needs a C compiler for the embedded Lua (gcc or clang).

```
cargo build --release
install -Dm755 target/release/bindanalyzer ~/.local/bin/bindanalyzer
```

## Usage

```
bindanalyzer                      # cheatsheet with hyprctl
bindanalyzer --compact            # fewer columns
bindanalyzer --view search        # start searching
bindanalyzer --no-live            # file only, no hyprctl
bindanalyzer --lang es            # Spanish interface
bindanalyzer -c other.conf --json # JSON dump of another file
```

Example setup as a cheatsheet in a Hyprland special workspace. `foot -a` sets
the window class, and the windowrule sends it to the special workspace as soon
as it appears. The second bind relaunches the cheatsheet if you close it with `q`.

```
exec-once = foot -a bindanalyzer -T bindanalyzer -e bindanalyzer --compact
windowrule = match:class ^(bindanalyzer)$, workspace special:bindanalyzer silent

bind = $mainMod, U, togglespecialworkspace, bindanalyzer
bind = $mainMod CTRL, U, exec, foot -a bindanalyzer -T bindanalyzer -e bindanalyzer --compact
```

`hyprctl reload` does not run `exec-once` again; launch the command once by
hand or use the relaunch bind.

### Keys

| Key | Action |
|---|---|
| `/` | search; `Tab` switches the field (all, app, key); `Esc` or `Enter` leaves the input |
| `f` | free keys; `←/→` or `n/p` changes the combination; `s` changes submap |
| `l` | free combos for a key |
| `c` | conflicts |
| `1` / `Esc` | back to the cheatsheet |
| `t` / `Tab` | tags panel: `space` toggles, `a` all, `n` none, `o` only this one |
| `T` | show or hide the tags panel |
| `m` | compact mode |
| `j/k`, `↑/↓`, `g/G`, `PgUp/PgDn` | move |
| `Enter` | bind detail (original line, file, flags, state) |
| `r` | reload (also automatic when the file changes) |
| `?` | help |
| `q` | quit |

## Layout

- `crates/core`: model, config parser, `hyprctl` reader, merge and queries. No UI dependencies.
- `crates/tui`: the ratatui interface and the `bindanalyzer` binary.

Tests: `cargo test`. To see the views rendered with your real config:

```
BINDANALYZER_CONFIG=~/.config/hypr/hyprland.conf cargo test -p bindanalyzer -- --ignored --nocapture real_config_frames
```

## Credits

This code was written with Claude Fable 5.1, Anthropic's AI model, working alongside the author.

## License

GPL-3.0-or-later. Use at your own risk.
