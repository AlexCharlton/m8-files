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
//! For song writing and file manipulation, you will need to load
//! the whole file in memory, in order to be able to overwrite it
//! 
//! ```
//! use m8_files::*;
//!
//! let mut song_data = std::fs::read("./examples/songs/TEST-FILE.m8s").unwrap();
//! let song_reader = Reader::new(song_data.clone());
//! let mutsong = Song::read_from_reader(&mut f).unwrap();
//! 
//! // let's renumber an instrument
//! let mut remapper = remapper::default();
//! remapper.instrument_mapping.mapping[instrument] = to_instrument as u8;
//! remapper.instrument_mapping.to_move.push(instrument as u8);
//! remapper.renumber(&mut song.song);
//! 
//! dbg!(song);
//! 
//! let mut output_writer = Writer::new(song_data);
//! 
//! song.write(&mut output_writer).unwrap();
//! // ready to be written elsewhere
//! let output_song_data = output_writer.finish();
//! ```
//! 
//! You also can perform more complex copies of chain, that 
//! will copy intrument/eq/table definitions required to copy
//! a chain from a song to another
//! 
//! ```
//! let from_song = Song::read(/*... */).unwrap();
//! let to_song = Song::read(/*... */).unwrap();
//! let chain : u8 = 12;
//! let mapping =
//!     Remapper::create(&from_song, &to_song, iter::once(&chain)).unwrap();
//! // you can inspec the mapping here if needed.
//! // and apply the remapping
//! mapping.apply(&from_song, &mut to_song);
//! // you now have the chain 12 in to_song, and you can edit a song
//! // cell to write it
//! let final_chain = mapping.out_chain(chain);
//! to_song.song.steps[2] = final_chain;
//! ```
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
