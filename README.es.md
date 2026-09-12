# bindanalyzer

Chuleta y analizador de atajos de teclado de Hyprland para el terminal. Escrito en Rust.

Pensado para vivir en un *special workspace* de Hyprland: se abre una vez, se
queda en segundo plano y se recarga solo cuando cambia la configuración.

[English version](README.md)

## Qué hace

- Lee `hyprland.conf` y todos sus `source =`, resolviendo `$variables`,
  comentarios, `submap`, `bind`, `bindm`, `binde`, `bindd`... y los `#TAGS:`.
- Consulta `hyprctl binds -j` para saber qué tiene cargado Hyprland de verdad,
  y avisa de los binds del fichero que no están cargados (amarillo) o de los
  cargados que no están en el fichero (gris).
- Vistas:
  - **Chuleta**: tabla filtrable por tags.
  - **Buscar** (`/`): por texto, por aplicación o por combinación (`super shift t`).
  - **Teclas libres** (`f`): qué teclas quedan libres para cada combinación de modificadores.
  - **Combos libres** (`l`): para una tecla dada, qué combinaciones están libres.
  - **Conflictos** (`c`): combinaciones repetidas en el mismo submap.
- `--json` vuelca todo para scripts, wofi, rofi, etc.
- Interfaz en inglés o castellano (`--lang`, o automático según `LANG`).

## Tags

Añade comentarios en tu `hyprland.conf`. Los binds que siguen a una línea
`#TAGS:` heredan esos tags hasta la siguiente. Una línea `#TAGS:` vacía los
limpia. Los binds sin tag aparecen bajo `(sin tag)`.

```
#TAGS: apps main
bind = $mainMod, F1, exec, firefox
bind = $mainMod, F2, exec, thunderbird

#TAGS: workspaces
bind = $mainMod, 1, workspace, 1
#TAGS:
```

## Compilar e instalar

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

## Licencia

GPL-3.0-or-later. Úsalo bajo tu propia responsabilidad.
