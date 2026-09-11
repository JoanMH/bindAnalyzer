//! Dibujo de la interfaz con ratatui.

use crate::app::{App, Focus, View};
use bindanalyzer_core::model::{Bind, ModMask, Origin};
use bindanalyzer_core::query;
use bindanalyzer_core::LiveStatus;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Cell, Clear, List, ListItem, Paragraph, Row, Table, Wrap};

const ACCENT: Color = Color::Cyan;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .split(area);

    draw_header(f, app, chunks[0]);

    let (main, tags_area) = if app.tags_visible {
        let h = Layout::horizontal([Constraint::Min(20), Constraint::Length(26)]).split(chunks[1]);
        (h[0], Some(h[1]))
    } else {
        (chunks[1], None)
    };

    match app.view {
        View::Cheatsheet | View::Search | View::Conflicts => draw_table(f, app, main),
        View::FreeKeys => draw_free_keys(f, app, main),
        View::FreeCombos => draw_free_combos(f, app, main),
    }
    if let Some(a) = tags_area {
        draw_tags(f, app, a);
    }
    draw_footer(f, app, chunks[2]);

    if app.detail {
        draw_detail(f, app, area);
    }
    if app.help {
        draw_help(f, area);
    }
}

fn mods_label(m: ModMask) -> String {
    if m.is_empty() {
        "sin modificador".to_string()
    } else {
        m.label()
    }
}

