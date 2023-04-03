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

mod fx;
mod instrument;
mod reader;
mod scale;
mod settings;
mod theme;
mod version;
pub use fx::*;
pub use instrument::*;
use reader::*;
pub use scale::*;
pub use settings::*;
pub use theme::*;
pub use version::*;

use std::fmt;

use arr_macro::arr;
use byteorder::{ByteOrder, LittleEndian};

#[derive(PartialEq, Clone)]
pub struct Song {
    pub version: Version,
    pub directory: String,
    pub transpose: u8,
    pub tempo: f32,
    pub quantize: u8,
    pub name: String,
    pub key: u8,

    pub song: SongSteps,
    pub phrases: Vec<Phrase>,
    pub chains: Vec<Chain>,
    pub instruments: Vec<Instrument>,
    pub tables: Vec<Table>,
    pub grooves: Vec<Groove>,
    pub scales: Vec<Scale>,

    pub mixer_settings: MixerSettings,
    pub effects_settings: EffectsSettings,
    pub midi_settings: MidiSettings,
    pub midi_mappings: Vec<MidiMapping>,
}

impl fmt::Debug for Song {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Song")
            .field("version", &self.version)
            .field("directory", &self.directory)
            .field("name", &self.name)
            .field("tempo", &self.tempo)
            .field("transpose", &self.transpose)
            .field("quantize", &self.quantize)
            .field("key", &self.key)
            .field("song", &self.song)
            .field("chains", self.chains.get(0).unwrap_or(&Chain::default()))
            .field("phrases", self.phrases.get(0).unwrap_or(&Phrase::default()))
            .field(
                "instruments",
                self.instruments.get(0).unwrap_or(&Instrument::default()),
            )
            .field("tables", &self.tables[0])
            .field("grooves", &self.grooves[0])
            .field("scales", &self.scales[0])
            .field("mixer_settings", &self.mixer_settings)
            .field("effects_settings", &self.effects_settings)
            .field("midi_settings", &self.midi_settings)
            .finish()
    }
}

impl Song {
    const SIZE_PRIOR_TO_2_5: usize = 0x1A970;
    const SIZE: usize = 0x1AD09;
    const N_PHRASES: usize = 255;
    const N_CHAINS: usize = 255;
    const N_INSTRUMENTS: usize = 128;
    const N_TABLES: usize = 256;
    const N_GROOVES: usize = 32;
    const N_SCALES: usize = 16;
    const N_MIDI_MAPPINGS: usize = 128;

    pub fn read(reader: &mut impl std::io::Read) -> Result<Self> {
        let mut buf: Vec<u8> = vec![];
        reader.read_to_end(&mut buf).unwrap();
        let len = buf.len();
        let reader = Reader::new(buf);

        if len < Self::SIZE_PRIOR_TO_2_5 + Version::SIZE {
            return Err(ParseError(
                "File is not long enough to be a M8 song".to_string(),
            ));
        }
        let version = Version::from_reader(&reader)?;
        if version.at_least(2, 5) && len < Self::SIZE + Version::SIZE {
            return Err(ParseError(
                "File is not long enough to be a M8 song".to_string(),
            ));
        }

        if version.at_least(3, 0) {
            Self::from_reader3(&reader, version)
        } else {
            Self::from_reader2(&reader, version)
        }
    }

