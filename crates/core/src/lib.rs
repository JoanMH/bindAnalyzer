//! `bindanalyzer-core`: modelo, parser y consultas sobre los binds de Hyprland.
//!
//! Dos fuentes de datos se combinan en un [`Snapshot`]:
//!
//! - el fichero de configuración (`hyprland.conf` y sus `source =`), que aporta
//!   los `#TAGS:`, la línea original y su posición;
//! - `hyprctl binds -j`, que es la verdad sobre lo que Hyprland tiene cargado.
//!
//! Sobre el snapshot se ejecutan las consultas de [`query`]: búsqueda, teclas
//! libres, combinaciones libres para una tecla y conflictos.

pub mod config;
pub mod hyprctl;
pub mod model;
pub mod query;
pub mod snapshot;

pub use model::{Bind, BindFlags, Key, ModMask, Origin, SourceRef};
pub use snapshot::{default_config_path, load, LiveStatus, LoadOptions, Snapshot};
