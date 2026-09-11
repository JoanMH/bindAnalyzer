//! Lectura de los binds cargados a través de `hyprctl binds -j`.

use crate::model::{Bind, BindFlags, Key, ModMask, Origin};
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum LiveError {
    #[error("Hyprland no está en ejecución (HYPRLAND_INSTANCE_SIGNATURE no definida)")]
    NotRunning,
    #[error("no se pudo ejecutar hyprctl: {0}")]
    Exec(std::io::Error),
    #[error("hyprctl devolvió un error: {0}")]
    Failed(String),
    #[error("JSON de hyprctl no válido: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawBind {
    locked: bool,
    mouse: bool,
    release: bool,
    repeat: bool,
    #[serde(rename = "longPress")]
    long_press: bool,
    non_consuming: bool,
    has_description: bool,
    modmask: u32,
    submap: String,
    key: String,
    keycode: u32,
    catch_all: bool,
    description: String,
    dispatcher: String,
    arg: String,
}

/// `true` si estamos dentro de una sesión de Hyprland.
pub fn is_running() -> bool {
    std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some()
}

/// Ejecuta `hyprctl binds -j` y convierte el resultado.
pub fn live_binds() -> Result<Vec<Bind>, LiveError> {
    if !is_running() {
        return Err(LiveError::NotRunning);
    }
    let out = Command::new("hyprctl")
        .args(["binds", "-j"])
        .output()
        .map_err(LiveError::Exec)?;
    if !out.status.success() {
        return Err(LiveError::Failed(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    parse_live_json(&out.stdout)
}

/// Convierte la salida JSON de `hyprctl binds -j`.
pub fn parse_live_json(bytes: &[u8]) -> Result<Vec<Bind>, LiveError> {
    let raw: Vec<RawBind> = serde_json::from_slice(bytes)?;
    Ok(raw.into_iter().map(convert).collect())
}

fn convert(r: RawBind) -> Bind {
    let key = if r.catch_all {
        Key::CatchAll
    } else if r.keycode != 0 {
        Key::Code(r.keycode)
    } else {
        Key::Sym(r.key)
    };
    Bind {
        mods: ModMask(r.modmask),
        key,
        dispatcher: r.dispatcher,
        arg: r.arg,
        submap: r.submap,
        flags: BindFlags {
            locked: r.locked,
            release: r.release,
            long_press: r.long_press,
            repeat: r.repeat,
            non_consuming: r.non_consuming,
            mouse: r.mouse,
            description: r.has_description,
            ..BindFlags::default()
        },
        description: r.description,
        tags: Vec::new(),
        source: None,
        origin: Origin::Live,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hyprctl_json() {
        let json = br#"[
          {"locked":false,"mouse":false,"release":false,"repeat":false,"longPress":false,
           "non_consuming":false,"auto_consuming":false,"has_description":false,"modmask":68,
           "submap":"","submap_universal":"false","key":"0","keycode":0,"catch_all":false,
           "description":"","dispatcher":"exec","arg":"~/x.sh"},
          {"modmask":64,"submap":"","key":"","keycode":36,"dispatcher":"exec","arg":"footclient"},
          {"modmask":65,"submap":"resize","key":"","keycode":0,"catch_all":true,"dispatcher":"submap","arg":"reset","repeat":true}
        ]"#;
        let b = parse_live_json(json).unwrap();
        assert_eq!(b.len(), 3);
        assert_eq!(b[0].mods, ModMask::SUPER | ModMask::CTRL);
        assert_eq!(b[0].key, Key::Sym("0".into()));
        assert_eq!(b[1].key, Key::Code(36));
        assert_eq!(b[2].key, Key::CatchAll);
        assert!(b[2].flags.repeat);
        assert_eq!(b[2].submap, "resize");
        assert_eq!(b[0].origin, Origin::Live);
    }
}
