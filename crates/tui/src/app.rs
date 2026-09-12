//! Estado de la aplicación y manejo de teclas. No dibuja nada.

use crate::i18n::{fill, Lang, Texts};
use bindanalyzer_core::model::{Bind, ModMask};
use bindanalyzer_core::query::{self, Conflict, SearchField};
use bindanalyzer_core::{load, LoadOptions, Snapshot};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::{ListState, TableState};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant, SystemTime};

/// Identificador interno del pseudo-tag que agrupa los binds sin `#TAGS:`.
/// Se muestra traducido con `Texts::untagged`.
pub const UNTAGGED: &str = "\u{1}untagged";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Cheatsheet,
    Search,
    FreeKeys,
    FreeCombos,
    Conflicts,
}

impl View {
    pub fn title(self, t: &Texts) -> &'static str {
        match self {
            View::Cheatsheet => t.view_cheatsheet,
            View::Search => t.view_search,
            View::FreeKeys => t.view_free_keys,
            View::FreeCombos => t.view_free_combos,
            View::Conflicts => t.view_conflicts,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Main,
    Tags,
    Input,
}

pub struct TagState {
    pub name: String,
    pub active: bool,
}

pub struct App {
    pub t: &'static Texts,
    pub opts: LoadOptions,
    pub snap: Snapshot,
    pub view: View,
    pub focus: Focus,
    pub tags: Vec<TagState>,
    pub tags_visible: bool,
    pub tag_list: ListState,
    /// Índices en `snap.binds` de las filas visibles de la tabla.
    pub rows: Vec<usize>,
    /// Número de grupo de cada fila, solo en la vista de conflictos.
    pub row_groups: Vec<usize>,
    pub conflicts: Vec<Conflict>,
    pub table: TableState,
    pub query: String,
    pub field: SearchField,
    pub letter: String,
    pub combos: Vec<ModMask>,
    pub combo_idx: usize,
    pub submaps: Vec<String>,
    pub submap_idx: usize,
    /// Desplazamiento vertical en las vistas de texto.
    pub scroll: u16,
    pub compact: bool,
    pub detail: bool,
    pub help: bool,
    pub status: String,
    pub quit: bool,
    last_check: Instant,
    mtime: Option<SystemTime>,
}

impl App {
    pub fn new(opts: LoadOptions, snap: Snapshot, compact: bool, view: View, lang: Lang) -> App {
        let mtime = snap.latest_mtime();
        let mut app = App {
            t: lang.texts(),
            opts,
            snap,
            view: View::Cheatsheet,
            focus: Focus::Main,
            tags: Vec::new(),
            tags_visible: true,
            tag_list: ListState::default(),
            rows: Vec::new(),
            row_groups: Vec::new(),
            conflicts: Vec::new(),
            table: TableState::default(),
            query: String::new(),
            field: SearchField::All,
            letter: String::new(),
            combos: Vec::new(),
            combo_idx: 0,
            submaps: Vec::new(),
            submap_idx: 0,
            scroll: 0,
            compact,
            detail: false,
            help: false,
            status: String::new(),
            quit: false,
            last_check: Instant::now(),
            mtime,
        };
        app.rebuild();
        app.set_view(view);
        if matches!(view, View::Search | View::FreeCombos) {
            app.focus = Focus::Input;
        }
        app
    }

    /// Recalcula todo lo derivado del snapshot conservando el estado de los tags.
    fn rebuild(&mut self) {
        let old: HashMap<String, bool> = self
            .tags
            .iter()
            .map(|t| (t.name.clone(), t.active))
            .collect();
        let mut tags: Vec<TagState> = self
            .snap
            .all_tags()
            .into_iter()
            .map(|name| {
                let active = old.get(&name).copied().unwrap_or(true);
                TagState { name, active }
            })
            .collect();
        if self.snap.has_untagged() {
            tags.push(TagState {
                name: UNTAGGED.to_string(),
                active: old.get(UNTAGGED).copied().unwrap_or(true),
            });
        }
        self.tags = tags;
        let cursor = self
            .tag_list
            .selected()
            .unwrap_or(0)
            .min(self.tags.len().saturating_sub(1));
        self.tag_list.select(if self.tags.is_empty() { None } else { Some(cursor) });

        self.submaps = self.snap.submaps();
        self.submap_idx = self.submap_idx.min(self.submaps.len().saturating_sub(1));
        self.conflicts = query::conflicts(&self.snap.binds);
        self.rebuild_combos();
        self.refresh_rows();
    }