    fn from_reader2(reader: &Reader, version: Version) -> Result<Self> {
        let directory = reader.read_string(128);
        let transpose = reader.read();
        let tempo = LittleEndian::read_f32(reader.read_bytes(4));
        let quantize = reader.read();
        let name = reader.read_string(12);
        let midi_settings = MidiSettings::from_reader(reader)?;
        let key = reader.read();
        reader.read_bytes(18); // Skip
        let mixer_settings = MixerSettings::from_reader(reader)?;
        // println!("{:x}", reader.pos());

        let grooves = (0..Self::N_GROOVES)
            .map(|i| Groove::from_reader(reader, i as u8))
            .collect::<Result<Vec<Groove>>>()?;
        let song = SongSteps::from_reader(reader)?;
        let phrases = (0..Self::N_PHRASES)
            .map(|i| Phrase::from_reader(reader, i as u8, version))
            .collect::<Result<Vec<Phrase>>>()?;
        let chains = (0..Self::N_CHAINS)
            .map(|i| Chain::from_reader(reader, i as u8))
            .collect::<Result<Vec<Chain>>>()?;
        let tables = (0..Self::N_TABLES)
            .map(|i| Table::from_reader(reader, i as u8, version))
            .collect::<Result<Vec<Table>>>()?;
        let instruments = (0..Self::N_INSTRUMENTS)
            .map(|i| Instrument::from_reader2(reader, i as u8, version))
            .collect::<Result<Vec<Instrument>>>()?;

        reader.read_bytes(3); // Skip
        let effects_settings = EffectsSettings::from_reader(reader)?;
        reader.set_pos(0x1A5FE);
        let midi_mappings = (0..Self::N_MIDI_MAPPINGS)
            .map(|_| MidiMapping::from_reader(reader))
            .collect::<Result<Vec<MidiMapping>>>()?;

        let scales: Vec<Scale> = if version.at_least(2, 5) {
            reader.set_pos(0x1AA7E);
            (0..Self::N_SCALES)
                .map(|i| Scale::from_reader(reader, i as u8))
                .collect::<Result<Vec<Scale>>>()?
        } else {
            (0..Self::N_SCALES)
                .map(|i| -> Scale {
                    let mut s = Scale::default();
                    s.number = i as u8;
                    s
                })
                .collect()
        };

        Ok(Self {
            version,
            directory,
            transpose,
            tempo,
            quantize,
            name,
            midi_settings,
            key,
            mixer_settings,
            grooves,
            song,
            phrases,
            chains,
            tables,
            instruments,
            scales,
            effects_settings,
            midi_mappings,
        })
    }

    fn from_reader3(reader: &Reader, version: Version) -> Result<Self> {
        // TODO read groove, scale
        let directory = reader.read_string(128);
        let transpose = reader.read();
        let tempo = LittleEndian::read_f32(reader.read_bytes(4));
        let quantize = reader.read();
        let name = reader.read_string(12);
        let midi_settings = MidiSettings::from_reader(reader)?;
        let key = reader.read();
        reader.read_bytes(18); // Skip
        let mixer_settings = MixerSettings::from_reader(reader)?;
        // println!("{:x}", reader.pos());

        let grooves = (0..Self::N_GROOVES)
            .map(|i| Groove::from_reader(reader, i as u8))
            .collect::<Result<Vec<Groove>>>()?;
        let song = SongSteps::from_reader(reader)?;
        let phrases = (0..Self::N_PHRASES)
            .map(|i| Phrase::from_reader(reader, i as u8, version))
            .collect::<Result<Vec<Phrase>>>()?;
        let chains = (0..Self::N_CHAINS)
            .map(|i| Chain::from_reader(reader, i as u8))
            .collect::<Result<Vec<Chain>>>()?;
        let tables = (0..Self::N_TABLES)
            .map(|i| Table::from_reader(reader, i as u8, version))
            .collect::<Result<Vec<Table>>>()?;
        let instruments = (0..Self::N_INSTRUMENTS)
            .map(|i| Instrument::from_reader3(reader, i as u8, version))
            .collect::<Result<Vec<Instrument>>>()?;

        reader.read_bytes(3); // Skip
        let effects_settings = EffectsSettings::from_reader(reader)?;
        reader.set_pos(0x1A5FE);
        let midi_mappings = (0..Self::N_MIDI_MAPPINGS)
            .map(|_| MidiMapping::from_reader(reader))
            .collect::<Result<Vec<MidiMapping>>>()?;

        let scales: Vec<Scale> = if version.at_least(2, 5) {
            reader.set_pos(0x1AA7E);
            (0..Self::N_SCALES)
                .map(|i| Scale::from_reader(reader, i as u8))
                .collect::<Result<Vec<Scale>>>()?
        } else {
            (0..Self::N_SCALES)
                .map(|i| -> Scale {
                    let mut s = Scale::default();
                    s.number = i as u8;
                    s
                })
                .collect()
        };

        Ok(Self {
            version,
            directory,
            transpose,
            tempo,
            quantize,
            name,
            midi_settings,
            key,
            mixer_settings,
            grooves,
            song,
            phrases,
            chains,
            tables,
            instruments,
            scales,
            effects_settings,
            midi_mappings,
        })
    }
}

