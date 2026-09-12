//! Dibujo de la interfaz con ratatui. Todos los textos salen de `app.t`.

use crate::app::{App, Focus, View, UNTAGGED};
use crate::i18n::{fill, Texts};
use bindanalyzer_core::model::{Bind, ModMask, Origin};
use bindanalyzer_core::query::{self, SearchField};
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
        draw_help(f, app.t, area);
    }
}

fn mods_label(t: &Texts, m: ModMask) -> String {
    if m.is_empty() {
        t.no_modifier.to_string()
    } else {
        m.label()
    }
}

fn submap_label(t: &Texts, submap: &str) -> String {
    if submap.is_empty() {
        t.global_submap.to_string()
    } else {
        submap.to_string()
    }
}

fn field_label(t: &Texts, field: SearchField) -> &'static str {
    match field {
        SearchField::All => t.field_all,
        SearchField::App => t.field_app,
        SearchField::Key => t.field_key,
    }
}

fn group_label(t: &Texts, name: &str) -> &'static str {
    match name {
        "Letters" => t.group_letters,
        "Digits" => t.group_digits,
        "Function" => t.group_function,
        "Navigation" => t.group_navigation,
        "Special" => t.group_special,
        _ => t.group_symbols,
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
    let t = app.t;
    let file = app
        .snap
        .config_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut spans = vec![
        Span::styled(" bindanalyzer ", Style::new().bold().reversed()),
        Span::styled(
            format!(" {} ", app.view.title(t)),
            Style::new().fg(ACCENT).bold(),
        ),
        Span::raw(fill(t.header_binds, &[&file, &app.snap.binds.len()])),
    ];
    let not_loaded = app.snap.not_loaded_count();
    if not_loaded > 0 {
        spans.push(Span::styled(
            fill(t.header_not_loaded, &[&not_loaded]),
            Style::new().fg(Color::Yellow),
        ));
    }
    let live_only = app.snap.live_only_count();
    if live_only > 0 {
        spans.push(Span::styled(
            fill(t.header_live_only, &[&live_only]),
            Style::new().fg(Color::DarkGray),
        ));
    }
    if !app.snap.warnings.is_empty() {
        spans.push(Span::styled(
            fill(t.header_warnings, &[&app.snap.warnings.len()]),
            Style::new().fg(Color::Yellow),
        ));
    }
    spans.push(match &app.snap.live {
        LiveStatus::Ok => Span::styled(t.live_ok, Style::new().fg(Color::Green)),
        LiveStatus::Disabled => Span::styled(t.live_disabled, Style::new().fg(Color::DarkGray)),
        LiveStatus::NotRunning => Span::styled(t.live_not_running, Style::new().fg(Color::Yellow)),
        LiveStatus::Error(_) => Span::styled(t.live_error, Style::new().fg(Color::Red)),
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
    let t = app.t;
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
    header.push(Cell::from(t.col_mods));
    widths.push(Constraint::Length(width(&|b| b.mods.label(), 4, 24)));
    header.push(Cell::from(t.col_key));
    widths.push(Constraint::Length(width(&|b| b.key.display(), 5, 20)));
    if show_submap {
        header.push(Cell::from(t.col_submap));
        widths.push(Constraint::Length(width(&|b| b.submap.clone(), 6, 12)));
    }
    header.push(Cell::from(t.col_action));
    widths.push(Constraint::Min(20));
    if show_desc {
        header.push(Cell::from(t.col_description));
        widths.push(Constraint::Length(width(&|b| b.description.clone(), 11, 30)));
    }
    if show_tags {
        header.push(Cell::from(t.col_tags));
        widths.push(Constraint::Length(width(&|b| b.tags.join(" "), 4, 24)));
    }
    if show_src {
        header.push(Cell::from(t.col_source));
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

    let view_title = app.view.title(t);
    let title = match app.view {
        View::Search => fill(
            t.title_search,
            &[&view_title, &rows_idx.len(), &field_label(t, app.field)],
        ),
        View::Conflicts => fill(t.title_conflicts, &[&view_title, &app.conflicts.len()]),
        _ => fill(t.title_table, &[&view_title, &rows_idx.len(), &binds.len()]),
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
            View::Search if app.query.trim().is_empty() => t.empty_type_to_search,
            View::Search => t.empty_no_results,
            View::Conflicts => t.empty_no_conflicts,
            _ => t.empty_no_visible,
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
    let t = app.t;
    let focused = app.focus == Focus::Tags;
    let items: Vec<ListItem> = app
        .tags
        .iter()
        .map(|tag| {
            let (mark, color) = if tag.active {
                ("●", Color::Green)
            } else {
                ("○", Color::Red)
            };
            let name = if tag.name == UNTAGGED {
                t.untagged.to_string()
            } else {
                tag.name.clone()
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {mark} "), Style::new().fg(color)),
                Span::raw(name),
            ]))
        })
        .collect();
    let active = app.tags.iter().filter(|t| t.active).count();
    let list = List::new(items)
        .block(
            Block::bordered()
                .title(fill(t.tags_title, &[&active, &app.tags.len()]))
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

fn bind_line(prefix: String, b: &Bind) -> Line<'static> {
    Line::from(vec![
        Span::styled(prefix, Style::new().fg(Color::Magenta)),
        Span::raw(b.action()),
        Span::styled(format!("  {}", b.location()), Style::new().fg(Color::DarkGray)),
    ])
}

fn draw_free_keys(f: &mut Frame, app: &App, area: Rect) {
    let t = app.t;
    let binds = &app.snap.binds;
    let submap = app.submap().to_string();
    let mods = app.current_combo();
    let report = query::free_keys(binds, mods, &submap);
    let total: usize = report.iter().map(|g| g.keys.len()).sum();
    let free: usize = report.iter().map(|g| g.free_count()).sum();

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::raw(t.free_submap),
            Span::styled(submap_label(t, &submap), Style::new().fg(ACCENT).bold()),
            Span::raw(t.free_modifiers),
            Span::styled(
                format!(" ◀ {} ▶ ", mods_label(t, mods)),
                Style::new().bold().reversed(),
            ),
            Span::styled(
                format!("  {}/{}", app.combo_idx + 1, app.combos.len()),
                Style::new().fg(Color::DarkGray),
            ),
        ]),
        Line::raw(""),
    ];
    for g in &report {
        let mut spans = vec![Span::styled(
            format!(" {:<11}", group_label(t, g.name)),
            Style::new().bold(),
        )];
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
        Span::styled(t.legend_green, Style::new().fg(Color::Green)),
        Span::raw(t.legend_free),
        Span::styled(t.legend_gray, Style::new().fg(Color::DarkGray).crossed_out()),
        Span::raw(t.legend_used),
    ]));
    lines.push(Line::raw(""));
    lines.push(Line::styled(t.free_used_with_combo, Style::new().bold()));
    for g in &report {
        for k in &g.keys {
            if let Some(i) = k.bound_by {
                lines.push(bind_line(format!("   {:<12}", k.key), &binds[i]));
            }
        }
    }
    let others = query::other_bound(binds, mods, &submap);
    if !others.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::styled(t.free_other_keys, Style::new().bold()));
        for i in others {
            let b = &binds[i];
            lines.push(bind_line(format!("   {:<12}", b.key.display()), b));
        }
    }

    let p = Paragraph::new(Text::from(lines))
        .block(
            Block::bordered()
                .title(fill(t.free_title, &[&free, &total]))
                .border_style(border_style(app.focus == Focus::Main)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.scroll, 0));
    f.render_widget(p, area);
}

