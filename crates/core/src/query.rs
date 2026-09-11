//! Consultas sobre una lista de binds: filtrado por tags, búsqueda, teclas
//! libres, combinaciones libres para una tecla y conflictos.
//!
//! Todas devuelven índices sobre el slice recibido para que la interfaz pueda
//! mapear filas a binds sin copiar.

use crate::model::{Bind, Key, ModMask};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::{HashMap, HashSet};

/// Sobre qué campos busca [`search`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchField {
    /// Combinación, acción, descripción, tags y submap.
    All,
    /// Solo el comando o argumento y la descripción: "qué atajo abre X".
    App,
    /// Solo la combinación de teclas, con sintaxis `super shift t`.
    Key,
}

impl SearchField {
    pub fn next(self) -> SearchField {
        match self {
            SearchField::All => SearchField::App,
            SearchField::App => SearchField::Key,
            SearchField::Key => SearchField::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SearchField::All => "todo",
            SearchField::App => "aplicación",
            SearchField::Key => "tecla",
        }
    }
}

/// Binds visibles según los tags activos. Los binds sin tag se incluyen si
/// `untagged` es `true`.
pub fn filter_by_tags(binds: &[Bind], active: &HashSet<String>, untagged: bool) -> Vec<usize> {
    binds
        .iter()
        .enumerate()
        .filter(|(_, b)| {
            if b.tags.is_empty() {
                untagged
            } else {
                b.tags.iter().any(|t| active.contains(t))
            }
        })
        .map(|(i, _)| i)
        .collect()
}

fn haystack(b: &Bind, field: SearchField) -> String {
    match field {
        SearchField::All => format!(
            "{} {} {} {} {} {}",
            b.combo(),
            b.dispatcher,
            b.arg,
            b.description,
            b.tags.join(" "),
            b.submap
        ),
        SearchField::App => format!("{} {}", b.arg, b.description),
        SearchField::Key => b.combo(),
    }
}