    fn rebuild_combos(&mut self) {
        let submap = self.submap().to_string();
        self.combos = query::combo_candidates(&self.snap.binds, &submap);
        self.combo_idx = self.combo_idx.min(self.combos.len().saturating_sub(1));
    }

    pub fn submap(&self) -> &str {
        self.submaps
            .get(self.submap_idx)
            .map(String::as_str)
            .unwrap_or("")
    }

    pub fn current_combo(&self) -> ModMask {
        self.combos
            .get(self.combo_idx)
            .copied()
            .unwrap_or(ModMask::SUPER)
    }

    pub fn active_tags(&self) -> (HashSet<String>, bool) {
        let active = self
            .tags
            .iter()
            .filter(|t| t.active && t.name != UNTAGGED)
            .map(|t| t.name.clone())
            .collect();
        let untagged = self.tags.iter().any(|t| t.name == UNTAGGED && t.active);
        (active, untagged)
    }

    /// Recalcula las filas visibles según la vista y conserva la selección.
    pub fn refresh_rows(&mut self) {
        self.row_groups.clear();
        self.rows = match self.view {
            View::Cheatsheet => {
                let (active, untagged) = self.active_tags();
                query::filter_by_tags(&self.snap.binds, &active, untagged)
            }
            View::Search => query::search(&self.snap.binds, &self.query, self.field),
            View::Conflicts => {
                let mut rows = Vec::new();
                for (g, c) in self.conflicts.iter().enumerate() {
                    for &i in &c.members {
                        rows.push(i);
                        self.row_groups.push(g + 1);
                    }
                }
                rows
            }
            View::FreeKeys | View::FreeCombos => Vec::new(),
        };
        if self.rows.is_empty() {
            self.table.select(None);
        } else {
            let sel = self
                .table
                .selected()
                .unwrap_or(0)
                .min(self.rows.len() - 1);
            self.table.select(Some(sel));
        }
    }

    pub fn set_view(&mut self, v: View) {
        self.view = v;
        self.scroll = 0;
        self.table.select(Some(0));
        *self.table.offset_mut() = 0;
        self.refresh_rows();
    }

    pub fn current(&self) -> Option<&Bind> {
        self.table
            .selected()
            .and_then(|s| self.rows.get(s))
            .map(|&i| &self.snap.binds[i])
    }

    pub fn reload(&mut self) {
        match load(&self.opts) {
            Ok(s) => {
                self.snap = s;
                self.mtime = self.snap.latest_mtime();
                self.rebuild();
                self.status = self.t.status_reloaded.to_string();
            }
            Err(e) => self.status = fill(self.t.status_reload_error, &[&e]),
        }
    }

    /// Comprueba cada dos segundos si algún fichero cambió y recarga.
    pub fn tick(&mut self) {
        if self.last_check.elapsed() < Duration::from_secs(2) {
            return;
        }
        self.last_check = Instant::now();
        let m = self.snap.latest_mtime();
        if m != self.mtime {
            self.reload();
            self.status = self.t.status_file_changed.to_string();
        }
    }

    pub fn handle_key(&mut self, k: KeyEvent) {
        self.status.clear();
        if self.help {
            self.help = false;
            return;
        }
        if self.detail {
            if matches!(k.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
                self.detail = false;
            }
            return;
        }
        match self.focus {
            Focus::Input => self.key_input(k),
            Focus::Tags => self.key_tags(k),
            Focus::Main => self.key_main(k),
        }
    }

    fn buf_mut(&mut self) -> &mut String {
        if self.view == View::FreeCombos {
            &mut self.letter
        } else {
            &mut self.query
        }
    }

    fn key_input(&mut self, k: KeyEvent) {
        let is_letter = self.view == View::FreeCombos;
        match k.code {
            KeyCode::Esc | KeyCode::Enter => self.focus = Focus::Main,
            KeyCode::Backspace => {
                self.buf_mut().pop();
            }
            KeyCode::Tab if !is_letter => self.field = self.field.next(),
            KeyCode::Char('u') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                self.buf_mut().clear()
            }
            KeyCode::Char(c) => self.buf_mut().push(c),
            _ => {}
        }
        if !is_letter {
            self.refresh_rows();
        }
    }

