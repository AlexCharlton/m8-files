# m8-files

[![Crates.io](https://img.shields.io/crates/v/m8-files)](https://crates.io/crates/m8-files)
[![Docs.rs](https://docs.rs/m8-files/badge.svg)](https://docs.rs/m8-files)
[![CI](https://github.com/AlexCharlton/m8-files/actions/workflows/ci.yml/badge.svg)](https://github.com/AlexCharlton/m8-files/actions/workflows/ci.yml)

Reads [Dirtwave M8](https://dirtywave.com/) files into Rust structs.

Big thanks to [m8-js](https://github.com/whitlockjc/m8-js) who did all the real dirty work.

## Usage

Add to your `Cargo.toml`:
```
m8-files = "0.2"
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
- Support writes?
- Throw more parse errors
- Interpret FXCommand based on Instrument
- Displays: MixerSettings, EffectsSettings, Instrument, MidiSettings, MidiMapping

## Changelog
### 0.2
- Add V3 support
- Fix instrument alignment issues
