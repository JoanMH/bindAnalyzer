//! Parser del fichero de configuración de Hyprland.
//!
//! Solo se interesa por lo que afecta a los binds: variables `$x = ...`,
//! `source = ...`, `submap = ...`, `bind[flags] = ...` y los comentarios
//! `#TAGS: a b c` que usa este programa para agrupar atajos.

use crate::model::{Bind, BindFlags, Key, ModMask, Origin, SourceRef};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Marcador de tags dentro de los comentarios de la config.
pub const TAGS_MARKER: &str = "#TAGS:";
const MAX_SOURCE_DEPTH: usize = 16;

#[derive(Debug, Default)]
pub struct ParsedConfig {
    pub binds: Vec<Bind>,
    /// Ficheros leídos, en orden, incluyendo los `source`.
    pub files: Vec<PathBuf>,
    pub warnings: Vec<String>,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read {0}: {1}")]
    Read(PathBuf, std::io::Error),
}

/// Lee y analiza un fichero de configuración y todos los que incluya.
pub fn parse_config(path: &Path) -> Result<ParsedConfig, ConfigError> {
    let text =
        std::fs::read_to_string(path).map_err(|e| ConfigError::Read(path.to_path_buf(), e))?;
    let mut p = Parser::default();
    p.parse_text(path, &text, 0);
    Ok(p.out)
}

/// Analiza un texto ya cargado. `path` se usa como origen de los binds y para
/// resolver `source` relativos.
pub fn parse_str(text: &str, path: &Path) -> ParsedConfig {
    let mut p = Parser::default();
    p.parse_text(path, text, 0);
    p.out
}

/// Quita el comentario final de una línea. `##` es un `#` literal, como en Hyprland.
pub fn strip_comment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '#' {
            if chars.peek() == Some(&'#') {
                chars.next();
                out.push('#');
            } else {
                break;
            }
        } else {
            out.push(c);
        }
    }
    out.trim_end().to_string()
}

/// Expande `~` al principio de una ruta.
pub fn expand_home(s: &str) -> String {
    if let Some(rest) = s.strip_prefix('~') {
        if let Some(home) = std::env::var_os("HOME") {
            return format!("{}{}", home.to_string_lossy(), rest);
        }
    }
    s.to_string()
}

#[derive(Default)]
struct Parser {
    out: ParsedConfig,
    tags: Vec<String>,
    submap: String,
}

impl Parser {
    fn warn(&mut self, path: &Path, line: usize, msg: impl AsRef<str>) {
        self.out
            .warnings
            .push(format!("{}:{}: {}", path.display(), line, msg.as_ref()));
    }

    fn parse_text(&mut self, path: &Path, text: &str, depth: usize) {
        self.out.files.push(path.to_path_buf());
        for (i, raw) in text.lines().enumerate() {
            let line_no = i + 1;
            let trimmed = raw.trim();

            if let Some(rest) = trimmed.strip_prefix(TAGS_MARKER) {
                self.tags = rest.split_whitespace().map(String::from).collect();
                continue;
            }
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let code = strip_comment(trimmed);
            let Some((k, v)) = code.split_once('=') else {
                continue;
            };
            let k = k.trim();
            let v = v.trim();

            if let Some(var) = k.strip_prefix('$') {
                let value = self.expand(v);
                self.out.variables.insert(var.to_string(), value);
                continue;
            }

            let value = self.expand(v);
            match k {
                "source" => self.handle_source(&value, path, line_no, depth),
                "submap" => {
                    self.submap = if value == "reset" {
                        String::new()
                    } else {
                        value
                    }
                }
                _ if k.starts_with("bind") => self.handle_bind(k, &value, path, line_no, raw),
                _ => {}
            }
        }
    }

    /// Sustituye `$variable` por su valor, probando primero los nombres más largos.
    fn expand(&self, v: &str) -> String {
        if !v.contains('$') || self.out.variables.is_empty() {
            return v.to_string();
        }
        let mut names: Vec<&String> = self.out.variables.keys().collect();
        names.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
        let mut s = v.to_string();
        for n in names {
            let pat = format!("${n}");
            if s.contains(&pat) {
                s = s.replace(&pat, &self.out.variables[n]);
            }
        }
        s
    }