fn draw_free_combos(f: &mut Frame, app: &App, area: Rect) {
    let t = app.t;
    let binds = &app.snap.binds;
    let submap = app.submap().to_string();
    let key = app.letter.trim().to_string();
    let editing = app.focus == Focus::Input;

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::raw(t.combos_key),
            Span::styled(
                format!(" {}{} ", key, if editing { "▏" } else { "" }),
                Style::new().bold().reversed(),
            ),
            Span::raw(t.free_submap),
            Span::styled(submap_label(t, &submap), Style::new().fg(ACCENT).bold()),
        ]),
        Line::raw(""),
    ];

    let mut free = 0;
    if key.is_empty() {
        lines.push(Line::styled(t.combos_prompt, Style::new().fg(Color::DarkGray)));
    } else {
        let usage = query::free_combos(binds, &key, &submap, &app.combos);
        for u in usage {
            let label = format!("   {:<22}", mods_label(t, u.mods));
            match u.bound_by {
                None => {
                    free += 1;
                    lines.push(Line::from(vec![
                        Span::styled(label, Style::new().fg(Color::Blue)),
                        Span::styled(t.combos_free, Style::new().fg(Color::Green).bold()),
                    ]));
                }
                Some(i) => {
                    let b = &binds[i];
                    lines.push(Line::from(vec![
                        Span::styled(label, Style::new().fg(Color::Blue)),
                        Span::styled(t.combos_used, Style::new().fg(Color::Red)),
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
        lines.push(Line::styled(t.combos_note, Style::new().fg(Color::DarkGray)));
    }

    let title = if key.is_empty() {
        t.combos_title.to_string()
    } else {
        fill(t.combos_title_key, &[&key, &free, &app.combos.len()])
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
    let t = app.t;
    let line = match (app.focus, app.view) {
        (Focus::Input, View::FreeCombos) => Line::from(vec![
            Span::styled(t.input_key_label, Style::new().bold()),
            Span::raw(app.letter.clone()),
            Span::styled("▏", Style::new().fg(ACCENT)),
            Span::styled(t.input_key_hint, Style::new().fg(Color::DarkGray)),
        ]),
        (Focus::Input, _) => Line::from(vec![
            Span::styled(" / ", Style::new().bold()),
            Span::raw(app.query.clone()),
            Span::styled("▏", Style::new().fg(ACCENT)),
            Span::styled(
                fill(t.input_search_hint, &[&field_label(t, app.field)]),
                Style::new().fg(Color::DarkGray),
            ),
        ]),
        (Focus::Tags, _) => hint(t.hint_tags.to_string()),
        (Focus::Main, View::Cheatsheet) => hint(t.hint_cheatsheet.to_string()),
        (Focus::Main, View::Search) => hint(fill(t.hint_search, &[&field_label(t, app.field)])),
        (Focus::Main, View::FreeKeys) => hint(t.hint_free_keys.to_string()),
        (Focus::Main, View::FreeCombos) => hint(t.hint_free_combos.to_string()),
        (Focus::Main, View::Conflicts) => hint(t.hint_conflicts.to_string()),
    };
    f.render_widget(Paragraph::new(line), area);
}

fn hint(s: String) -> Line<'static> {
    Line::styled(s, Style::new().fg(Color::DarkGray))
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
    let t = app.t;
    let Some(b) = app.current() else {
        return;
    };
    let live_ok = app.snap.live == LiveStatus::Ok;
    let state = match (b.origin, live_ok) {
        (Origin::Both, _) => t.state_loaded,
        (Origin::Config, true) => t.state_not_loaded,
        (Origin::Config, false) => t.state_in_file,
        (Origin::Live, _) => t.state_live_only,
    };
    let names = b.flags.describe().join(", ");
    let flags = if names.is_empty() {
        t.flags_none.to_string()
    } else {
        format!("{} ({})", b.flags.letters(), names)
    };
    let mut lines = vec![
        field(t.detail_combo, b.combo()),
        field(t.detail_submap, submap_label(t, &b.submap)),
        field(t.detail_dispatcher, b.dispatcher.clone()),
        field(t.detail_arg, b.arg.clone()),
        field(t.detail_flags, flags),
        field(t.detail_description, b.description.clone()),
        field(t.detail_tags, b.tags.join(" ")),
        field(t.detail_state, state.to_string()),
    ];
    if let Some(s) = &b.source {
        lines.push(field(t.detail_file, format!("{}:{}", s.file.display(), s.line)));
        lines.push(Line::raw(""));
        lines.push(Line::styled(s.raw.clone(), Style::new().fg(Color::DarkGray)));
    }
    let popup = centered(80, 60, area);
    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(
                Block::bordered()
                    .title(t.detail_title)
                    .border_style(Style::new().fg(ACCENT)),
            )
            .wrap(Wrap { trim: false }),
        popup,
    );
}

fn draw_help(f: &mut Frame, t: &Texts, area: Rect) {
    let lines: Vec<Line> = t
        .help_rows
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
                    .title(t.help_title)
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
    use crate::i18n::Lang;
    use bindanalyzer_core::config::parse_str;
    use bindanalyzer_core::snapshot::build;
    use bindanalyzer_core::LoadOptions;
    use crossterm::event::{KeyCode, KeyEvent};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use std::path::Path;

    fn app(lang: Lang) -> App {
        let parsed = parse_str(
            "#TAGS: apps\nbind = SUPER, T, exec, kitty\nbind = SUPER SHIFT, T, exec, ghostty\n#TAGS:\nbind = SUPER, Q, killactive,\nbind = SUPER, Q, exec, dup\n",
            Path::new("hyprland.conf"),
        );
        let snap = build(Path::new("hyprland.conf"), parsed, None);
        App::new(
            LoadOptions { config: "hyprland.conf".into(), use_live: false },
            snap,
            false,
            View::Cheatsheet,
            lang,
        )
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

    fn keys(app: &mut App, s: &str) {
        for c in s.chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(c)));
        }
    }

    #[test]
    fn renders_every_view_without_panicking() {
        let mut app = app(Lang::En);
        let s = render(&mut app);
        assert!(s.contains("Cheatsheet"), "{s}");
        assert!(s.contains("kitty"));
        assert!(s.contains("(untagged)"));

        keys(&mut app, "/ghost");
        let s = render(&mut app);
        assert!(s.contains("ghostty"));
        assert!(!s.contains("kitty"), "{s}");

        app.handle_key(KeyEvent::from(KeyCode::Esc));
        keys(&mut app, "f");
        let s = render(&mut app);
        assert!(s.contains("Free keys"));

        keys(&mut app, "lt");
        app.handle_key(KeyEvent::from(KeyCode::Enter));
        let s = render(&mut app);
        assert!(s.contains("used"), "{s}");
        assert!(s.contains("free"));

        keys(&mut app, "c");
        let s = render(&mut app);
        assert!(s.contains("1 repeated combos"), "{s}");

        app.handle_key(KeyEvent::from(KeyCode::Enter));
        let s = render(&mut app);
        assert!(s.contains("Detail"));
        app.handle_key(KeyEvent::from(KeyCode::Esc));
        keys(&mut app, "?");
        let s = render(&mut app);
        assert!(s.contains("Help"));
    }

    #[test]
    fn spanish_texts_are_used_when_selected() {
        let mut app = app(Lang::Es);
        let s = render(&mut app);
        assert!(s.contains("Chuleta"), "{s}");
        assert!(s.contains("(sin tag)"));
        assert!(s.contains("Acción"));
        keys(&mut app, "c");
        let s = render(&mut app);
        assert!(s.contains("combinaciones repetidas"), "{s}");
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
        let mut app = App::new(opts, snap, false, View::Cheatsheet, Lang::from_env());
        fn show(app: &mut App, title: &str) {
            println!("=== {title}");
            print!("{}", render_size(app, 150, 38));
        }
        show(&mut app, "cheatsheet");
        keys(&mut app, "c");
        show(&mut app, "conflicts");
        keys(&mut app, "f");
        show(&mut app, "free keys");
        app.handle_key(KeyEvent::from(KeyCode::Right));
        show(&mut app, "free keys, next combo");
        keys(&mut app, "lt");
        app.handle_key(KeyEvent::from(KeyCode::Enter));
        show(&mut app, "free combos for t");
        keys(&mut app, "/fire");
        show(&mut app, "search fire");
        app.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
        app.handle_key(KeyEvent::from(KeyCode::Tab));
        app.handle_key(KeyEvent::from(KeyCode::Tab));
        keys(&mut app, "super shift t");
        show(&mut app, "key search");
        app.handle_key(KeyEvent::from(KeyCode::Esc));
        keys(&mut app, "m");
        app.handle_key(KeyEvent::from(KeyCode::Enter));
        show(&mut app, "detail");
    }

    #[test]
    fn tags_panel_filters_rows() {
        let mut app = app(Lang::En);
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