#[derive(PartialEq, Clone)]
pub struct SongSteps {
    pub steps: [u8; 2048],
}
impl SongSteps {
    pub fn print_screen(&self) -> String {
        self.print_screen_from(0)
    }

    pub fn print_screen_from(&self, start: u8) -> String {
        (start..start + 16).fold("   1  2  3  4  5  6  7  8  \n".to_string(), |s, row| {
            s + &self.print_row(row) + "\n"
        })
    }

    pub fn print_row(&self, row: u8) -> String {
        let start = row as usize * 8;
        (start..start + 8).fold(format!("{row:02x} "), |s, b| -> String {
            let v = self.steps[b];
            let repr = if v == 255 {
                format!("-- ")
            } else {
                format!("{:02x} ", v)
            };
            s + &repr
        })
    }

    fn from_reader(reader: &Reader) -> Result<Self> {
        Ok(Self {
            steps: reader.read_bytes(2048).try_into().unwrap(),
        })
    }
}

impl fmt::Display for SongSteps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SONG\n\n{}", self.print_screen())
    }
}
impl fmt::Debug for SongSteps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

#[derive(PartialEq, Clone, Default)]
pub struct Chain {
    pub number: u8,
    pub steps: [ChainStep; 16],
}
impl Chain {
    pub fn print_screen(&self) -> String {
        (0..16).fold("  PH TSP\n".to_string(), |s, row| {
            s + &self.steps[row].print(row as u8) + "\n"
        })
    }

    fn from_reader(reader: &Reader, number: u8) -> Result<Self> {
        Ok(Self {
            number,
            steps: arr![ChainStep::from_reader(reader)?; 16],
        })
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CHAIN {:02x}\n\n{}", self.number, self.print_screen())
    }
}
impl fmt::Debug for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct ChainStep {
    pub phrase: u8,
    pub transpose: u8,
}
impl Default for ChainStep {
    fn default() -> Self {
        Self {
            phrase: 255,
            transpose: 0,
        }
    }
}
impl ChainStep {
    pub fn print(&self, row: u8) -> String {
        if self.phrase == 255 {
            format!("{:x} -- 00", row)
        } else {
            format!("{:x} {:02x} {:02x}", row, self.phrase, self.transpose)
        }
    }

    fn from_reader(reader: &Reader) -> Result<Self> {
        Ok(Self {
            phrase: reader.read(),
            transpose: reader.read(),
        })
    }
}

#[derive(PartialEq, Clone, Default)]
pub struct Phrase {
    pub number: u8,
    pub steps: [Step; 16],
    version: Version,
}
impl Phrase {
    pub fn print_screen(&self) -> String {
        (0..16).fold("  N   V  I  FX1   FX2   FX3  \n".to_string(), |s, row| {
            s + &self.steps[row].print(row as u8, self.version) + "\n"
        })
    }

    fn from_reader(reader: &Reader, number: u8, version: Version) -> Result<Self> {
        Ok(Self {
            number,
            steps: arr![Step::from_reader(reader)?; 16],
            version,
        })
    }
}

impl fmt::Display for Phrase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PHRASE {:02x}\n\n{}", self.number, self.print_screen())
    }
}
impl fmt::Debug for Phrase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct Step {
    pub note: Note,
    pub velocity: u8,
    pub instrument: u8,
    pub fx1: FX,
    pub fx2: FX,
    pub fx3: FX,
}
impl Step {
    pub fn print(&self, row: u8, version: Version) -> String {
        let velocity = if self.velocity == 255 {
            format!("--")
        } else {
            format!("{:02x}", self.velocity)
        };
        let instrument = if self.instrument == 255 {
            format!("--")
        } else {
            format!("{:02x}", self.instrument)
        };
        format!(
            "{:x} {} {} {} {} {} {}",
            row,
            self.note,
            velocity,
            instrument,
            self.fx1.print(version),
            self.fx2.print(version),
            self.fx3.print(version)
        )
    }

