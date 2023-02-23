# m8-files

Reads Dirtwave M8 files into Rust structs.

Big thanks to [m8-js](https://github.com/whitlockjc/m8-js) who did all the real dirty work.

## Usage

Add to your `Cargo.toml`:
```
m8-files = "0.1.0"
```

Load an example song:
```
$ cargo run --example read_song -- examples/songs/DEMO1.m8s
```

## TODO
- Support writes?
- Throw more parse errors
- Tests
- Interpret FXCommand based on Instrument
- Displays: MixerSettings, EffectsSettings, Instrument, MidiSettings, MidiMapping