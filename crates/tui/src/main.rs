//! `bindanalyzer`: chuleta y analizador de atajos de Hyprland en el terminal.

mod app;
mod ui;

use anyhow::{Context, Result};
use app::{App, View};
use bindanalyzer_core::{default_config_path, load, LoadOptions, Snapshot};
use clap::{Parser, ValueEnum};
use crossterm::event::{self, Event, KeyEventKind};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum StartView {
    Cheatsheet,
    Search,
    Free,
    Letter,
    Conflicts,
}

impl From<StartView> for View {
    fn from(v: StartView) -> View {
        match v {
            StartView::Cheatsheet => View::Cheatsheet,
            StartView::Search => View::Search,
            StartView::Free => View::FreeKeys,
            StartView::Letter => View::FreeCombos,
            StartView::Conflicts => View::Conflicts,
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "bindanalyzer",
    version,
    about = "Chuleta y analizador de atajos de teclado de Hyprland"
)]
struct Args {
    /// Fichero de configuración (por defecto ~/.config/hypr/hyprland.conf)
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// No consultar `hyprctl binds -j`, usar solo el fichero
    #[arg(long)]
    no_live: bool,

    /// Arrancar en modo compacto (menos columnas)
    #[arg(long)]
    compact: bool,

    /// Vista inicial
    #[arg(long, value_enum, default_value_t = StartView::Cheatsheet)]
    view: StartView,

    /// Volcar los binds en JSON y salir (para scripts, wofi, rofi...)
    #[arg(long)]
    json: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let opts = LoadOptions {
        config: args.config.unwrap_or_else(default_config_path),
        use_live: !args.no_live,
    };
    let snap = load(&opts).with_context(|| "no se pudo cargar la configuración")?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&to_json(&snap))?);
        return Ok(());
    }

    let mut app = App::new(opts, snap, args.compact, args.view.into());
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;
        if event::poll(Duration::from_millis(500))? {
            if let Event::Key(k) = event::read()? {
                if k.kind == KeyEventKind::Press {
                    app.handle_key(k);
                }
            }
        }
        app.tick();
        if app.quit {
            return Ok(());
        }
    }
}

fn to_json(snap: &Snapshot) -> serde_json::Value {
    let binds: Vec<serde_json::Value> = snap
        .binds
        .iter()
        .map(|b| {
            serde_json::json!({
                "combo": b.combo(),
                "mods": b.mods.names(),
                "modmask": b.mods.bits(),
                "key": b.key.display(),
                "dispatcher": b.dispatcher,
                "arg": b.arg,
                "action": b.action(),
                "submap": b.submap,
                "flags": b.flags.letters(),
                "description": b.description,
                "tags": b.tags,
                "file": b.source.as_ref().map(|s| s.file.display().to_string()),
                "line": b.source.as_ref().map(|s| s.line),
                "origin": format!("{:?}", b.origin).to_lowercase(),
            })
        })
        .collect();
    serde_json::json!({
        "config": snap.config_path.display().to_string(),
        "files": snap.files,
        "live": format!("{:?}", snap.live),
        "warnings": snap.warnings,
        "binds": binds,
    })
}
