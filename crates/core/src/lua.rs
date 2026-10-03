//! Lector de `hyprland.lua`.
//!
//! Ejecuta el fichero en un intérprete Lua embebido con una tabla `hl` falsa:
//! `hl.bind` anota cada bind con su línea, `hl.dsp.*` devuelve descripciones
//! legibles en lugar de actuar, `hl.define_submap` fija el submap y el resto
//! de `hl.*` no hace nada. Los `--TAGS:` se leen del texto por número de línea.

use crate::config::{ConfigError, ParsedConfig, TAGS_MARKER};
use crate::model::{Bind, BindFlags, Key, ModMask, Origin, SourceRef};
use mlua::{Lua, Table, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Código Lua que sustituye a la API `hl` de Hyprland.
const PRELUDE: &str = r##"
local __binds, __files, __prints, __unbinds = {}, {}, {}, {}
local __submap = ""

local function render(v, depth)
  local t = type(v)
  if t == "string" then return v
  elseif t == "number" or t == "boolean" then return tostring(v)
  elseif t == "function" then return "function"
  elseif t == "nil" then return ""
  elseif t == "table" then
    if v.__dsp then return v.desc end
    if depth > 3 then return "{...}" end
    local keys = {}
    for k in pairs(v) do keys[#keys + 1] = k end
    table.sort(keys, function(a, b) return tostring(a) < tostring(b) end)
    local parts = {}
    for _, k in ipairs(keys) do
      local val = v[k]
      if type(k) == "number" then parts[#parts + 1] = render(val, depth + 1)
      elseif val == true then parts[#parts + 1] = tostring(k)
      else parts[#parts + 1] = tostring(k) .. "=" .. render(val, depth + 1) end
    end
    local s = table.concat(parts, " ")
    if depth > 0 then s = "{" .. s .. "}" end
    return s
  end
  return tostring(v)
end

local function make_dsp(path)
  return setmetatable({}, {
    __index = function(_, k)
      return make_dsp(path == "" and tostring(k) or path .. "." .. tostring(k))
    end,
    __call = function(_, ...)
      local parts = {}
      for i = 1, select("#", ...) do parts[#parts + 1] = render((select(i, ...)), 0) end
      local name = path
      if path == "exec_cmd" then name = "exec" elseif path == "exec_raw" then name = "exec-raw" end
      local arg = table.concat(parts, " ")
      local desc = name
      if arg ~= "" then desc = name .. " " .. arg end
      return { __dsp = true, dispatcher = name, arg = arg, desc = desc }
    end,
  })
end

local noop_proxy
noop_proxy = setmetatable({}, {
  __index = function() return noop_proxy end,
  __call = function() return nil end,
})

local hl = setmetatable({}, { __index = function() return noop_proxy end })
hl.dsp = make_dsp("")

function hl.bind(keys, action, opts)
  local info = debug.getinfo(2, "Sl")
  local dispatcher, arg
  if type(action) == "table" and action.__dsp then
    dispatcher, arg = action.dispatcher, action.arg
  elseif type(action) == "function" then
    dispatcher, arg = "lua", "function"
  elseif type(action) == "string" then
    dispatcher, arg = action, ""
  else
    dispatcher, arg = tostring(action), ""
  end
  local o = {}
  if type(opts) == "table" then
    for k, v in pairs(opts) do o[tostring(k)] = v end
  end
  __binds[#__binds + 1] = {
    keys = tostring(keys), dispatcher = dispatcher, arg = arg, opts = o,
    file = info and info.source or "?", line = info and info.currentline or 0,
    submap = __submap,
  }
  return { set_enabled = function() end }
end

function hl.unbind(keys)
  __unbinds[#__unbinds + 1] = { keys = tostring(keys), submap = __submap }
end

function hl.define_submap(name, a, b)
  local fn = type(b) == "function" and b or a
  local saved = __submap
  __submap = tostring(name)
  if type(fn) == "function" then fn() end
  __submap = saved
end

function hl.dispatch() end

_G.hl = hl

print = function(...)
  local p = {}
  for i = 1, select("#", ...) do p[#p + 1] = tostring((select(i, ...))) end
  __prints[#__prints + 1] = table.concat(p, "\t")
end

local orig_require = require
require = function(name)
  local path = package.searchpath(tostring(name), package.path)
  if path then __files[#__files + 1] = path end
  return orig_require(name)
end

_G.__bindanalyzer = { binds = __binds, files = __files, prints = __prints, unbinds = __unbinds }
"##;

/// Lee y ejecuta un fichero `hyprland.lua`.
pub fn parse_lua(path: &Path) -> Result<ParsedConfig, ConfigError> {
    let text =
        std::fs::read_to_string(path).map_err(|e| ConfigError::Read(path.to_path_buf(), e))?;
    Ok(parse_lua_str(&text, path))
}

/// Ejecuta un texto Lua. `path` da nombre al chunk y resuelve los `require`
/// relativos a su directorio.
pub fn parse_lua_str(text: &str, path: &Path) -> ParsedConfig {
    let mut out = ParsedConfig {
        files: vec![path.to_path_buf()],
        ..ParsedConfig::default()
    };
    let main_name = path.display().to_string();

    // El fichero de configuración es código del usuario que Hyprland ejecuta
    // igualmente; aquí necesitamos `debug.getinfo` para conocer las líneas.
    let lua = unsafe { Lua::unsafe_new() };

    if let Err(e) = prepare(&lua, path) {
        out.warnings.push(format!("{main_name}: lua setup failed: {e}"));
        return out;
    }
    if let Err(e) = lua
        .load(text)
        .set_name(format!("@{main_name}"))
        .exec()
    {
        out.warnings
            .push(format!("{main_name}: lua error: {}", short_error(&e)));
    }

    match collect(&lua, path, text, &mut out) {
        Ok(()) => {}
        Err(e) => out
            .warnings
            .push(format!("{main_name}: could not read results: {e}")),
    }
    out
}

fn prepare(lua: &Lua, path: &Path) -> mlua::Result<()> {
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let package: Table = lua.globals().get("package")?;
    let old: String = package.get("path").unwrap_or_default();
    let d = dir.display();
    package.set("path", format!("{d}/?.lua;{d}/?/init.lua;{old}"))?;
    lua.load(PRELUDE).set_name("=bindanalyzer").exec()
}

fn short_error(e: &mlua::Error) -> String {
    match e {
        mlua::Error::RuntimeError(s) | mlua::Error::SyntaxError { message: s, .. } => {
            s.lines().next().unwrap_or("").to_string()
        }
        other => other.to_string(),
    }
}

/// Tags vigentes en cada línea (índice 1) según los comentarios `--TAGS:`.
fn tag_map(text: &str) -> Vec<Vec<String>> {
    let mut map = vec![Vec::new()];
    let mut current: Vec<String> = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("--") {
            let rest = rest.trim_start();
            if let Some(tags) = rest.strip_prefix(&TAGS_MARKER[1..]) {
                current = tags.split_whitespace().map(String::from).collect();
            }
        }
        map.push(current.clone());
    }
    map
}

struct FileInfo {
    lines: Vec<String>,
    tags: Vec<Vec<String>>,
}

/// Línea original y tags vigentes en `file:line`, leyendo el fichero la primera vez.
fn line_info(
    infos: &mut HashMap<PathBuf, FileInfo>,
    file: &Path,
    line: usize,
) -> (String, Vec<String>) {
    if !infos.contains_key(file) {
        let Ok(text) = std::fs::read_to_string(file) else {
            return (String::new(), Vec::new());
        };
        infos.insert(
            file.to_path_buf(),
            FileInfo {
                lines: text.lines().map(String::from).collect(),
                tags: tag_map(&text),
            },
        );
    }
    let info = &infos[file];
    (
        info.lines.get(line.wrapping_sub(1)).cloned().unwrap_or_default(),
        info.tags.get(line).cloned().unwrap_or_default(),
    )
}

fn collect(lua: &Lua, main: &Path, main_text: &str, out: &mut ParsedConfig) -> mlua::Result<()> {
    let result: Table = lua.globals().get("__bindanalyzer")?;

    let files: Table = result.get("files")?;
    for f in files.sequence_values::<String>() {
        let p = PathBuf::from(f?);
        if !out.files.contains(&p) {
            out.files.push(p);
        }
    }
    let prints: Table = result.get("prints")?;
    for p in prints.sequence_values::<String>() {
        out.warnings.push(format!("lua print: {}", p?));
    }

    let mut infos: HashMap<PathBuf, FileInfo> = HashMap::new();
    infos.insert(
        main.to_path_buf(),
        FileInfo {
            lines: main_text.lines().map(String::from).collect(),
            tags: tag_map(main_text),
        },
    );
    let binds: Table = result.get("binds")?;
    for entry in binds.sequence_values::<Table>() {
        let entry = entry?;
        let keys: String = entry.get("keys")?;
        let dispatcher: String = entry.get("dispatcher")?;
        let arg: String = entry.get("arg")?;
        let submap: String = entry.get("submap")?;
        let source: String = entry.get("file")?;
        let line: usize = entry.get::<Option<usize>>("line")?.unwrap_or(0);
        let opts: Table = entry.get("opts")?;

        let file = PathBuf::from(source.strip_prefix('@').unwrap_or(&source));
        let (mods, key) = parse_keys(&keys);
        let (mods, unknown) = mods;
        if !unknown.is_empty() {
            out.warnings.push(format!(
                "{}:{line}: unknown modifiers: {}",
                file.display(),
                unknown.join(" ")
            ));
        }

        let (raw, tags) = line_info(&mut infos, &file, line);

        let flag = |name: &str| -> bool {
            matches!(opts.get::<Value>(name), Ok(Value::Boolean(true)))
        };
        let description: String = match opts.get::<Value>("description") {
            Ok(Value::String(s)) => s.to_str().map(|s| s.to_string()).unwrap_or_default(),
            _ => String::new(),
        };
        let flags = BindFlags {
            locked: flag("locked"),
            release: flag("release"),
            long_press: flag("long_press"),
            repeat: flag("repeating") || flag("repeat"),
            non_consuming: flag("non_consuming"),
            mouse: flag("mouse"),
            transparent: flag("transparent"),
            ignore_mods: flag("ignore_mods"),
            separate: flag("separate"),
            description: !description.is_empty(),
            bypass_inhibit: flag("bypass_inhibit"),
            click: flag("click"),
            drag: flag("drag"),
        };

        out.binds.push(Bind {
            mods,
            key: Key::parse(&key),
            dispatcher,
            arg,
            submap,
            flags,
            description,
            tags,
            source: Some(SourceRef { file, line, raw }),
            origin: Origin::Config,
        });
    }

    let unbinds: Table = result.get("unbinds")?;
    for u in unbinds.sequence_values::<Table>() {
        let u = u?;
        let keys: String = u.get("keys")?;
        let submap: String = u.get("submap")?;
        if keys.eq_ignore_ascii_case("all") {
            out.binds.clear();
            continue;
        }
        let ((mods, _), key) = parse_keys(&keys);
        let norm = Key::parse(&key).norm();
        out.binds
            .retain(|b| !(b.submap == submap && b.mods == mods && b.key.norm() == norm));
    }
    Ok(())
}

/// `"SUPER + SHIFT + Q"` → (modificadores, desconocidos), tecla.
fn parse_keys(keys: &str) -> ((ModMask, Vec<String>), String) {
    let tokens: Vec<&str> = keys
        .split(|c: char| c == '+' || c.is_whitespace())
        .filter(|t| !t.is_empty())
        .collect();
    let Some((key, mods)) = tokens.split_last() else {
        return ((ModMask::NONE, Vec::new()), String::new());
    };
    (ModMask::from_names(&mods.join(" ")), key.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
local mainMod = "SUPER"
--TAGS: displays
hl.bind("SUPER + CTRL + 0", hl.dsp.exec_cmd("~/.config/hypr/scripts/eDP-1-only.sh"))
-- TAGS: window move
hl.bind("SUPER + mouse:272", hl.dsp.window.drag())
hl.bind("SUPER + C", function() local m = hl.get_active_monitor(); if not m then return end end)
hl.bind("mouse:49", hl.dsp.focus({ workspace = "e-1" }))
--TAGS: submap resize
hl.bind(mainMod .. " + R", hl.dsp.submap("resize"))
hl.define_submap("resize", function()
    hl.bind("right", hl.dsp.window.resize({ x = 10, y = 0, relative = true }), { repeating = true })
    hl.bind(1, function() end)
    hl.bind("Return", hl.dsp.submap("reset"))
end)
--TAGS:
hl.bind(mainMod .. " + code:36", hl.dsp.exec_cmd("footclient"))
hl.bind(mainMod .. " + Q", hl.dsp.window.close(), { description = "Cerrar ventana" })
--TAGS: workspaces
for i = 1, 3 do
  hl.bind(mainMod .. " + " .. i, hl.dsp.focus({ workspace = tostring(i) }))
end
hl.config({ general = { gaps_in = 5 } })
hl.monitor({ output = "eDP-1" })
hl.bind("SUPER + Z", hl.dsp.exec_cmd("zzz"))
hl.unbind("SUPER + Z")
print("hola", 1)
"#;

    fn parse() -> ParsedConfig {
        parse_lua_str(SAMPLE, Path::new("/tmp/hyprland.lua"))
    }

    #[test]
    fn collects_binds_with_actions_and_lines() {
        let p = parse();
        let names: Vec<String> = p.binds.iter().map(|b| b.combo()).collect();
        assert_eq!(p.binds.len(), 13, "{names:?} warnings={:?}", p.warnings);
        let b = &p.binds[0];
        assert_eq!(b.mods, ModMask::SUPER | ModMask::CTRL);
        assert_eq!(b.key, Key::Sym("0".into()));
        assert_eq!(b.action(), "exec ~/.config/hypr/scripts/eDP-1-only.sh");
        assert_eq!(b.source.as_ref().unwrap().line, 4);
        assert!(b.source.as_ref().unwrap().raw.starts_with("hl.bind(\"SUPER + CTRL + 0\""));
        assert_eq!(b.tags, vec!["displays"]);
    }

    #[test]
    fn renders_dispatchers_functions_and_tables() {
        let p = parse();
        let drag = &p.binds[1];
        assert_eq!(drag.action(), "window.drag");
        assert_eq!(drag.tags, vec!["window", "move"]);
        let func = &p.binds[2];
        assert_eq!(func.action(), "lua function");
        let focus = &p.binds[3];
        assert_eq!(focus.mods, ModMask::NONE);
        assert_eq!(focus.key, Key::Sym("mouse:49".into()));
        assert_eq!(focus.action(), "focus workspace=e-1");
    }

    #[test]
    fn submaps_options_and_variables() {
        let p = parse();
        let r = &p.binds[4];
        assert_eq!(r.mods, ModMask::SUPER);
        assert_eq!(r.action(), "submap resize");
        let right = &p.binds[5];
        assert_eq!(right.submap, "resize");
        assert!(right.flags.repeat);
        assert_eq!(right.action(), "window.resize relative x=10 y=0");
        assert_eq!(right.tags, vec!["submap", "resize"]);
        let one = &p.binds[6];
        assert_eq!(one.key, Key::Sym("1".into()));
        assert_eq!(one.submap, "resize");
        let ret = &p.binds[7];
        assert_eq!(ret.action(), "submap reset");
        let code = &p.binds[8];
        assert_eq!(code.key, Key::Code(36));
        assert!(code.tags.is_empty());
        assert_eq!(code.submap, "");
        let q = &p.binds[9];
        assert_eq!(q.description, "Cerrar ventana");
        assert!(q.flags.description);
    }

    #[test]
    fn loops_unbind_and_print_capture() {
        let p = parse();
        let ws: Vec<&Bind> = p.binds.iter().filter(|b| b.tags == vec!["workspaces"]).collect();
        assert_eq!(ws.len(), 3);
        assert_eq!(ws[2].action(), "focus workspace=3");
        assert_eq!(ws[0].source.as_ref().unwrap().line, 21);
        assert!(!p.binds.iter().any(|b| b.key == Key::Sym("Z".into())));
        assert!(p.warnings.iter().any(|w| w == "lua print: hola\t1"), "{:?}", p.warnings);
    }

    #[test]
    fn requires_files_from_config_dir_and_scopes_tags() {
        let dir = std::env::temp_dir().join(format!("bindanalyzer-lua-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("extra.lua"),
            "--TAGS: extra\nhl.bind(\"SUPER + X\", hl.dsp.exec_cmd(\"foo\"))\n",
        )
        .unwrap();
        let main = dir.join("hyprland.lua");
        std::fs::write(
            &main,
            "--TAGS: main\nrequire(\"extra\")\nhl.bind(\"SUPER + Y\", hl.dsp.exec_cmd(\"bar\"))\n",
        )
        .unwrap();
        let p = parse_lua(&main).unwrap();
        assert_eq!(p.files.len(), 2, "{:?}", p.files);
        assert_eq!(p.binds.len(), 2);
        assert_eq!(p.binds[0].tags, vec!["extra"]);
        assert_eq!(p.binds[0].source.as_ref().unwrap().file, dir.join("extra.lua"));
        assert_eq!(p.binds[0].source.as_ref().unwrap().line, 2);
        assert_eq!(p.binds[1].tags, vec!["main"]);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn errors_keep_binds_collected_so_far() {
        let p = parse_lua_str(
            "hl.bind(\"SUPER + A\", hl.dsp.exit())\nerror(\"boom\")\nhl.bind(\"SUPER + B\", hl.dsp.exit())\n",
            Path::new("x.lua"),
        );
        assert_eq!(p.binds.len(), 1);
        assert!(p.warnings.iter().any(|w| w.contains("boom")), "{:?}", p.warnings);
        let p = parse_lua_str("hl.bind(\"NOPE + A\", hl.dsp.exit())", Path::new("x.lua"));
        assert_eq!(p.binds.len(), 1);
        assert!(p.warnings[0].contains("unknown modifiers: NOPE"));
    }
}