fn border_style(focused: bool) -> Style {
    if focused {
        Style::new().fg(ACCENT)
    } else {
        Style::new().fg(Color::DarkGray)
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let file = app
        .snap
        .config_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut spans = vec![
        Span::styled(" bindanalyzer ", Style::new().bold().reversed()),
        Span::styled(
            format!(" {} ", app.view.title()),
            Style::new().fg(ACCENT).bold(),
        ),
        Span::raw(format!(" {file}  {} binds", app.snap.binds.len())),
    ];
    let not_loaded = app.snap.not_loaded_count();
    if not_loaded > 0 {
        spans.push(Span::styled(
            format!("  {not_loaded} sin cargar"),
            Style::new().fg(Color::Yellow),
        ));
    }
    let live_only = app.snap.live_only_count();
    if live_only > 0 {
        spans.push(Span::styled(
            format!("  {live_only} solo en hyprctl"),
            Style::new().fg(Color::DarkGray),
        ));
    }
    if !app.snap.warnings.is_empty() {
        spans.push(Span::styled(
            format!("  {} avisos", app.snap.warnings.len()),
            Style::new().fg(Color::Yellow),
        ));
    }
    spans.push(match &app.snap.live {
        LiveStatus::Ok => Span::styled("  hyprctl ✓", Style::new().fg(Color::Green)),
        LiveStatus::Disabled => Span::styled("  sin hyprctl", Style::new().fg(Color::DarkGray)),
        LiveStatus::NotRunning => {
            Span::styled("  Hyprland no activo", Style::new().fg(Color::Yellow))
        }
        LiveStatus::Error(_) => Span::styled("  hyprctl error", Style::new().fg(Color::Red)),
    });
    if !app.status.is_empty() {
        spans.push(Span::styled(
            format!("  · {}", app.status),
            Style::new().fg(Color::DarkGray),
        ));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn origin_style(b: &Bind, live_ok: bool) -> Style {
    match (b.origin, live_ok) {
        (Origin::Config, true) => Style::new().fg(Color::Yellow),
        (Origin::Live, _) => Style::new().fg(Color::DarkGray).italic(),
        _ => Style::new(),
    }
}

fn draw_table(f: &mut Frame, app: &mut App, area: Rect) {
    let binds = &app.snap.binds;
    let rows_idx = &app.rows;
    let live_ok = app.snap.live == LiveStatus::Ok;

    let show_group = app.view == View::Conflicts;
    let show_submap = rows_idx.iter().any(|&i| !binds[i].submap.is_empty());
    let show_desc = !app.compact && rows_idx.iter().any(|&i| !binds[i].description.is_empty());
    let show_tags = !app.compact;
    let show_src = !app.compact;

    let width = |sel: &dyn Fn(&Bind) -> String, min: usize, max: usize| -> u16 {
        rows_idx
            .iter()
            .map(|&i| sel(&binds[i]).chars().count())
            .max()
            .unwrap_or(0)
            .clamp(min, max) as u16
    };

    let mut header: Vec<Cell> = Vec::new();
    let mut widths: Vec<Constraint> = Vec::new();
    if show_group {
        header.push(Cell::from("#"));
        widths.push(Constraint::Length(3));
    }
    header.push(Cell::from("Mods"));
    widths.push(Constraint::Length(width(&|b| b.mods.label(), 4, 24)));
    header.push(Cell::from("Tecla"));
    widths.push(Constraint::Length(width(&|b| b.key.display(), 5, 20)));
    if show_submap {
        header.push(Cell::from("Submap"));
        widths.push(Constraint::Length(width(&|b| b.submap.clone(), 6, 12)));
    }
    header.push(Cell::from("Acción"));
    widths.push(Constraint::Min(20));
    if show_desc {
        header.push(Cell::from("Descripción"));
        widths.push(Constraint::Length(width(&|b| b.description.clone(), 11, 30)));
    }
    if show_tags {
        header.push(Cell::from("Tags"));
        widths.push(Constraint::Length(width(&|b| b.tags.join(" "), 4, 24)));
    }
    if show_src {
        header.push(Cell::from("Origen"));
        widths.push(Constraint::Length(width(&|b| b.location(), 6, 28)));
    }

    let rows: Vec<Row> = rows_idx
        .iter()
        .enumerate()
        .map(|(r, &i)| {
            let b = &binds[i];
            let mut cells: Vec<Cell> = Vec::new();
            if show_group {
                let g = app.row_groups.get(r).map(|g| g.to_string()).unwrap_or_default();
                cells.push(Cell::from(g).style(Style::new().fg(Color::Yellow)));
            }
            cells.push(Cell::from(b.mods.label()).style(Style::new().fg(Color::Blue)));
            cells.push(Cell::from(b.key.display()).style(Style::new().fg(Color::Magenta).bold()));
            if show_submap {
                cells.push(Cell::from(b.submap.clone()).style(Style::new().fg(ACCENT)));
            }
            cells.push(Cell::from(b.action()));
            if show_desc {
                cells.push(Cell::from(b.description.clone()));
            }
            if show_tags {
                cells.push(Cell::from(b.tags.join(" ")).style(Style::new().fg(Color::DarkGray)));
            }
            if show_src {
                cells.push(Cell::from(b.location()).style(Style::new().fg(Color::DarkGray)));
            }
            Row::new(cells).style(origin_style(b, live_ok))
        })
        .collect();

    let title = match app.view {
        View::Search => format!(
            " {} · {} resultados · campo: {} ",
            app.view.title(),
            rows_idx.len(),
            app.field.label()
        ),
        View::Conflicts => format!(
            " {} · {} combinaciones repetidas ",
            app.view.title(),
            app.conflicts.len()
        ),
        _ => format!(" {} · {} de {} ", app.view.title(), rows_idx.len(), binds.len()),
    };

    let table = Table::new(rows, widths)
        .header(Row::new(header).style(Style::new().bold().underlined()))
        .block(
            Block::bordered()
                .title(title)
                .border_style(border_style(app.focus == Focus::Main)),
        )
        .row_highlight_style(Style::new().reversed())
        .highlight_symbol("▶ ")
        .column_spacing(1);
    f.render_stateful_widget(table, area, &mut app.table);

    if rows_idx.is_empty() {
        let msg = match app.view {
            View::Search if app.query.trim().is_empty() => "Escribe para buscar",
            View::Search => "Sin resultados",
            View::Conflicts => "Sin combinaciones repetidas",
            _ => "Ningún bind visible con los tags activos",
        };
        let inner = Rect {
            x: area.x + 2,
            y: area.y + 2,
            width: area.width.saturating_sub(4),
            height: 1,
        };
        f.render_widget(
            Paragraph::new(msg).style(Style::new().fg(Color::DarkGray)),
            inner,
        );
    }
}

fn draw_tags(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Tags;
    let items: Vec<ListItem> = app
        .tags
        .iter()
        .map(|t| {
            let (mark, color) = if t.active {
                ("●", Color::Green)
            } else {
                ("○", Color::Red)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {mark} "), Style::new().fg(color)),
                Span::raw(t.name.clone()),
            ]))
        })
        .collect();
    let active = app.tags.iter().filter(|t| t.active).count();
    let list = List::new(items)
        .block(
            Block::bordered()
                .title(format!(" Tags {}/{} [t] ", active, app.tags.len()))
                .border_style(border_style(focused)),
        )
        .highlight_style(if focused {
            Style::new().reversed()
        } else {
            Style::new()
        })
        .highlight_symbol(if focused { "▶" } else { " " });
    f.render_stateful_widget(list, area, &mut app.tag_list);
}

fn draw_free_keys(f: &mut Frame, app: &App, area: Rect) {
    let binds = &app.snap.binds;
    let submap = app.submap().to_string();
    let submap_label = if submap.is_empty() {
        "global".to_string()
    } else {
        submap.clone()
    };
    let mods = app.current_combo();
    let report = query::free_keys(binds, mods, &submap);
    let total: usize = report.iter().map(|g| g.keys.len()).sum();
    let free: usize = report.iter().map(|g| g.free_count()).sum();

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::raw(" Submap: "),
            Span::styled(submap_label, Style::new().fg(ACCENT).bold()),
            Span::raw("   Modificadores: "),
            Span::styled(format!(" ◀ {} ▶ ", mods_label(mods)), Style::new().bold().reversed()),
            Span::styled(
                format!("  {}/{}", app.combo_idx + 1, app.combos.len()),
                Style::new().fg(Color::DarkGray),
            ),
        ]),
        Line::raw(""),
    ];
    for g in &report {
        let mut spans = vec![Span::styled(format!(" {:<11}", g.name), Style::new().bold())];
        for k in &g.keys {
            let style = match k.bound_by {
                None => Style::new().fg(Color::Green),
                Some(_) => Style::new().fg(Color::DarkGray).crossed_out(),
            };
            spans.push(Span::styled(format!("{} ", k.key), style));
        }
        lines.push(Line::from(spans));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("verde", Style::new().fg(Color::Green)),
        Span::raw(" libre · "),
        Span::styled("gris tachado", Style::new().fg(Color::DarkGray).crossed_out()),
        Span::raw(" ocupada"),
    ]));
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        " Ocupadas con esta combinación:",
        Style::new().bold(),
    ));
    for g in &report {
        for k in &g.keys {
            if let Some(i) = k.bound_by {
                let b = &binds[i];
                lines.push(Line::from(vec![
                    Span::styled(format!("   {:<12}", k.key), Style::new().fg(Color::Magenta)),
                    Span::raw(b.action()),
                    Span::styled(format!("  {}", b.location()), Style::new().fg(Color::DarkGray)),
                ]));
            }
        }
    }
    let others = query::other_bound(binds, mods, &submap);
    if !others.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::styled(" Otras teclas ocupadas:", Style::new().bold()));
        for i in others {
            let b = &binds[i];
            lines.push(Line::from(vec![
                Span::styled(format!("   {:<12}", b.key.display()), Style::new().fg(Color::Magenta)),
                Span::raw(b.action()),
                Span::styled(format!("  {}", b.location()), Style::new().fg(Color::DarkGray)),
            ]));
        }
    }

    let p = Paragraph::new(Text::from(lines))
        .block(
            Block::bordered()
                .title(format!(" Teclas libres · {free} libres de {total} "))
                .border_style(border_style(app.focus == Focus::Main)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.scroll, 0));
    f.render_widget(p, area);
}

