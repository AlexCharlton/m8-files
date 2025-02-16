# m8-files

[![Crates.io](https://img.shields.io/crates/v/m8-files)](https://crates.io/crates/m8-files)
[![Docs.rs](https://docs.rs/m8-files/badge.svg)](https://docs.rs/m8-files)
[![CI](https://github.com/AlexCharlton/m8-files/actions/workflows/ci.yml/badge.svg)](https://github.com/AlexCharlton/m8-files/actions/workflows/ci.yml)

Reads [Dirtwave M8](https://dirtywave.com/) files into Rust structs.

Big thanks to [m8-js](https://github.com/whitlockjc/m8-js) who did all the real dirty work.

## Usage

Add to your `Cargo.toml`:
```
m8-files = "0.3.1"
```
Or
```
$ cargo add play-files
```


Load an example song:
```
$ cargo run --example read_song -- examples/songs/DEFAULT.m8s
```

## TODO
- Add song groove, scale, note_preview
- Add settings: output/speaker volume
- Throw more parse errors
- Displays: MixerSettings, EffectsSettings, Instrument, MidiSettings, MidiMapping

## Changelog

### 0.3.1

 - Fixing visibility of Eq types (was private in 0.3)
 - FmAlgo content is now publicly visible
 - Each instrument filter type can now be accessed through a methdo.
 - Parameters and modulation destination constants is now public.

### 0.3

- v4 reading
- v4 overwriting, you can load, modify elements and rewrite the same song.
  * Does not work with song other than v4/v4.1
- Added EQ with plotting
- Mapped all FX instruction depending on the names
- Mapped enums to many instrument parameters with human readable information

### 0.2
- Add V3 support
- Fix instrument alignment issues