    fn key_tags(&mut self, k: KeyEvent) {
        let cur = self.tag_list.selected().unwrap_or(0);
        let last = self.tags.len().saturating_sub(1);
        match k.code {
            KeyCode::Char('j') | KeyCode::Down => self.tag_list.select(Some((cur + 1).min(last))),
            KeyCode::Char('k') | KeyCode::Up => self.tag_list.select(Some(cur.saturating_sub(1))),
            KeyCode::Char(' ') | KeyCode::Enter => {
                if let Some(t) = self.tags.get_mut(cur) {
                    t.active = !t.active;
                }
            }
            KeyCode::Char('a') => self.tags.iter_mut().for_each(|t| t.active = true),
            KeyCode::Char('n') => self.tags.iter_mut().for_each(|t| t.active = false),
            KeyCode::Char('o') => {
                for (i, t) in self.tags.iter_mut().enumerate() {
                    t.active = i == cur;
                }
            }
            KeyCode::Esc | KeyCode::Tab | KeyCode::Char('t') => self.focus = Focus::Main,
            KeyCode::Char('q') => self.quit = true,
            _ => {}
        }
        self.refresh_rows();
    }

    fn key_main(&mut self, k: KeyEvent) {
        let text_view = matches!(self.view, View::FreeKeys | View::FreeCombos);
        match k.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Esc => {
                if self.view != View::Cheatsheet {
                    self.set_view(View::Cheatsheet);
                }
            }
            KeyCode::Char('?') => self.help = true,
            KeyCode::Char('r') => self.reload(),
            KeyCode::Char('m') => self.compact = !self.compact,
            KeyCode::Char('T') => self.tags_visible = !self.tags_visible,
            KeyCode::Char('1') => self.set_view(View::Cheatsheet),
            KeyCode::Char('/') => {
                self.set_view(View::Search);
                self.focus = Focus::Input;
            }
            KeyCode::Char('i') if self.view == View::Search => self.focus = Focus::Input,
            KeyCode::Tab if self.view == View::Search => {
                self.field = self.field.next();
                self.refresh_rows();
            }
            KeyCode::Char('t') | KeyCode::Tab => {
                self.tags_visible = true;
                if self.tag_list.selected().is_none() && !self.tags.is_empty() {
                    self.tag_list.select(Some(0));
                }
                self.focus = Focus::Tags;
            }
            KeyCode::Char('f') => self.set_view(View::FreeKeys),
            KeyCode::Char('c') => self.set_view(View::Conflicts),
            KeyCode::Char('p') | KeyCode::Left if self.view == View::FreeKeys => {
                self.combo_idx = self
                    .combo_idx
                    .checked_sub(1)
                    .unwrap_or(self.combos.len().saturating_sub(1));
            }
            KeyCode::Char('n') | KeyCode::Right if self.view == View::FreeKeys => {
                if !self.combos.is_empty() {
                    self.combo_idx = (self.combo_idx + 1) % self.combos.len();
                }
            }
            KeyCode::Char('l') => {
                self.set_view(View::FreeCombos);
                self.focus = Focus::Input;
            }
            KeyCode::Char('s') if text_view => {
                self.submap_idx = (self.submap_idx + 1) % self.submaps.len().max(1);
                self.rebuild_combos();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if text_view {
                    self.scroll = self.scroll.saturating_add(1);
                } else {
                    self.move_sel(1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if text_view {
                    self.scroll = self.scroll.saturating_sub(1);
                } else {
                    self.move_sel(-1);
                }
            }
            KeyCode::PageDown => self.move_sel(10),
            KeyCode::PageUp => self.move_sel(-10),
            KeyCode::Char('g') | KeyCode::Home => self.select_abs(0),
            KeyCode::Char('G') | KeyCode::End => self.select_abs(usize::MAX),
            KeyCode::Enter if self.current().is_some() => self.detail = true,
            _ => {}
        }
    }

    fn move_sel(&mut self, delta: i32) {
        if self.rows.is_empty() {
            return;
        }
        let cur = self.table.selected().unwrap_or(0) as i32;
        let n = (cur + delta).clamp(0, self.rows.len() as i32 - 1);
        self.table.select(Some(n as usize));
    }

    fn select_abs(&mut self, i: usize) {
        if self.rows.is_empty() {
            return;
        }
        self.table.select(Some(i.min(self.rows.len() - 1)));
    }
}
