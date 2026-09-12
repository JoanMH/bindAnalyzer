//! Textos de la interfaz en inglés y castellano.
//!
//! Sin librerías: una estructura con todos los textos y dos constantes, una por
//! idioma. Las plantillas con `{}` se rellenan con [`fill`].

use clap::ValueEnum;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Lang {
    En,
    Es,
}

impl Lang {
    /// Idioma según `LC_ALL`, `LC_MESSAGES` o `LANG`. Inglés salvo que empiecen por `es`.
    pub fn from_env() -> Lang {
        for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Some(v) = std::env::var_os(var) {
                let v = v.to_string_lossy();
                if v.is_empty() {
                    continue;
                }
                return if v.to_lowercase().starts_with("es") {
                    Lang::Es
                } else {
                    Lang::En
                };
            }
        }
        Lang::En
    }

    pub fn texts(self) -> &'static Texts {
        match self {
            Lang::En => &EN,
            Lang::Es => &ES,
        }
    }
}

/// Sustituye los `{}` de una plantilla, en orden.
pub fn fill(template: &str, args: &[&dyn std::fmt::Display]) -> String {
    let mut out = template.to_string();
    for a in args {
        out = out.replacen("{}", &a.to_string(), 1);
    }
    out
}

pub struct Texts {
    pub view_cheatsheet: &'static str,
    pub view_search: &'static str,
    pub view_free_keys: &'static str,
    pub view_free_combos: &'static str,
    pub view_conflicts: &'static str,

    pub untagged: &'static str,
    pub no_modifier: &'static str,
    pub global_submap: &'static str,

    pub status_reloaded: &'static str,
    pub status_reload_error: &'static str,
    pub status_file_changed: &'static str,

    pub header_binds: &'static str,
    pub header_not_loaded: &'static str,
    pub header_live_only: &'static str,
    pub header_warnings: &'static str,
    pub live_ok: &'static str,
    pub live_disabled: &'static str,
    pub live_not_running: &'static str,
    pub live_error: &'static str,

    pub col_mods: &'static str,
    pub col_key: &'static str,
    pub col_submap: &'static str,
    pub col_action: &'static str,
    pub col_description: &'static str,
    pub col_tags: &'static str,
    pub col_source: &'static str,

    pub title_search: &'static str,
    pub title_conflicts: &'static str,
    pub title_table: &'static str,
    pub empty_type_to_search: &'static str,
    pub empty_no_results: &'static str,
    pub empty_no_conflicts: &'static str,
    pub empty_no_visible: &'static str,

    pub field_all: &'static str,
    pub field_app: &'static str,
    pub field_key: &'static str,

    pub group_letters: &'static str,
    pub group_digits: &'static str,
    pub group_function: &'static str,
    pub group_navigation: &'static str,
    pub group_special: &'static str,
    pub group_symbols: &'static str,

    pub tags_title: &'static str,

    pub free_submap: &'static str,
    pub free_modifiers: &'static str,
    pub legend_green: &'static str,
    pub legend_free: &'static str,
    pub legend_gray: &'static str,
    pub legend_used: &'static str,
    pub free_used_with_combo: &'static str,
    pub free_other_keys: &'static str,
    pub free_title: &'static str,

    pub combos_key: &'static str,
    pub combos_prompt: &'static str,
    pub combos_free: &'static str,
    pub combos_used: &'static str,
    pub combos_note: &'static str,
    pub combos_title: &'static str,
    pub combos_title_key: &'static str,

    pub input_key_label: &'static str,
    pub input_key_hint: &'static str,
    pub input_search_hint: &'static str,
    pub hint_tags: &'static str,
    pub hint_cheatsheet: &'static str,
    pub hint_search: &'static str,
    pub hint_free_keys: &'static str,
    pub hint_free_combos: &'static str,
    pub hint_conflicts: &'static str,

    pub detail_title: &'static str,
    pub detail_combo: &'static str,
    pub detail_submap: &'static str,
    pub detail_dispatcher: &'static str,
    pub detail_arg: &'static str,
    pub detail_flags: &'static str,
    pub detail_description: &'static str,
    pub detail_tags: &'static str,
    pub detail_state: &'static str,
    pub detail_file: &'static str,
    pub flags_none: &'static str,
    pub state_loaded: &'static str,
    pub state_not_loaded: &'static str,
    pub state_in_file: &'static str,
    pub state_live_only: &'static str,

    pub help_title: &'static str,
    pub help_rows: &'static [(&'static str, &'static str)],
}