fn draw_free_combos(f: &mut Frame, app: &App, area: Rect) {
    let binds = &app.snap.binds;
    let submap = app.submap().to_string();
    let submap_label = if submap.is_empty() {
        "global".to_string()
    } else {
        submap.clone()
    };
    let key = app.letter.trim().to_string();
    let editing = app.focus == Focus::Input;

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::raw(" Tecla: "),
            Span::styled(
                format!(" {}{} ", key, if editing { "▏" } else { "" }),
                Style::new().bold().reversed(),
            ),
            Span::raw("   Submap: "),
            Span::styled(submap_label, Style::new().fg(ACCENT).bold()),
        ]),
        Line::raw(""),
    ];

    let mut free = 0;
    if key.is_empty() {
        lines.push(Line::styled(
            " Escribe una tecla (q, F5, Return, space, mouse:272...) y pulsa Enter.",
            Style::new().fg(Color::DarkGray),
        ));
    } else {
        let usage = query::free_combos(binds, &key, &submap, &app.combos);
        for u in usage {
            let label = format!("   {:<22}", mods_label(u.mods));
            match u.bound_by {
                None => {
                    free += 1;
                    lines.push(Line::from(vec![
                        Span::styled(label, Style::new().fg(Color::Blue)),
                        Span::styled("libre", Style::new().fg(Color::Green).bold()),
                    ]));
                }
                Some(i) => {
                    let b = &binds[i];
                    lines.push(Line::from(vec![
                        Span::styled(label, Style::new().fg(Color::Blue)),
                        Span::styled("ocupada  ", Style::new().fg(Color::Red)),
                        Span::raw(b.action()),
                        Span::styled(
                            format!("  {}", b.location()),
                            Style::new().fg(Color::DarkGray),
                        ),
                    ]));
                }
            }
        }
        lines.push(Line::raw(""));
        lines.push(Line::styled(
            " Se listan las combinaciones ya usadas en este submap y las estándar alrededor de SUPER.",
            Style::new().fg(Color::DarkGray),
        ));
    }

    let title = if key.is_empty() {
        " Combos libres ".to_string()
    } else {
        format!(" Combos libres para {key} · {free} libres de {} ", app.combos.len())
    };
    let p = Paragraph::new(Text::from(lines))
        .block(
            Block::bordered()
                .title(title)
                .border_style(border_style(app.focus != Focus::Tags)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.scroll, 0));
    f.render_widget(p, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let line = match (app.focus, app.view) {
        (Focus::Input, View::FreeCombos) => Line::from(vec![
            Span::styled(" Tecla: ", Style::new().bold()),
            Span::raw(app.letter.clone()),
            Span::styled("▏", Style::new().fg(ACCENT)),
            Span::styled("   Enter listo · Esc cancelar", Style::new().fg(Color::DarkGray)),
        ]),
        (Focus::Input, _) => Line::from(vec![
            Span::styled(" / ", Style::new().bold()),
            Span::raw(app.query.clone()),
            Span::styled("▏", Style::new().fg(ACCENT)),
            Span::styled(
                format!(
                    "   Tab campo: {}  · Enter listo · Esc salir · Ctrl-u borrar",
                    app.field.label()
                ),
                Style::new().fg(Color::DarkGray),
            ),
        ]),
        (Focus::Tags, _) => hint(" j/k mover · espacio alternar · a todos · n ninguno · o solo este · Esc volver"),
        (Focus::Main, View::Cheatsheet) => hint(
            " / buscar · f teclas libres · l combos de una tecla · c conflictos · t tags · m compacto · Enter detalle · r recargar · ? ayuda · q salir",
        ),
        (Focus::Main, View::Search) => hint(&format!(
            " i editar · Tab campo ({}) · Enter detalle · Esc chuleta · q salir",
            app.field.label()
        )),
        (Focus::Main, View::FreeKeys) => {
            hint(" ←/→ o n/p cambiar combinación · s submap · j/k desplazar · l combos de una tecla · Esc chuleta")
        }
        (Focus::Main, View::FreeCombos) => hint(" l otra tecla · s submap · Esc chuleta · q salir"),
        (Focus::Main, View::Conflicts) => hint(" Enter detalle · Esc chuleta · q salir"),
    };
    f.render_widget(Paragraph::new(line), area);
}

