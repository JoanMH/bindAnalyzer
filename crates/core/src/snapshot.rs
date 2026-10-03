//! Carga y fusión de las dos fuentes: fichero de configuración y `hyprctl`.

use crate::config::{self, ConfigError, ParsedConfig};
use crate::lua;
use crate::hyprctl::{self, LiveError};
use crate::model::{Bind, BindId, Key, Origin};
use serde::Serialize;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct LoadOptions {
    pub config: PathBuf,
    /// Consultar `hyprctl binds -j` además del fichero.
    pub use_live: bool,
}

impl Default for LoadOptions {
    fn default() -> Self {
        LoadOptions {
            config: default_config_path(),
            use_live: true,
        }
    }
}

/// `hyprland.lua` si existe, si no `hyprland.conf`, bajo `$XDG_CONFIG_HOME/hypr`
/// o `~/.config/hypr`. Es el mismo orden que sigue Hyprland.
pub fn default_config_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("hypr");
    let lua = dir.join("hyprland.lua");
    if lua.exists() {
        lua
    } else {
        dir.join("hyprland.conf")
    }
}

/// Lee el fichero con el parser que corresponde a su extensión.
pub fn parse_any(path: &Path) -> Result<ParsedConfig, ConfigError> {
    if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("lua")) {
        lua::parse_lua(path)
    } else {
        config::parse_config(path)
    }
}

/// Estado de la consulta a `hyprctl`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum LiveStatus {
    /// El usuario pidió no consultar `hyprctl`.
    Disabled,
    /// No estamos dentro de una sesión de Hyprland.
    NotRunning,
    /// `hyprctl` falló.
    Error(String),
    Ok,
}

#[derive(Debug, Serialize)]
pub struct Snapshot {
    pub config_path: PathBuf,
    /// Binds del fichero en su orden, seguidos de los que solo conoce `hyprctl`.
    pub binds: Vec<Bind>,
    pub files: Vec<PathBuf>,
    pub warnings: Vec<String>,
    pub live: LiveStatus,
}

impl Snapshot {
    /// Tags distintos, ordenados.
    pub fn all_tags(&self) -> Vec<String> {
        let set: BTreeSet<&String> = self.binds.iter().flat_map(|b| b.tags.iter()).collect();
        set.into_iter().cloned().collect()
    }

    pub fn has_untagged(&self) -> bool {
        self.binds.iter().any(|b| b.tags.is_empty())
    }

    /// Submaps presentes; el global (cadena vacía) siempre va primero.
    pub fn submaps(&self) -> Vec<String> {
        let mut set: BTreeSet<String> = self.binds.iter().map(|b| b.submap.clone()).collect();
        set.remove("");
        let mut v = vec![String::new()];
        v.extend(set);
        v
    }

    /// Binds del fichero que Hyprland no tiene cargados. Solo tiene sentido si `live == Ok`.
    pub fn not_loaded_count(&self) -> usize {
        if self.live != LiveStatus::Ok {
            return 0;
        }
        self.binds.iter().filter(|b| b.origin == Origin::Config).count()
    }

    pub fn live_only_count(&self) -> usize {
        self.binds.iter().filter(|b| b.origin == Origin::Live).count()
    }

    /// Fecha de modificación más reciente entre todos los ficheros leídos.
    pub fn latest_mtime(&self) -> Option<SystemTime> {
        self.files
            .iter()
            .filter_map(|f| std::fs::metadata(f).ok()?.modified().ok())
            .max()
    }
}

/// Lee la configuración y, si procede, `hyprctl`, y fusiona ambas.
pub fn load(opts: &LoadOptions) -> Result<Snapshot, ConfigError> {
    let parsed = parse_any(&opts.config)?;
    let live = if opts.use_live {
        Some(hyprctl::live_binds())
    } else {
        None
    };
    Ok(build(&opts.config, parsed, live))
}

/// Construye el snapshot a partir de las piezas ya leídas. Útil para tests.
pub fn build(
    config_path: &Path,
    parsed: ParsedConfig,
    live: Option<Result<Vec<Bind>, LiveError>>,
) -> Snapshot {
    let mut warnings = parsed.warnings;
    let mut binds = parsed.binds;

    let status = match live {
        None => LiveStatus::Disabled,
        Some(Err(LiveError::NotRunning)) => LiveStatus::NotRunning,
        Some(Err(e)) => {
            warnings.push(format!("hyprctl: {e}"));
            LiveStatus::Error(e.to_string())
        }
        Some(Ok(live_binds)) => {
            merge(&mut binds, live_binds);
            LiveStatus::Ok
        }
    };

    Snapshot {
        config_path: config_path.to_path_buf(),
        binds,
        files: parsed.files,
        warnings,
        live: status,
    }
}