pub static EN: Texts = Texts {
    view_cheatsheet: "Cheatsheet",
    view_search: "Search",
    view_free_keys: "Free keys",
    view_free_combos: "Free combos",
    view_conflicts: "Conflicts",

    untagged: "(untagged)",
    no_modifier: "no modifier",
    global_submap: "global",

    status_reloaded: "Reloaded",
    status_reload_error: "Reload error: {}",
    status_file_changed: "Config changed, reloaded",

    header_binds: " {}  {} binds",
    header_not_loaded: "  {} not loaded",
    header_live_only: "  {} only in hyprctl",
    header_warnings: "  {} warnings",
    live_ok: "  hyprctl ✓",
    live_disabled: "  no hyprctl",
    live_not_running: "  Hyprland not running",
    live_error: "  hyprctl error",

    col_mods: "Mods",
    col_key: "Key",
    col_submap: "Submap",
    col_action: "Action",
    col_description: "Description",
    col_tags: "Tags",
    col_source: "Source",

    title_search: " {} · {} results · field: {} ",
    title_conflicts: " {} · {} repeated combos ",
    title_table: " {} · {} of {} ",
    empty_type_to_search: "Type to search",
    empty_no_results: "No results",
    empty_no_conflicts: "No repeated combos",
    empty_no_visible: "No bind visible with the active tags",

    field_all: "all",
    field_app: "app",
    field_key: "key",

    group_letters: "Letters",
    group_digits: "Digits",
    group_function: "Function",
    group_navigation: "Navigation",
    group_special: "Special",
    group_symbols: "Symbols",

    tags_title: " Tags {}/{} [t] ",

    free_submap: " Submap: ",
    free_modifiers: "   Modifiers: ",
    legend_green: "green",
    legend_free: " free · ",
    legend_gray: "gray strikethrough",
    legend_used: " used",
    free_used_with_combo: " Used with this combination:",
    free_other_keys: " Other used keys:",
    free_title: " Free keys · {} free of {} ",

    combos_key: " Key: ",
    combos_prompt: " Type a key (q, F5, Return, space, mouse:272...) and press Enter.",
    combos_free: "free",
    combos_used: "used  ",
    combos_note: " Lists the combinations already used in this submap plus the standard ones around SUPER.",
    combos_title: " Free combos ",
    combos_title_key: " Free combos for {} · {} free of {} ",

    input_key_label: " Key: ",
    input_key_hint: "   Enter done · Esc cancel",
    input_search_hint: "   Tab field: {}  · Enter done · Esc leave · Ctrl-u clear",
    hint_tags: " j/k move · space toggle · a all · n none · o only this · Esc back",
    hint_cheatsheet: " / search · f free keys · l combos for a key · c conflicts · t tags · m compact · Enter detail · r reload · ? help · q quit",
    hint_search: " i edit · Tab field ({}) · Enter detail · Esc cheatsheet · q quit",
    hint_free_keys: " ←/→ or n/p change combination · s submap · j/k scroll · l combos for a key · Esc cheatsheet",
    hint_free_combos: " l another key · s submap · Esc cheatsheet · q quit",
    hint_conflicts: " Enter detail · Esc cheatsheet · q quit",

    detail_title: " Detail · Esc closes ",
    detail_combo: "Combination",
    detail_submap: "Submap",
    detail_dispatcher: "Dispatcher",
    detail_arg: "Argument",
    detail_flags: "Flags",
    detail_description: "Description",
    detail_tags: "Tags",
    detail_state: "State",
    detail_file: "File",
    flags_none: "none",
    state_loaded: "loaded in Hyprland",
    state_not_loaded: "in the file but NOT loaded (missing hyprctl reload?)",
    state_in_file: "in the file",
    state_live_only: "loaded but not in the file",

    help_title: " Help · any key closes ",
    help_rows: &[
        ("/", "search (Tab switches the field: all, app, key)"),
        ("f", "free keys for a modifier combination (←/→ or n/p changes it)"),
        ("l", "free combinations for a key"),
        ("c", "repeated combinations"),
        ("1 / Esc", "back to the cheatsheet"),
        ("t / Tab", "tags panel (space toggles, a all, n none, o only this)"),
        ("T", "show or hide the tags panel"),
        ("m", "compact mode (fewer columns)"),
        ("j/k ↑/↓", "move · g/G top/bottom · PgUp/PgDn jumps"),
        ("Enter", "detail of the selected bind"),
        ("r", "reload (also automatic when the file changes)"),
        ("q", "quit"),
        ("", ""),
        ("yellow", "bind in the file that Hyprland has not loaded"),
        ("gray", "loaded bind that is not in the file"),
        ("", ""),
        ("Key search", "type the combination: super shift t · super+t · f5"),
    ],
};