    fn handle_source(&mut self, value: &str, from: &Path, line: usize, depth: usize) {
        if depth >= MAX_SOURCE_DEPTH {
            self.warn(from, line, "too many nested source levels");
            return;
        }
        let expanded = expand_home(value);
        let base = from.parent().map(Path::to_path_buf).unwrap_or_default();
        let pattern = if Path::new(&expanded).is_relative() {
            base.join(&expanded).to_string_lossy().into_owned()
        } else {
            expanded
        };

        let paths: Vec<PathBuf> = if pattern.contains(['*', '?', '[']) {
            match glob::glob(&pattern) {
                Ok(it) => it.filter_map(Result::ok).collect(),
                Err(e) => {
                    self.warn(from, line, format!("invalid source pattern: {e}"));
                    return;
                }
            }
        } else {
            vec![PathBuf::from(pattern)]
        };

        if paths.is_empty() {
            self.warn(from, line, format!("source matches nothing: {value}"));
        }
        for p in paths {
            match std::fs::read_to_string(&p) {
                Ok(text) => {
                    let saved_tags = std::mem::take(&mut self.tags);
                    self.parse_text(&p, &text, depth + 1);
                    self.tags = saved_tags;
                }
                Err(e) => self.warn(from, line, format!("cannot read {}: {e}", p.display())),
            }
        }
    }