/// Empareja cada bind del fichero con uno de `hyprctl` por identidad.
/// Los del fichero sin pareja quedan como `Config`; los de `hyprctl` sin pareja
/// se añaden al final como `Live`.
///
/// Segunda pasada: con configuración Lua, `hyprctl` informa de los binds por
/// keycode (`code:NN`) con la tecla vacía aunque funcionan. Como lista los
/// binds en orden de registro, igual que el fichero, los sobrantes de
/// `hyprctl` sin tecla se emparejan en orden con los sobrantes del fichero
/// que tengan el mismo submap y modificadores.
fn merge(binds: &mut Vec<Bind>, live: Vec<Bind>) {
    let mut pool: HashMap<BindId, Vec<Bind>> = HashMap::new();
    for l in live {
        pool.entry(l.id()).or_default().push(l);
    }
    for b in binds.iter_mut() {
        if let Some(v) = pool.get_mut(&b.id()) {
            if let Some(l) = v.pop() {
                b.origin = Origin::Both;
                if b.description.is_empty() {
                    b.description = l.description;
                }
            }
        }
    }
    let mut rest: Vec<Bind> = pool.into_values().flatten().collect();
    rest.sort_by_key(|a| a.id());

    let mut keyless: Vec<Bind> = Vec::new();
    for l in rest.drain(..) {
        if l.key == Key::Sym(String::new()) {
            keyless.push(l);
        } else {
            binds.push(l);
        }
    }
    for b in binds.iter_mut().filter(|b| b.origin == Origin::Config) {
        let Some(pos) = keyless
            .iter()
            .position(|l| l.submap == b.submap && l.mods == b.mods)
        else {
            continue;
        };
        let l = keyless.remove(pos);
        b.origin = Origin::Both;
        if b.description.is_empty() {
            b.description = l.description;
        }
    }
    binds.extend(keyless);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hyprctl::parse_live_json;

    #[test]
    fn merge_marks_loaded_missing_and_live_only() {
        let parsed = config::parse_str(
            "#TAGS: a\nbind = SUPER, Q, killactive,\nbind = SUPER, 36, exec, foot\nbind = SUPER, N, exec, nuevo\n",
            Path::new("x.conf"),
        );
        let live = parse_live_json(
            br#"[{"modmask":64,"key":"Q","dispatcher":"killactive"},
                 {"modmask":64,"key":"","keycode":36,"dispatcher":"exec","arg":"foot"},
                 {"modmask":64,"key":"W","dispatcher":"exec","arg":"plugin"}]"#,
        )
        .unwrap();
        let s = build(Path::new("x.conf"), parsed, Some(Ok(live)));
        assert_eq!(s.live, LiveStatus::Ok);
        assert_eq!(s.binds.len(), 4);
        assert_eq!(s.binds[0].origin, Origin::Both);
        assert_eq!(s.binds[1].origin, Origin::Both);
        assert_eq!(s.binds[2].origin, Origin::Config);
        assert_eq!(s.binds[3].origin, Origin::Live);
        assert_eq!(s.not_loaded_count(), 1);
        assert_eq!(s.live_only_count(), 1);
        assert_eq!(s.all_tags(), vec!["a"]);
        assert!(s.has_untagged());
    }

    #[test]
    fn keyless_live_binds_pair_in_order() {
        let parsed = config::parse_str(
            "bind = SUPER, 36, exec, foot\nbind = , 122, exec, vol-\nbind = , 123, exec, vol+\nbind = SUPER, Q, killactive,\n",
            Path::new("x.conf"),
        );
        let live = parse_live_json(
            br#"[{"modmask":64,"key":"","dispatcher":"__lua","arg":"1"},
                 {"modmask":0,"key":"","dispatcher":"__lua","arg":"3"},
                 {"modmask":0,"key":"","dispatcher":"__lua","arg":"5"},
                 {"modmask":64,"key":"Q","dispatcher":"__lua","arg":"7"},
                 {"modmask":8,"key":"","dispatcher":"__lua","arg":"9"}]"#,
        )
        .unwrap();
        let s = build(Path::new("x.conf"), parsed, Some(Ok(live)));
        assert_eq!(s.binds.len(), 5);
        assert!(s.binds[..4].iter().all(|b| b.origin == Origin::Both), "{:?}", s.binds);
        assert_eq!(s.binds[1].action(), "exec vol-");
        assert_eq!(s.binds[4].origin, Origin::Live);
        assert_eq!(s.binds[4].action(), "lua callback #9");
        assert!(s.warnings.is_empty(), "{:?}", s.warnings);
    }

    #[test]
    fn without_hyprland_everything_is_config() {
        let parsed = config::parse_str("bind = SUPER, Q, killactive,\n", Path::new("x.conf"));
        let s = build(Path::new("x.conf"), parsed, Some(Err(LiveError::NotRunning)));
        assert_eq!(s.live, LiveStatus::NotRunning);
        assert_eq!(s.not_loaded_count(), 0);
        assert!(s.warnings.is_empty());
    }
}