pub static ES: Texts = Texts {
    view_cheatsheet: "Chuleta",
    view_search: "Buscar",
    view_free_keys: "Teclas libres",
    view_free_combos: "Combos libres",
    view_conflicts: "Conflictos",

    untagged: "(sin tag)",
    no_modifier: "sin modificador",
    global_submap: "global",

    status_reloaded: "Recargado",
    status_reload_error: "Error al recargar: {}",
    status_file_changed: "Config modificada, recargado",

    header_binds: " {}  {} binds",
    header_not_loaded: "  {} sin cargar",
    header_live_only: "  {} solo en hyprctl",
    header_warnings: "  {} avisos",
    live_ok: "  hyprctl ✓",
    live_disabled: "  sin hyprctl",
    live_not_running: "  Hyprland no activo",
    live_error: "  hyprctl error",

    col_mods: "Mods",
    col_key: "Tecla",
    col_submap: "Submap",
    col_action: "Acción",
    col_description: "Descripción",
    col_tags: "Tags",
    col_source: "Origen",

    title_search: " {} · {} resultados · campo: {} ",
    title_conflicts: " {} · {} combinaciones repetidas ",
    title_table: " {} · {} de {} ",
    empty_type_to_search: "Escribe para buscar",
    empty_no_results: "Sin resultados",
    empty_no_conflicts: "Sin combinaciones repetidas",
    empty_no_visible: "Ningún bind visible con los tags activos",

    field_all: "todo",
    field_app: "aplicación",
    field_key: "tecla",

    group_letters: "Letras",
    group_digits: "Números",
    group_function: "Función",
    group_navigation: "Navegación",
    group_special: "Especiales",
    group_symbols: "Símbolos",

    tags_title: " Tags {}/{} [t] ",

    free_submap: " Submap: ",
    free_modifiers: "   Modificadores: ",
    legend_green: "verde",
    legend_free: " libre · ",
    legend_gray: "gris tachado",
    legend_used: " ocupada",
    free_used_with_combo: " Ocupadas con esta combinación:",
    free_other_keys: " Otras teclas ocupadas:",
    free_title: " Teclas libres · {} libres de {} ",

    combos_key: " Tecla: ",
    combos_prompt: " Escribe una tecla (q, F5, Return, space, mouse:272...) y pulsa Enter.",
    combos_free: "libre",
    combos_used: "ocupada  ",
    combos_note: " Se listan las combinaciones ya usadas en este submap y las estándar alrededor de SUPER.",
    combos_title: " Combos libres ",
    combos_title_key: " Combos libres para {} · {} libres de {} ",

    input_key_label: " Tecla: ",
    input_key_hint: "   Enter listo · Esc cancelar",
    input_search_hint: "   Tab campo: {}  · Enter listo · Esc salir · Ctrl-u borrar",
    hint_tags: " j/k mover · espacio alternar · a todos · n ninguno · o solo este · Esc volver",
    hint_cheatsheet: " / buscar · f teclas libres · l combos de una tecla · c conflictos · t tags · m compacto · Enter detalle · r recargar · ? ayuda · q salir",
    hint_search: " i editar · Tab campo ({}) · Enter detalle · Esc chuleta · q salir",
    hint_free_keys: " ←/→ o n/p cambiar combinación · s submap · j/k desplazar · l combos de una tecla · Esc chuleta",
    hint_free_combos: " l otra tecla · s submap · Esc chuleta · q salir",
    hint_conflicts: " Enter detalle · Esc chuleta · q salir",

    detail_title: " Detalle · Esc cierra ",
    detail_combo: "Combinación",
    detail_submap: "Submap",
    detail_dispatcher: "Dispatcher",
    detail_arg: "Argumento",
    detail_flags: "Flags",
    detail_description: "Descripción",
    detail_tags: "Tags",
    detail_state: "Estado",
    detail_file: "Fichero",
    flags_none: "ninguno",
    state_loaded: "cargado en Hyprland",
    state_not_loaded: "en el fichero pero NO cargado (¿falta hyprctl reload?)",
    state_in_file: "en el fichero",
    state_live_only: "cargado pero no está en el fichero",

    help_title: " Ayuda · cualquier tecla cierra ",
    help_rows: &[
        ("/", "buscar (Tab cambia el campo: todo, aplicación, tecla)"),
        ("f", "teclas libres para una combinación de modificadores (←/→ o n/p cambia)"),
        ("l", "combinaciones libres para una tecla"),
        ("c", "combinaciones repetidas"),
        ("1 / Esc", "volver a la chuleta"),
        ("t / Tab", "ir al panel de tags (espacio alterna, a todos, n ninguno, o solo)"),
        ("T", "mostrar u ocultar el panel de tags"),
        ("m", "modo compacto (menos columnas)"),
        ("j/k ↑/↓", "moverse · g/G inicio/fin · PgUp/PgDn saltos"),
        ("Enter", "detalle del bind seleccionado"),
        ("r", "recargar (también automático al cambiar el fichero)"),
        ("q", "salir"),
        ("", ""),
        ("amarillo", "bind en el fichero que Hyprland no tiene cargado"),
        ("gris", "bind cargado que no está en el fichero"),
        ("", ""),
        ("Búsqueda por tecla", "escribe la combinación: super shift t · super+t · f5"),
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_replaces_in_order() {
        assert_eq!(fill(" {} · {} of {} ", &[&"A", &2, &3]), " A · 2 of 3 ");
    }
}
