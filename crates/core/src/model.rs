//! Tipos del dominio: modificadores, tecla, flags y el propio bind.

use serde::Serialize;
use std::fmt;
use std::path::PathBuf;

/// Máscara de modificadores con los mismos bits que usa Hyprland
/// (`modmask` en `hyprctl binds -j`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct ModMask(pub u32);

impl ModMask {
    pub const NONE: ModMask = ModMask(0);
    pub const SHIFT: ModMask = ModMask(1);
    pub const CAPS: ModMask = ModMask(2);
    pub const CTRL: ModMask = ModMask(4);
    pub const ALT: ModMask = ModMask(8);
    pub const MOD2: ModMask = ModMask(16);
    pub const MOD3: ModMask = ModMask(32);
    pub const SUPER: ModMask = ModMask(64);
    pub const MOD5: ModMask = ModMask(128);

    /// Orden canónico de presentación.
    const TABLE: [(ModMask, &'static str); 8] = [
        (Self::SUPER, "SUPER"),
        (Self::CTRL, "CTRL"),
        (Self::ALT, "ALT"),
        (Self::SHIFT, "SHIFT"),
        (Self::CAPS, "CAPS"),
        (Self::MOD2, "MOD2"),
        (Self::MOD3, "MOD3"),
        (Self::MOD5, "MOD5"),
    ];

    /// Reconoce un nombre de modificador con los alias que acepta Hyprland.
    pub fn parse_one(name: &str) -> Option<ModMask> {
        match name.to_ascii_uppercase().as_str() {
            "SHIFT" => Some(Self::SHIFT),
            "CAPS" => Some(Self::CAPS),
            "CTRL" | "CONTROL" => Some(Self::CTRL),
            "ALT" | "MOD1" => Some(Self::ALT),
            "MOD2" => Some(Self::MOD2),
            "MOD3" => Some(Self::MOD3),
            "SUPER" | "WIN" | "LOGO" | "MOD4" => Some(Self::SUPER),
            "MOD5" => Some(Self::MOD5),
            _ => None,
        }
    }

    /// Analiza `"SUPER SHIFT"` o `"SUPER+SHIFT"`. Devuelve la máscara y los
    /// nombres que no se han reconocido.
    pub fn from_names(s: &str) -> (ModMask, Vec<String>) {
        let mut mask = ModMask::NONE;
        let mut unknown = Vec::new();
        for tok in s
            .split(|c: char| c.is_whitespace() || c == '+')
            .filter(|t| !t.is_empty())
        {
            match Self::parse_one(tok) {
                Some(m) => mask = mask | m,
                None => unknown.push(tok.to_string()),
            }
        }
        (mask, unknown)
    }

    pub fn bits(self) -> u32 {
        self.0
    }

    pub fn contains(self, other: ModMask) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn names(self) -> Vec<&'static str> {
        Self::TABLE
            .iter()
            .filter(|(m, _)| self.contains(*m))
            .map(|(_, n)| *n)
            .collect()
    }

    /// `"SUPER+SHIFT"`; cadena vacía si no hay modificadores.
    pub fn label(self) -> String {
        self.names().join("+")
    }
}

impl std::ops::BitOr for ModMask {
    type Output = ModMask;
    fn bitor(self, rhs: ModMask) -> ModMask {
        ModMask(self.0 | rhs.0)
    }
}

impl fmt::Display for ModMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label())
    }
}

/// La tecla de un bind tal y como la entiende Hyprland.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(tag = "kind", content = "value")]
pub enum Key {
    /// Nombre de keysym (`Q`, `F1`, `mouse:272`, `XF86AudioMute`).
    Sym(String),
    /// Keycode X11 (`36`, `code:36`). Hyprland trata como keycode los números mayores que 9.
    Code(u32),
    /// `catchall`: captura cualquier tecla con esos modificadores.
    CatchAll,
}

impl Key {
    pub fn parse(s: &str) -> Key {
        let s = s.trim();
        if s.eq_ignore_ascii_case("catchall") {
            return Key::CatchAll;
        }
        if let Some(rest) = s.strip_prefix("code:") {
            if let Ok(c) = rest.trim().parse() {
                return Key::Code(c);
            }
        }
        if let Ok(n) = s.parse::<u32>() {
            if n > 9 {
                return Key::Code(n);
            }
        }
        Key::Sym(s.to_string())
    }

    /// Forma normalizada para comparar identidades.
    pub fn norm(&self) -> String {
        match self {
            Key::Sym(s) => s.to_lowercase(),
            Key::Code(c) => format!("code:{c}"),
            Key::CatchAll => "catchall".to_string(),
        }
    }

    /// Nombre de keysym en minúsculas, traduciendo keycodes conocidos.
    pub fn sym_name(&self) -> Option<String> {
        match self {
            Key::Sym(s) => Some(s.to_lowercase()),
            Key::Code(c) => keycode_to_sym(*c).map(|s| s.to_lowercase()),
            Key::CatchAll => None,
        }
    }