fn hint(s: &str) -> Line<'static> {
    Line::styled(s.to_string(), Style::new().fg(Color::DarkGray))
}

fn centered(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let v = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(v[1])[1]
}

fn field(name: &str, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {name:<13}"), Style::new().fg(ACCENT).bold()),
        Span::raw(value),
    ])
}

fn draw_detail(f: &mut Frame, app: &App, area: Rect) {
    let Some(b) = app.current() else {
        return;
    };
    let live_ok = app.snap.live == LiveStatus::Ok;
    let state = match (b.origin, live_ok) {
        (Origin::Both, _) => "cargado en Hyprland".to_string(),
        (Origin::Config, true) => "en el fichero pero NO cargado (¿falta hyprctl reload?)".to_string(),
        (Origin::Config, false) => "en el fichero".to_string(),
        (Origin::Live, _) => "cargado pero no está en el fichero".to_string(),
    };
    let mut flags = b.flags.describe().join(", ");
    if flags.is_empty() {
        flags = "ninguno".to_string();
    } else {
        flags = format!("{} ({})", b.flags.letters(), flags);
    }
    let mut lines = vec![
        field("Combinación", b.combo()),
        field(
            "Submap",
            if b.submap.is_empty() {
                "global".to_string()
            } else {
                b.submap.clone()
            },
        ),
        field("Dispatcher", b.dispatcher.clone()),
        field("Argumento", b.arg.clone()),
        field("Flags", flags),
        field("Descripción", b.description.clone()),
        field("Tags", b.tags.join(" ")),
        field("Estado", state),
    ];
    if let Some(s) = &b.source {
        lines.push(field("Fichero", format!("{}:{}", s.file.display(), s.line)));
        lines.push(Line::raw(""));
        lines.push(Line::styled(s.raw.clone(), Style::new().fg(Color::DarkGray)));
    }
    let popup = centered(80, 60, area);
    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(
                Block::bordered()
                    .title(" Detalle · Esc cierra ")
                    .border_style(Style::new().fg(ACCENT)),
            )
            .wrap(Wrap { trim: false }),
        popup,
    );
}