    fn handle_bind(&mut self, k: &str, value: &str, path: &Path, line: usize, raw: &str) {
        let (flags, unknown_flags) = BindFlags::from_letters(&k[4..]);
        if !unknown_flags.is_empty() {
            let letters: String = unknown_flags.iter().collect();
            self.warn(path, line, format!("unknown bind flags: {letters}"));
        }

        let nparts = if flags.description { 5 } else { 4 };
        let parts: Vec<&str> = value.splitn(nparts, ',').map(str::trim).collect();
        if parts.len() < nparts - 1 {
            self.warn(path, line, "incomplete bind, missing fields");
            return;
        }
        let (mods_s, key_s, desc, dispatcher, arg) = if flags.description {
            (parts[0], parts[1], parts[2], parts[3], parts.get(4).copied().unwrap_or(""))
        } else {
            (parts[0], parts[1], "", parts[2], parts.get(3).copied().unwrap_or(""))
        };

        let (mods, unknown_mods) = ModMask::from_names(mods_s);
        if !unknown_mods.is_empty() {
            self.warn(
                path,
                line,
                format!("unknown modifiers: {}", unknown_mods.join(" ")),
            );
        }
        if dispatcher.is_empty() {
            self.warn(path, line, "bind without dispatcher");
        }

        self.out.binds.push(Bind {
            mods,
            key: Key::parse(key_s),
            dispatcher: dispatcher.to_string(),
            arg: arg.to_string(),
            submap: self.submap.clone(),
            flags,
            description: desc.to_string(),
            tags: self.tags.clone(),
            source: Some(SourceRef {
                file: path.to_path_buf(),
                line,
                raw: raw.trim_end().to_string(),
            }),
            origin: Origin::Config,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r###"
$mainMod = SUPER
$term = kitty

#TAGS: displays
bind = SUPER CTRL, 0, exec, ~/.config/hypr/scripts/eDP-1-only.sh
bind =SUPER, C, resizeactive, exact 50% 50%
bind = , mouse:274, killactive,          # El click de la rueda
bindm=SUPER, mouse:272, movewindow
#TAGS: apps
bind = $mainMod, 36, exec, footclient
bind = $mainMod SHIFT, T, exec, $term --title "a, b"
bindd = $mainMod, Q, Cerrar ventana, killactive,
bind = $mainMod, P, pseudo, # dwindle
bind = $mainMod, H, exec, notify-send "##1"
#TAGS:
bind = $mainMod, R, submap, resize
submap=resize
binde=,right,resizeactive,50 0
bind = , Return, submap, reset
submap = reset
bind = $mainMod, Z, exec, tail
"###;

    fn parse() -> ParsedConfig {
        parse_str(SAMPLE, Path::new("/tmp/hyprland.conf"))
    }

    #[test]
    fn parses_every_bind_variant() {
        let p = parse();
        assert_eq!(p.binds.len(), 13, "warnings: {:?}", p.warnings);
        assert!(p.warnings.is_empty(), "{:?}", p.warnings);
    }

    #[test]
    fn strips_prefix_and_spaces() {
        let p = parse();
        let b = &p.binds[1];
        assert_eq!(b.mods, ModMask::SUPER);
        assert_eq!(b.key, Key::Sym("C".into()));
        assert_eq!(b.dispatcher, "resizeactive");
        assert_eq!(b.arg, "exact 50% 50%");
        assert_eq!(b.source.as_ref().unwrap().line, 7);
    }

    #[test]
    fn strips_trailing_comments_and_keeps_escaped_hash() {
        let p = parse();
        let b = &p.binds[2];
        assert_eq!(b.mods, ModMask::NONE);
        assert_eq!(b.key, Key::Sym("mouse:274".into()));
        assert_eq!(b.dispatcher, "killactive");
        assert_eq!(b.arg, "");
        let p_bind = p.binds.iter().find(|b| b.key == Key::Sym("P".into())).unwrap();
        assert_eq!(p_bind.arg, "");
        let h_bind = p.binds.iter().find(|b| b.key == Key::Sym("H".into())).unwrap();
        assert_eq!(h_bind.arg, "notify-send \"#1\"");
    }

    #[test]
    fn expands_variables_and_keeps_commas_in_args() {
        let p = parse();
        let t = p.binds.iter().find(|b| b.key == Key::Sym("T".into())).unwrap();
        assert_eq!(t.mods, ModMask::SUPER | ModMask::SHIFT);
        assert_eq!(t.arg, "kitty --title \"a, b\"");
    }

    #[test]
    fn keycodes_flags_and_descriptions() {
        let p = parse();
        let code = p.binds.iter().find(|b| b.key == Key::Code(36)).unwrap();
        assert_eq!(code.arg, "footclient");
        let m = &p.binds[3];
        assert!(m.flags.mouse);
        assert_eq!(m.key, Key::Sym("mouse:272".into()));
        let q = p.binds.iter().find(|b| b.key == Key::Sym("Q".into())).unwrap();
        assert!(q.flags.description);
        assert_eq!(q.description, "Cerrar ventana");
        assert_eq!(q.dispatcher, "killactive");
    }

    #[test]
    fn tags_and_submaps_follow_the_file() {
        let p = parse();
        assert_eq!(p.binds[0].tags, vec!["displays"]);
        assert_eq!(p.binds[4].tags, vec!["apps"]);
        let r = p.binds.iter().find(|b| b.key == Key::Sym("R".into())).unwrap();
        assert!(r.tags.is_empty());
        assert_eq!(r.submap, "");
        let right = p.binds.iter().find(|b| b.key == Key::Sym("right".into())).unwrap();
        assert_eq!(right.submap, "resize");
        assert!(right.flags.repeat);
        let z = p.binds.iter().find(|b| b.key == Key::Sym("Z".into())).unwrap();
        assert_eq!(z.submap, "");
    }

    #[test]
    fn follows_source_and_restores_tags() {
        let dir = std::env::temp_dir().join(format!("bindanalyzer-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let inc = dir.join("extra.conf");
        std::fs::write(&inc, "#TAGS: extra\nbind = SUPER, X, exec, foo\n").unwrap();
        let main = dir.join("hyprland.conf");
        std::fs::write(
            &main,
            format!(
                "#TAGS: main\nsource = {}\nbind = SUPER, Y, exec, bar\nsource = ./missing.conf\n",
                inc.display()
            ),
        )
        .unwrap();

        let p = parse_config(&main).unwrap();
        assert_eq!(p.files.len(), 2);
        assert_eq!(p.binds.len(), 2);
        assert_eq!(p.binds[0].tags, vec!["extra"]);
        assert_eq!(p.binds[0].source.as_ref().unwrap().file, inc);
        assert_eq!(p.binds[1].tags, vec!["main"]);
        assert_eq!(p.warnings.len(), 1, "{:?}", p.warnings);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn warns_on_unknown_modifier_and_incomplete_bind() {
        let p = parse_str("bind = $nope, Q, exec, x\nbind = SUPER, Q\n", Path::new("x.conf"));
        assert_eq!(p.binds.len(), 1);
        assert_eq!(p.warnings.len(), 2);
    }
}