    /// Texto para mostrar: `Return [36]` para keycodes conocidos.
    pub fn display(&self) -> String {
        match self {
            Key::Sym(s) => s.clone(),
            Key::Code(c) => match keycode_to_sym(*c) {
                Some(n) => format!("{n} [{c}]"),
                None => format!("code:{c}"),
            },
            Key::CatchAll => "catchall".to_string(),
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display())
    }
}

/// Keycodes X11 (evdev + 8) de un teclado US estándar.
pub fn keycode_to_sym(code: u32) -> Option<&'static str> {
    Some(match code {
        9 => "Escape",
        10 => "1",
        11 => "2",
        12 => "3",
        13 => "4",
        14 => "5",
        15 => "6",
        16 => "7",
        17 => "8",
        18 => "9",
        19 => "0",
        20 => "minus",
        21 => "equal",
        22 => "BackSpace",
        23 => "Tab",
        24 => "q",
        25 => "w",
        26 => "e",
        27 => "r",
        28 => "t",
        29 => "y",
        30 => "u",
        31 => "i",
        32 => "o",
        33 => "p",
        34 => "bracketleft",
        35 => "bracketright",
        36 => "Return",
        37 => "Control_L",
        38 => "a",
        39 => "s",
        40 => "d",
        41 => "f",
        42 => "g",
        43 => "h",
        44 => "j",
        45 => "k",
        46 => "l",
        47 => "semicolon",
        48 => "apostrophe",
        49 => "grave",
        50 => "Shift_L",
        51 => "backslash",
        52 => "z",
        53 => "x",
        54 => "c",
        55 => "v",
        56 => "b",
        57 => "n",
        58 => "m",
        59 => "comma",
        60 => "period",
        61 => "slash",
        62 => "Shift_R",
        63 => "KP_Multiply",
        64 => "Alt_L",
        65 => "space",
        66 => "Caps_Lock",
        67 => "F1",
        68 => "F2",
        69 => "F3",
        70 => "F4",
        71 => "F5",
        72 => "F6",
        73 => "F7",
        74 => "F8",
        75 => "F9",
        76 => "F10",
        77 => "Num_Lock",
        78 => "Scroll_Lock",
        79 => "KP_7",
        80 => "KP_8",
        81 => "KP_9",
        82 => "KP_Subtract",
        83 => "KP_4",
        84 => "KP_5",
        85 => "KP_6",
        86 => "KP_Add",
        87 => "KP_1",
        88 => "KP_2",
        89 => "KP_3",
        90 => "KP_0",
        91 => "KP_Decimal",
        94 => "less",
        95 => "F11",
        96 => "F12",
        104 => "KP_Enter",
        105 => "Control_R",
        106 => "KP_Divide",
        107 => "Print",
        108 => "Alt_R",
        110 => "Home",
        111 => "Up",
        112 => "Prior",
        113 => "Left",
        114 => "Right",
        115 => "End",
        116 => "Down",
        117 => "Next",
        118 => "Insert",
        119 => "Delete",
        121 => "XF86AudioMute",
        122 => "XF86AudioLowerVolume",
        123 => "XF86AudioRaiseVolume",
        127 => "Pause",
        133 => "Super_L",
        134 => "Super_R",
        135 => "Menu",
        _ => return None,
    })
}

/// Flags de `bind[flags] =`. Las letras son las de la documentación de Hyprland.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize)]
pub struct BindFlags {
    /// `l`: funciona con la pantalla bloqueada.
    pub locked: bool,
    /// `r`: se dispara al soltar la tecla.
    pub release: bool,
    /// `o`: pulsación larga.
    pub long_press: bool,
    /// `e`: se repite mientras se mantiene.
    pub repeat: bool,
    /// `n`: no consume la tecla, la ventana también la recibe.
    pub non_consuming: bool,
    /// `m`: bind de ratón.
    pub mouse: bool,
    /// `t`: transparente, no puede ser sombreado por otros binds.
    pub transparent: bool,
    /// `i`: ignora modificadores.
    pub ignore_mods: bool,
    /// `s`: separa las combinaciones de modificadores.
    pub separate: bool,
    /// `d`: lleva descripción (`bindd`).
    pub description: bool,
    /// `p`: ignora las peticiones de inhibición de la app.
    pub bypass_inhibit: bool,
    /// `c`: click.
    pub click: bool,
    /// `g`: drag.
    pub drag: bool,
}

impl BindFlags {
    /// Devuelve los flags y las letras no reconocidas.
    pub fn from_letters(s: &str) -> (BindFlags, Vec<char>) {
        let mut f = BindFlags::default();
        let mut unknown = Vec::new();
        for c in s.chars() {
            match c {
                'l' => f.locked = true,
                'r' => f.release = true,
                'o' => f.long_press = true,
                'e' => f.repeat = true,
                'n' => f.non_consuming = true,
                'm' => f.mouse = true,
                't' => f.transparent = true,
                'i' => f.ignore_mods = true,
                's' => f.separate = true,
                'd' => f.description = true,
                'p' => f.bypass_inhibit = true,
                'c' => f.click = true,
                'g' => f.drag = true,
                other => unknown.push(other),
            }
        }
        (f, unknown)
    }

