# bindanalyzer

Cheatsheet and analyzer for Hyprland key binds, in the terminal. Written in Rust.

Meant to live in a Hyprland *special workspace*: start it once, keep it in the
background, and it reloads itself whenever your configuration changes.

[Versión en castellano](README.es.md)

## What it does

- Reads `hyprland.conf` and every file it `source`s, resolving `$variables`,
  comments, `submap`, `bind`, `bindm`, `binde`, `bindd`... and `#TAGS:` markers.
- Queries `hyprctl binds -j` to know what Hyprland has actually loaded, and
  flags binds that are in the file but not loaded (yellow) or loaded but not in
  the file (gray).
- Views:
  - **Cheatsheet**: table filterable by tags.
  - **Search** (`/`): by text, by application, or by key combination (`super shift t`).
  - **Free keys** (`f`): which keys are still free for each modifier combination.
  - **Free combos** (`l`): for a given key, which modifier combinations are free.
  - **Conflicts** (`c`): repeated combinations in the same submap.
- `--json` dumps everything for scripts, wofi, rofi, etc.
- Interface in English or Spanish (`--lang`, or automatic from `LANG`).

## Tags

Add comments to your `hyprland.conf`. Binds following a `#TAGS:` line inherit
those tags until the next one. An empty `#TAGS:` line clears them. Binds
without tags show up under `(untagged)`.

```
#TAGS: apps main
bind = $mainMod, F1, exec, firefox
bind = $mainMod, F2, exec, thunderbird

#TAGS: workspaces
bind = $mainMod, 1, workspace, 1
#TAGS:
```

## Build and install

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

## License

GPL-3.0-or-later. Use at your own risk.