fn draw_help(f: &mut Frame, area: Rect) {
    let rows = [
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
    ];
    let lines: Vec<Line> = rows
        .iter()
        .map(|(k, d)| {
            Line::from(vec![
                Span::styled(format!(" {k:<20}"), Style::new().fg(ACCENT).bold()),
                Span::raw(*d),
            ])
        })
        .collect();
    let popup = centered(80, 80, area);
    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(
                Block::bordered()
                    .title(" Ayuda · cualquier tecla cierra ")
                    .border_style(Style::new().fg(ACCENT)),
            )
            .wrap(Wrap { trim: false }),
        popup,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::View;
    use bindanalyzer_core::config::parse_str;
    use bindanalyzer_core::snapshot::build;
    use bindanalyzer_core::LoadOptions;
    use crossterm::event::{KeyCode, KeyEvent};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use std::path::Path;

    fn app() -> App {
        let parsed = parse_str(
            "#TAGS: apps\nbind = SUPER, T, exec, kitty\nbind = SUPER SHIFT, T, exec, ghostty\n#TAGS:\nbind = SUPER, Q, killactive,\nbind = SUPER, Q, exec, dup\n",
            Path::new("hyprland.conf"),
        );
        let snap = build(Path::new("hyprland.conf"), parsed, None);
        App::new(LoadOptions { config: "hyprland.conf".into(), use_live: false }, snap, false, View::Cheatsheet)
    }

    fn render(app: &mut App) -> String {
        render_size(app, 120, 24)
    }

    fn render_size(app: &mut App, w: u16, h: u16) -> String {
        let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
        t.draw(|f| draw(f, app)).unwrap();
        let buf = t.backend().buffer().clone();
        let mut out = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    #[test]
    fn renders_every_view_without_panicking() {
        let mut app = app();
        let s = render(&mut app);
        assert!(s.contains("Chuleta"), "{s}");
        assert!(s.contains("kitty"));
        assert!(s.contains("(sin tag)"));

        app.handle_key(KeyEvent::from(KeyCode::Char('/')));
        for c in "ghost".chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(c)));
        }
        let s = render(&mut app);
        assert!(s.contains("ghostty"));
        assert!(!s.contains("kitty"), "{s}");

        app.handle_key(KeyEvent::from(KeyCode::Esc));
        app.handle_key(KeyEvent::from(KeyCode::Char('f')));
        let s = render(&mut app);
        assert!(s.contains("Teclas libres"));

        app.handle_key(KeyEvent::from(KeyCode::Char('l')));
        for c in "t".chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(c)));
        }
        app.handle_key(KeyEvent::from(KeyCode::Enter));
        let s = render(&mut app);
        assert!(s.contains("ocupada"), "{s}");
        assert!(s.contains("libre"));

        app.handle_key(KeyEvent::from(KeyCode::Char('c')));
        let s = render(&mut app);
        assert!(s.contains("1 combinaciones repetidas"), "{s}");

        app.handle_key(KeyEvent::from(KeyCode::Enter));
        let s = render(&mut app);
        assert!(s.contains("Detalle"));
        app.handle_key(KeyEvent::from(KeyCode::Esc));
        app.handle_key(KeyEvent::from(KeyCode::Char('?')));
        let s = render(&mut app);
        assert!(s.contains("Ayuda"));
    }

    /// Herramienta de desarrollo: imprime las vistas con una config real.
    /// `BINDANALYZER_CONFIG=~/.config/hypr/hyprland.conf cargo test -p bindanalyzer -- --ignored --nocapture real_config_frames`
    #[test]
    #[ignore]
    fn real_config_frames() {
        use crossterm::event::KeyModifiers;
        let Some(path) = std::env::var_os("BINDANALYZER_CONFIG") else {
            return;
        };
        let opts = LoadOptions { config: path.into(), use_live: true };
        let snap = bindanalyzer_core::load(&opts).unwrap();
        let mut app = App::new(opts, snap, false, View::Cheatsheet);
        fn show(app: &mut App, title: &str) {
            println!("=== {title}");
            print!("{}", render_size(app, 150, 38));
        }
        fn keys(app: &mut App, s: &str) {
            for c in s.chars() {
                app.handle_key(KeyEvent::from(KeyCode::Char(c)));
            }
        }
        show(&mut app, "chuleta");
        keys(&mut app, "c");
        show(&mut app, "conflictos");
        keys(&mut app, "f");
        show(&mut app, "teclas libres");
        app.handle_key(KeyEvent::from(KeyCode::Right));
        show(&mut app, "teclas libres, siguiente combo");
        keys(&mut app, "lt");
        app.handle_key(KeyEvent::from(KeyCode::Enter));
        show(&mut app, "combos libres para t");
        keys(&mut app, "/fire");
        show(&mut app, "buscar fire");
        app.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
        app.handle_key(KeyEvent::from(KeyCode::Tab));
        app.handle_key(KeyEvent::from(KeyCode::Tab));
        keys(&mut app, "super shift t");
        show(&mut app, "buscar por tecla");
        app.handle_key(KeyEvent::from(KeyCode::Esc));
        keys(&mut app, "m");
        app.handle_key(KeyEvent::from(KeyCode::Enter));
        show(&mut app, "detalle");
    }

    #[test]
    fn tags_panel_filters_rows() {
        let mut app = app();
        assert_eq!(app.rows.len(), 4);
        app.handle_key(KeyEvent::from(KeyCode::Char('t')));
        app.handle_key(KeyEvent::from(KeyCode::Char(' ')));
        assert_eq!(app.rows.len(), 2);
        app.handle_key(KeyEvent::from(KeyCode::Char('n')));
        assert!(app.rows.is_empty());
        app.handle_key(KeyEvent::from(KeyCode::Char('a')));
        assert_eq!(app.rows.len(), 4);
    }
}