    /// Letras activas, en el orden en que suelen escribirse.
    pub fn letters(&self) -> String {
        let pairs = [
            (self.locked, 'l'),
            (self.release, 'r'),
            (self.long_press, 'o'),
            (self.repeat, 'e'),
            (self.non_consuming, 'n'),
            (self.mouse, 'm'),
            (self.transparent, 't'),
            (self.ignore_mods, 'i'),
            (self.separate, 's'),
            (self.description, 'd'),
            (self.bypass_inhibit, 'p'),
            (self.click, 'c'),
            (self.drag, 'g'),
        ];
        pairs.iter().filter(|(on, _)| *on).map(|(_, c)| *c).collect()
    }

    /// Nombres legibles de los flags activos.
    pub fn describe(&self) -> Vec<&'static str> {
        let pairs = [
            (self.locked, "locked"),
            (self.release, "release"),
            (self.long_press, "long press"),
            (self.repeat, "repeat"),
            (self.non_consuming, "non-consuming"),
            (self.mouse, "mouse"),
            (self.transparent, "transparent"),
            (self.ignore_mods, "ignore mods"),
            (self.separate, "separate"),
            (self.description, "description"),
            (self.bypass_inhibit, "bypass inhibit"),
            (self.click, "click"),
            (self.drag, "drag"),
        ];
        pairs.iter().filter(|(on, _)| *on).map(|(_, n)| *n).collect()
    }
}

/// Dónde está escrito el bind.
#[derive(Clone, Debug, Serialize)]
pub struct SourceRef {
    pub file: PathBuf,
    pub line: usize,
    /// Línea original sin recortar.
    pub raw: String,
}

/// De dónde sale un bind tras fusionar fichero y `hyprctl`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Origin {
    /// Está en el fichero. Si `hyprctl` está disponible, significa que no está cargado.
    Config,
    /// Lo devuelve `hyprctl` pero no aparece en el fichero.
    Live,
    /// Está en el fichero y cargado.
    Both,
}

/// Identidad de un bind: submap, modificadores y tecla normalizada.
pub type BindId = (String, ModMask, String);

#[derive(Clone, Debug, Serialize)]
pub struct Bind {
    pub mods: ModMask,
    pub key: Key,
    pub dispatcher: String,
    pub arg: String,
    /// Submap al que pertenece; cadena vacía para el global.
    pub submap: String,
    pub flags: BindFlags,
    pub description: String,
    pub tags: Vec<String>,
    pub source: Option<SourceRef>,
    pub origin: Origin,
}

impl Bind {
    pub fn id(&self) -> BindId {
        (self.submap.clone(), self.mods, self.key.norm())
    }

    /// `SUPER+SHIFT+T`.
    pub fn combo(&self) -> String {
        let m = self.mods.label();
        let k = self.key.display();
        if m.is_empty() {
            k
        } else {
            format!("{m}+{k}")
        }
    }

    /// Dispatcher y argumento en una sola cadena.
    pub fn action(&self) -> String {
        if self.arg.is_empty() {
            self.dispatcher.clone()
        } else {
            format!("{} {}", self.dispatcher, self.arg)
        }
    }

    /// `fichero:línea` con solo el nombre del fichero.
    pub fn location(&self) -> String {
        match &self.source {
            Some(s) => {
                let name = s
                    .file
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| s.file.display().to_string());
                format!("{name}:{}", s.line)
            }
            None => String::from("hyprctl"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modmask_matches_hyprland_bits() {
        let (m, unk) = ModMask::from_names("SUPER CTRL");
        assert_eq!(m.bits(), 68);
        assert!(unk.is_empty());
        assert_eq!(ModMask::from_names("$mainMod SHIFT").1, vec!["$mainMod"]);
        assert_eq!(ModMask::from_names("super+alt").0, ModMask::SUPER | ModMask::ALT);
        assert_eq!(ModMask(65).label(), "SUPER+SHIFT");
        assert_eq!(ModMask::NONE.label(), "");
    }

    #[test]
    fn key_parsing_follows_hyprland_rules() {
        assert_eq!(Key::parse("Q"), Key::Sym("Q".into()));
        assert_eq!(Key::parse("0"), Key::Sym("0".into()));
        assert_eq!(Key::parse("36"), Key::Code(36));
        assert_eq!(Key::parse("code:65"), Key::Code(65));
        assert_eq!(Key::parse("catchall"), Key::CatchAll);
        assert_eq!(Key::Code(36).display(), "Return [36]");
        assert_eq!(Key::Code(36).sym_name().as_deref(), Some("return"));
        assert_eq!(Key::Code(999).display(), "code:999");
    }

    #[test]
    fn flags_roundtrip() {
        let (f, unk) = BindFlags::from_letters("elx");
        assert!(f.repeat && f.locked && !f.mouse);
        assert_eq!(unk, vec!['x']);
        assert_eq!(f.letters(), "le");
    }
}