    fn from_reader(reader: &Reader) -> Result<Self> {
        Ok(Self {
            note: Note(reader.read()),
            velocity: reader.read(),
            instrument: reader.read(),
            fx1: FX::from_reader(reader)?,
            fx2: FX::from_reader(reader)?,
            fx3: FX::from_reader(reader)?,
        })
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Note(pub u8);
impl Default for Note {
    fn default() -> Self {
        Note(255)
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 255 {
            write!(f, "---")
        } else if self.0 >= 0x80 {
            write!(f, "OFF") // This isn't really true for < V3
        } else {
            let oct = (self.0 / 12) + 1;
            let n = match self.0 % 12 {
                0 => "C-",
                1 => "C#",
                2 => "D-",
                3 => "D#",
                4 => "E-",
                5 => "F-",
                6 => "F#",
                7 => "G-",
                8 => "G#",
                9 => "A-",
                10 => "A#",
                11 => "B-",
                _ => panic!(),
            };
            write!(f, "{}{:X}", n, oct)
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct Table {
    pub number: u8,
    pub steps: [TableStep; 16],
    version: Version,
}
impl Table {
    pub fn print_screen(&self) -> String {
        (0..16).fold("  N  V  FX1   FX2   FX3  \n".to_string(), |s, row| {
            s + &self.steps[row].print(row as u8, self.version) + "\n"
        })
    }

    fn from_reader(reader: &Reader, number: u8, version: Version) -> Result<Self> {
        Ok(Self {
            number,
            steps: arr![TableStep::from_reader(reader)?; 16],
            version,
        })
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TABLE {:02x}\n\n{}", self.number, self.print_screen())
    }
}
impl fmt::Debug for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct TableStep {
    pub transpose: u8,
    pub velocity: u8,
    pub fx1: FX,
    pub fx2: FX,
    pub fx3: FX,
}
impl TableStep {
    pub fn print(&self, row: u8, version: Version) -> String {
        let transpose = if self.transpose == 255 {
            format!("--")
        } else {
            format!("{:02x}", self.transpose)
        };
        let velocity = if self.velocity == 255 {
            format!("--")
        } else {
            format!("{:02x}", self.velocity)
        };
        format!(
            "{:x} {} {} {} {} {}",
            row,
            transpose,
            velocity,
            self.fx1.print(version),
            self.fx2.print(version),
            self.fx3.print(version)
        )
    }

    fn from_reader(reader: &Reader) -> Result<Self> {
        Ok(Self {
            transpose: reader.read(),
            velocity: reader.read(),
            fx1: FX::from_reader(reader)?,
            fx2: FX::from_reader(reader)?,
            fx3: FX::from_reader(reader)?,
        })
    }
}

#[derive(PartialEq, Clone)]
pub struct Groove {
    pub number: u8,
    pub steps: [u8; 16],
}
impl Groove {
    fn from_reader(reader: &Reader, number: u8) -> Result<Self> {
        Ok(Self {
            number,
            steps: reader.read_bytes(16).try_into().unwrap(),
        })
    }

    pub fn active_steps(&self) -> &[u8] {
        let end = (&self.steps).iter().position(|&x| x == 255).unwrap_or(15);
        &self.steps[0..end]
    }
}

impl fmt::Display for Groove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Groove {}:{:?}", self.number, self.active_steps())
    }
}
impl fmt::Debug for Groove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::fs::File;

    fn test_file() -> Song {
        let mut f = File::open("./examples/songs/TEST-FILE.m8s").expect("Could not open TEST-FILE");
        Song::read(&mut f).expect("Could not parse TEST-FILE")
    }

    #[test]
    fn test_instrument_reading() {
        let test_file = test_file();
        // dbg!(&test_file.instruments[0..8]);
        assert!(match &test_file.instruments[0] {
            Instrument::None => true,
            _ => false,
        });
        assert!(
            match &test_file.instruments[1] {
                Instrument::WavSynth(s) => {
                    assert_eq!(s.transpose, true);
                    assert_eq!(s.size, 0x20);
                    assert_eq!(s.synth_params.mixer_reverb, 0xD0);
                    assert!(match s.synth_params.mods[0] {
                        Mod::AHDEnv(_) => true,
                        _ => false,
                    });
                    assert!(match s.synth_params.mods[1] {
                        Mod::ADSREnv(_) => true,
                        _ => false,
                    });
                    assert!(match s.synth_params.mods[2] {
                        Mod::DrumEnv(_) => true,
                        _ => false,
                    });
                    assert!(match s.synth_params.mods[3] {
                        Mod::LFO(_) => true,
                        _ => false,
                    });

                    true
                }
                _ => false,
            },
            "Should be a WavSynth"
        );
        assert!(match &test_file.instruments[2] {
            Instrument::MacroSynth(s) => {
                assert_eq!(s.transpose, false);
                assert!(match s.synth_params.mods[0] {
                    Mod::TrigEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[1] {
                    Mod::TrackingEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[2] {
                    Mod::LFO(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[3] {
                    Mod::LFO(_) => true,
                    _ => false,
                });

                true
            }
            _ => false,
        });
        assert!(match &test_file.instruments[3] {
            Instrument::Sampler(s) => {
                assert!(match s.synth_params.mods[0] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[1] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[2] {
                    Mod::LFO(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[3] {
                    Mod::LFO(_) => true,
                    _ => false,
                });

                assert_eq!(&s.name, "SAMP");
                assert_eq!(
                    &s.sample_path,
                    "/Samples/Drums/Hits/TR505/bass drum 505.wav"
                );

                true
            }
            _ => false,
        });
        assert!(match &test_file.instruments[4] {
            Instrument::FMSynth(s) => {
                assert!(match s.synth_params.mods[0] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[1] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[2] {
                    Mod::LFO(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[3] {
                    Mod::LFO(_) => true,
                    _ => false,
                });

                true
            }
            _ => false,
        });
        assert!(match &test_file.instruments[5] {
            Instrument::HyperSynth(s) => {
                assert!(match s.synth_params.mods[0] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[1] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[2] {
                    Mod::LFO(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[3] {
                    Mod::LFO(_) => true,
                    _ => false,
                });
                assert_eq!(s.scale, 0xFF);
                assert_eq!(s.chord[0], 0x01);
                assert_eq!(s.chord[6], 0x3C);

                true
            }
            _ => false,
        });
        assert!(match &test_file.instruments[6] {
            Instrument::MIDIOut(s) => {
                assert!(match s.mods[0] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.mods[1] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.mods[2] {
                    Mod::LFO(_) => true,
                    _ => false,
                });
                assert!(match s.mods[3] {
                    Mod::LFO(_) => true,
                    _ => false,
                });
                true
            }
            _ => false,
        });
        assert!(match &test_file.instruments[7] {
            Instrument::External(s) => {
                assert!(match s.synth_params.mods[0] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[1] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[2] {
                    Mod::LFO(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[3] {
                    Mod::LFO(_) => true,
                    _ => false,
                });

                assert_eq!(s.cca.number, 1);
                assert_eq!(s.ccb.number, 2);
                assert_eq!(s.ccd.number, 4);
                true
            }
            _ => false,
        });
    }

    #[test]
    fn test_mixer_reading() {
        let test_file = test_file();
        // dbg!(&test_file.mixer_settings);
        assert_eq!(test_file.mixer_settings.track_volume[0], 0xE0);
        assert_eq!(test_file.mixer_settings.track_volume[7], 0xE0);
        assert_eq!(test_file.mixer_settings.dj_filter, 0x80);
        assert_eq!(test_file.mixer_settings.dj_filter_type, 0x02);
    }

    #[test]
    fn test_song_reading() {
        let test_file = test_file();
        // dbg!(&test_file);
        assert_eq!(test_file.name, "TEST-FILE");
        assert_eq!(test_file.tempo, 120.0);
        assert_eq!(test_file.transpose, 0x0C);
        assert_eq!(test_file.quantize, 0x02);
    }
}