/// Búsqueda de texto. Primero se buscan coincidencias literales de cada
/// palabra (sin distinguir mayúsculas) y se devuelven en el orden del fichero;
/// si no hay ninguna, se recurre a búsqueda difusa ordenada por puntuación.
/// Consulta vacía: todos los binds en su orden.
pub fn search(binds: &[Bind], query: &str, field: SearchField) -> Vec<usize> {
    let q = query.trim();
    if q.is_empty() {
        return (0..binds.len()).collect();
    }
    if field == SearchField::Key {
        return key_search(binds, q);
    }
    let words: Vec<&str> = q.split_whitespace().collect();
    let lower_words: Vec<String> = words.iter().map(|w| w.to_lowercase()).collect();

    let literal: Vec<usize> = binds
        .iter()
        .enumerate()
        .filter(|(_, b)| {
            let hay = haystack(b, field).to_lowercase();
            lower_words.iter().all(|w| hay.contains(w.as_str()))
        })
        .map(|(i, _)| i)
        .collect();
    if !literal.is_empty() {
        return literal;
    }

    let matcher = SkimMatcherV2::default();
    let mut scored: Vec<(i64, usize)> = binds
        .iter()
        .enumerate()
        .filter_map(|(i, b)| {
            let hay = haystack(b, field);
            let mut total = 0;
            for w in &words {
                total += matcher.fuzzy_match(&hay, w)?;
            }
            Some((total, i))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().map(|(_, i)| i).collect()
}

/// Búsqueda por combinación: `super shift t`, `super+t`, `t`, `super`.
/// Los modificadores dados deben estar en el bind; la tecla, si se indica,
/// debe coincidir o ser prefijo. Las coincidencias exactas van primero.
pub fn key_search(binds: &[Bind], query: &str) -> Vec<usize> {
    let mut mods = ModMask::NONE;
    let mut key_tokens: Vec<String> = Vec::new();
    for tok in query
        .split(|c: char| c.is_whitespace() || c == '+')
        .filter(|t| !t.is_empty())
    {
        match ModMask::parse_one(tok) {
            Some(m) => mods = mods | m,
            None => key_tokens.push(tok.to_lowercase()),
        }
    }
    let key = key_tokens.join(" ");

    let mut scored: Vec<(i32, usize)> = binds
        .iter()
        .enumerate()
        .filter_map(|(i, b)| {
            if !b.mods.contains(mods) {
                return None;
            }
            let mut score = if b.mods == mods { 10 } else { 0 };
            if !key.is_empty() {
                let name = b.key.sym_name().unwrap_or_else(|| b.key.norm());
                score += if name == key {
                    2
                } else if name.starts_with(&key) {
                    1
                } else {
                    return None;
                };
            }
            Some((score, i))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().map(|(_, i)| i).collect()
}

/// Grupo de teclas del "universo" que se considera para las teclas libres.
pub struct KeyGroup {
    pub name: &'static str,
    pub keys: &'static [&'static str],
}

pub const KEY_GROUPS: &[KeyGroup] = &[
    KeyGroup {
        name: "Letras",
        keys: &[
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q",
            "r", "s", "t", "u", "v", "w", "x", "y", "z",
        ],
    },
    KeyGroup {
        name: "Números",
        keys: &["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
    },
    KeyGroup {
        name: "Función",
        keys: &[
            "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12",
        ],
    },
    KeyGroup {
        name: "Navegación",
        keys: &[
            "up", "down", "left", "right", "Home", "End", "Prior", "Next", "Insert", "Delete",
        ],
    },
    KeyGroup {
        name: "Especiales",
        keys: &["Return", "space", "Tab", "Escape", "BackSpace", "Print"],
    },
    KeyGroup {
        name: "Símbolos",
        keys: &[
            "grave",
            "minus",
            "equal",
            "bracketleft",
            "bracketright",
            "backslash",
            "semicolon",
            "apostrophe",
            "comma",
            "period",
            "slash",
        ],
    },
];

/// `true` si la tecla forma parte de algún grupo del universo.
pub fn in_universe(key_lower: &str) -> bool {
    KEY_GROUPS
        .iter()
        .flat_map(|g| g.keys.iter())
        .any(|k| k.eq_ignore_ascii_case(key_lower))
}

/// Índice del primer bind que ocupa `mods + key` en `submap`. Un bind
/// `catchall` con esos modificadores ocupa cualquier tecla.
pub fn bound_by(binds: &[Bind], mods: ModMask, key_lower: &str, submap: &str) -> Option<usize> {
    binds.iter().position(|b| {
        b.submap == submap
            && b.mods == mods
            && match &b.key {
                Key::CatchAll => true,
                k => k.sym_name().as_deref() == Some(key_lower),
            }
    })
}

pub struct KeyUsage {
    pub key: &'static str,
    pub bound_by: Option<usize>,
}

pub struct GroupReport {
    pub name: &'static str,
    pub keys: Vec<KeyUsage>,
}

impl GroupReport {
    pub fn free_count(&self) -> usize {
        self.keys.iter().filter(|k| k.bound_by.is_none()).count()
    }
}

/// Estado de cada tecla del universo para una combinación de modificadores.
pub fn free_keys(binds: &[Bind], mods: ModMask, submap: &str) -> Vec<GroupReport> {
    KEY_GROUPS
        .iter()
        .map(|g| GroupReport {
            name: g.name,
            keys: g
                .keys
                .iter()
                .map(|k| KeyUsage {
                    key: k,
                    bound_by: bound_by(binds, mods, &k.to_lowercase(), submap),
                })
                .collect(),
        })
        .collect()
}

/// Binds del submap con esos modificadores cuya tecla no está en el universo
/// (teclas multimedia, ratón, keycodes desconocidos).
pub fn other_bound(binds: &[Bind], mods: ModMask, submap: &str) -> Vec<usize> {
    binds
        .iter()
        .enumerate()
        .filter(|(_, b)| {
            b.submap == submap
                && b.mods == mods
                && match b.key.sym_name() {
                    Some(n) => !in_universe(&n),
                    None => true,
                }
        })
        .map(|(i, _)| i)
        .collect()
}

/// Combinaciones de modificadores usadas en el submap, con su frecuencia,
/// de más a menos usada.
pub fn used_modmasks(binds: &[Bind], submap: &str) -> Vec<(ModMask, usize)> {
    let mut counts: HashMap<ModMask, usize> = HashMap::new();
    for b in binds.iter().filter(|b| b.submap == submap) {
        *counts.entry(b.mods).or_default() += 1;
    }
    let mut v: Vec<(ModMask, usize)> = counts.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    v
}

/// Combinaciones habituales alrededor de SUPER.
pub const STANDARD_COMBOS: [ModMask; 7] = [
    ModMask::SUPER,
    ModMask(ModMask::SUPER.0 | ModMask::SHIFT.0),
    ModMask(ModMask::SUPER.0 | ModMask::CTRL.0),
    ModMask(ModMask::SUPER.0 | ModMask::ALT.0),
    ModMask(ModMask::SUPER.0 | ModMask::CTRL.0 | ModMask::SHIFT.0),
    ModMask(ModMask::SUPER.0 | ModMask::CTRL.0 | ModMask::ALT.0),
    ModMask(ModMask::SUPER.0 | ModMask::SHIFT.0 | ModMask::ALT.0),
];

/// Combinaciones a ofrecer: primero las que ya se usan en el submap, por
/// frecuencia, y después las estándar que falten.
pub fn combo_candidates(binds: &[Bind], submap: &str) -> Vec<ModMask> {
    let mut v: Vec<ModMask> = used_modmasks(binds, submap)
        .into_iter()
        .map(|(m, _)| m)
        .collect();
    for m in STANDARD_COMBOS {
        if !v.contains(&m) {
            v.push(m);
        }
    }
    v
}

pub struct ComboUsage {
    pub mods: ModMask,
    pub bound_by: Option<usize>,
}

/// Para una tecla, qué combinaciones están libres y cuáles ocupadas.
pub fn free_combos(binds: &[Bind], key: &str, submap: &str, combos: &[ModMask]) -> Vec<ComboUsage> {
    let k = Key::parse(key)
        .sym_name()
        .unwrap_or_else(|| key.trim().to_lowercase());
    combos
        .iter()
        .map(|&m| ComboUsage {
            mods: m,
            bound_by: bound_by(binds, m, &k, submap),
        })
        .collect()
}

/// Varios binds sobre la misma combinación en el mismo submap. Hyprland los
/// ejecuta todos, así que puede ser intencionado, pero conviene verlo.
pub struct Conflict {
    pub label: String,
    pub submap: String,
    pub members: Vec<usize>,
}

pub fn conflicts(binds: &[Bind]) -> Vec<Conflict> {
    let mut groups: HashMap<(String, ModMask, String), Vec<usize>> = HashMap::new();
    for (i, b) in binds.iter().enumerate() {
        let name = b.key.sym_name().unwrap_or_else(|| b.key.norm());
        groups
            .entry((b.submap.clone(), b.mods, name))
            .or_default()
            .push(i);
    }
    let mut out: Vec<Conflict> = groups
        .into_iter()
        .filter(|(_, v)| v.len() > 1)
        .map(|((submap, _, _), members)| Conflict {
            label: binds[members[0]].combo(),
            submap,
            members,
        })
        .collect();
    out.sort_by_key(|c| c.members[0]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_str;
    use std::path::Path;

    fn binds() -> Vec<Bind> {
        parse_str(
            "#TAGS: apps\nbind = SUPER, T, exec, kitty\nbind = SUPER SHIFT, T, exec, ghostty\n\
             bind = SUPER, F1, exec, firefox\nbindd = SUPER, Q, Cerrar ventana, killactive,\n\
             #TAGS:\nbind = SUPER, 36, exec, footclient\nbind = SUPER, Q, exec, otra-cosa\n\
             submap = resize\nbind = , right, resizeactive, 10 0\nbind = , catchall, submap, reset\nsubmap = reset\n",
            Path::new("x.conf"),
        )
        .binds
    }

    #[test]
    fn tag_filter_handles_untagged() {
        let b = binds();
        let active: HashSet<String> = ["apps".to_string()].into_iter().collect();
        assert_eq!(filter_by_tags(&b, &active, false).len(), 4);
        assert_eq!(filter_by_tags(&b, &active, true).len(), 8);
        assert_eq!(filter_by_tags(&b, &HashSet::new(), true).len(), 4);
    }

    #[test]
    fn fuzzy_search_by_app_and_all() {
        let b = binds();
        let r = search(&b, "fire", SearchField::App);
        assert_eq!(r, vec![2]);
        let r = search(&b, "cerrar", SearchField::All);
        assert_eq!(r, vec![3]);
        let r = search(&b, "exec kitty", SearchField::All);
        assert_eq!(r, vec![0]);
        // literal primero: "fire" no debe arrastrar "F1" ni "footclient"
        assert_eq!(search(&b, "fire", SearchField::All), vec![2]);
        // sin coincidencia literal se usa la difusa
        assert_eq!(search(&b, "frfx", SearchField::App), vec![2]);
        assert_eq!(search(&b, "", SearchField::All).len(), b.len());
    }

    #[test]
    fn key_search_prefers_exact_mods() {
        let b = binds();
        let r = search(&b, "super t", SearchField::Key);
        assert_eq!(r, vec![0, 1]);
        let r = search(&b, "super+shift+t", SearchField::Key);
        assert_eq!(r, vec![1]);
        let r = search(&b, "return", SearchField::Key);
        assert_eq!(r, vec![4]);
        assert_eq!(search(&b, "super", SearchField::Key).len(), 6);
    }

    #[test]
    fn free_keys_respect_keycodes_and_catchall() {
        let b = binds();
        assert_eq!(bound_by(&b, ModMask::SUPER, "t", ""), Some(0));
        assert_eq!(bound_by(&b, ModMask::SUPER, "return", ""), Some(4));
        assert_eq!(bound_by(&b, ModMask::SUPER, "z", ""), None);
        assert_eq!(bound_by(&b, ModMask::NONE, "z", "resize"), Some(7));
        let report = free_keys(&b, ModMask::SUPER, "");
        let letters = &report[0];
        assert_eq!(letters.free_count(), 24);
    }

    #[test]
    fn free_combos_for_a_key() {
        let b = binds();
        let combos = combo_candidates(&b, "");
        assert_eq!(combos[0], ModMask::SUPER);
        let r = free_combos(&b, "T", "", &combos);
        assert_eq!(r[0].bound_by, Some(0));
        let shift = r
            .iter()
            .find(|c| c.mods == ModMask::SUPER | ModMask::SHIFT)
            .unwrap();
        assert_eq!(shift.bound_by, Some(1));
        let ctrl = r
            .iter()
            .find(|c| c.mods == ModMask::SUPER | ModMask::CTRL)
            .unwrap();
        assert_eq!(ctrl.bound_by, None);
    }

    #[test]
    fn detects_duplicate_combos() {
        let b = binds();
        let c = conflicts(&b);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].members, vec![3, 5]);
        assert_eq!(c[0].label, "SUPER+Q");
    }
}
