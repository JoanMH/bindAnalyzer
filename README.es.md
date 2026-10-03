# bindanalyzer

Chuleta y analizador de atajos de teclado de Hyprland para el terminal. Escrito en Rust.

Pensado para vivir en un *special workspace* de Hyprland: se abre una vez, se
queda en segundo plano y se recarga solo cuando cambia la configuración.

[English version](README.md)

## Qué hace

- Lee `hyprland.lua` (Hyprland 0.55+) ejecutándolo en un intérprete Lua
  embebido con una tabla `hl` falsa, así que variables, bucles, `require`,
  `hl.define_submap` y opciones de bind se resuelven exactamente como los ve
  Hyprland. Los dispatchers `hl.dsp.*` se muestran como acciones legibles
  (`exec firefox`, `window.close`, `focus workspace=3`); las funciones Lua
  salen como `lua function`, o con su opción `description` si la tienen.
- Sigue leyendo el `hyprland.conf` antiguo y todos sus `source =`, resolviendo
  `$variables`, comentarios, `submap`, `bind`, `bindm`, `binde`, `bindd`...
  Si existen los dos, gana `hyprland.lua`, como en Hyprland; con `-c fichero`
  se elige por la extensión.
- Consulta `hyprctl binds -j` para saber qué tiene cargado Hyprland de verdad,
  y avisa de los binds del fichero que no están cargados (amarillo) o de los
  cargados que no están en el fichero (gris). Con config Lua, Hyprland solo
  informa de callbacks `__lua`, así que el texto de la acción sale siempre del fichero.
- Vistas:
  - **Chuleta**: tabla filtrable por tags.
  - **Buscar** (`/`): por texto, por aplicación o por combinación (`super shift t`).
  - **Teclas libres** (`f`): qué teclas quedan libres para cada combinación de modificadores.
  - **Combos libres** (`l`): para una tecla dada, qué combinaciones están libres.
  - **Conflictos** (`c`): combinaciones repetidas en el mismo submap.
- `--json` vuelca todo para scripts, wofi, rofi, etc.
- Interfaz en inglés o castellano (`--lang`, o automático según `LANG`).

## Tags

Añade comentarios en tu config. Los binds que siguen a un comentario `TAGS:`
heredan esos tags hasta el siguiente. Un comentario `TAGS:` vacío los limpia.
Los binds sin tag aparecen bajo `(sin tag)`. Los ficheros cargados con
`require` o `source` tienen sus propios tags.

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

En el formato antiguo la marca es `#TAGS:`:

```
#TAGS: apps main
bind = $mainMod, F1, exec, firefox
#TAGS:
```

Nota sobre binds por keycode (`code:36`): con config Lua, Hyprland 0.56 los
reporta en `hyprctl` con la tecla vacía aunque funcionan. bindanalyzer los
empareja con el fichero por orden de registro.

## Compilar e instalar

Necesita un compilador de C para el Lua embebido (gcc o clang).

```
cargo build --release
install -Dm755 target/release/bindanalyzer ~/.local/bin/bindanalyzer
```

## Uso

```
bindanalyzer                      # chuleta con hyprctl
bindanalyzer --compact            # menos columnas
bindanalyzer --view search        # arrancar buscando
bindanalyzer --no-live            # solo el fichero, sin hyprctl
bindanalyzer --lang en            # interfaz en inglés
bindanalyzer -c otra.conf --json  # volcado JSON de otro fichero
```

Ejemplo para tenerlo como chuleta en un special workspace de Hyprland.
`foot -a` fija la clase de la ventana, y la windowrule la manda al workspace
especial nada más aparecer. El segundo bind relanza la chuleta si la cierras con `q`.

```
exec-once = foot -a bindanalyzer -T bindanalyzer -e bindanalyzer --compact
windowrule = match:class ^(bindanalyzer)$, workspace special:bindanalyzer silent

bind = $mainMod, U, togglespecialworkspace, bindanalyzer
bind = $mainMod CTRL, U, exec, foot -a bindanalyzer -T bindanalyzer -e bindanalyzer --compact
```

Al recargar la configuración con `hyprctl reload` el `exec-once` no se vuelve a
ejecutar; lanza el comando una vez a mano o usa el bind de relanzar.

### Teclas

| Tecla | Acción |
|---|---|
| `/` | buscar; `Tab` cambia el campo (todo, aplicación, tecla); `Esc` o `Enter` sale del cuadro |
| `f` | teclas libres; `←/→` o `n/p` cambia la combinación; `s` cambia de submap |
| `l` | combos libres para una tecla |
| `c` | conflictos |
| `1` / `Esc` | volver a la chuleta |
| `t` / `Tab` | panel de tags: `espacio` alterna, `a` todos, `n` ninguno, `o` solo este |
| `T` | mostrar u ocultar el panel de tags |
| `m` | modo compacto |
| `j/k`, `↑/↓`, `g/G`, `PgUp/PgDn` | moverse |
| `Enter` | detalle del bind (línea original, fichero, flags, estado) |
| `r` | recargar (también automático al cambiar el fichero) |
| `?` | ayuda |
| `q` | salir |

## Estructura

- `crates/core`: modelo, parser de la config, lectura de `hyprctl`, fusión y consultas. Sin dependencias de interfaz.
- `crates/tui`: la interfaz con ratatui y el binario `bindanalyzer`.

Tests: `cargo test`. Para ver las vistas renderizadas con tu config real:

```
BINDANALYZER_CONFIG=~/.config/hypr/hyprland.conf cargo test -p bindanalyzer -- --ignored --nocapture real_config_frames
```

## Créditos

Este código ha sido realizado con Claude Fable 5.1, el modelo de IA de Anthropic, en colaboración con el autor.

## Licencia

GPL-3.0-or-later. Úsalo bajo tu propia responsabilidad.
