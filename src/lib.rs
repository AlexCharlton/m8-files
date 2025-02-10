//! This library lets you parse Dirtywave M8 data
//!
//! See, in particular, the `read` method available on:
//! - [`Song::read`]
//! - [`Instrument::read`]
//! - [`Scale::read`]
//! - [`Theme::read`]
//!
//! E.g.:
//! ```
//! use m8_files::*;
//!
//! let mut f = std::fs::File::open("./examples/songs/TEST-FILE.m8s").unwrap();
//! let song = Song::read(&mut f).unwrap();
//! dbg!(song);
//! ```
//!
mod songs;
mod eq;
mod fx;
mod instruments;
pub mod reader;
pub mod writer;
mod scale;
mod settings;
mod theme;
mod version;
pub mod remapper;

pub use fx::*;
pub use instruments::*;
pub use scale::*;
pub use settings::*;
pub use theme::*;
pub use songs::*;
pub use version::*;
