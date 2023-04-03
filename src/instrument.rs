use crate::reader::*;
use crate::version::*;

use arr_macro::arr;

#[derive(PartialEq, Debug, Clone)]
pub enum Instrument {
    WavSynth(WavSynth),
    MacroSynth(MacroSynth),
    Sampler(Sampler),
    MIDIOut(MIDIOut),
    FMSynth(FMSynth),
    HyperSynth(HyperSynth),
    External(ExternalInst),
    None,
}
impl Default for Instrument {
    fn default() -> Self {
        Self::None
    }
}
impl Instrument {
    const SIZE: usize = 215;

    pub fn read(reader: &mut impl std::io::Read) -> Result<Self> {
        let mut buf: Vec<u8> = vec![];
        reader.read_to_end(&mut buf).unwrap();
        let len = buf.len();
        let reader = Reader::new(buf);

        if len < Self::SIZE + Version::SIZE {
            return Err(ParseError(
                "File is not long enough to be a M8 Instrument".to_string(),
            ));
        }
        let version = Version::from_reader(&reader)?;
        if version.at_least(3, 0) {
            Self::from_reader3(&reader, 0, version)
        } else {
            Self::from_reader2(&reader, 0, version)
        }
    }

    pub(crate) fn from_reader2(reader: &Reader, number: u8, version: Version) -> Result<Self> {
        let start_pos = reader.pos();
        let kind = reader.read();
        let name = reader.read_string(12);
        let transpose = reader.read_bool();
        let table_tick = reader.read();
        let (volume, pitch, fine_tune) = if kind != 3 {
            (reader.read(), reader.read(), reader.read())
        } else {
            (0, 0, 0)
        };

        let finalize = || -> () {
            reader.set_pos(start_pos + Self::SIZE);
        };

        Ok(match kind {
            0x00 => {
                // WavSyn
                let shape = reader.read();
                let size = reader.read();
                let mult = reader.read();
                let warp = reader.read();
                let mirror = reader.read();
                let synth_params = SynthParams::from_reader2(reader, volume, pitch, fine_tune)?;
                finalize();
                Self::WavSynth(WavSynth {
                    number,
                    name,
                    transpose,
                    table_tick,
                    synth_params,

                    shape,
                    size,
                    mult,
                    warp,
                    mirror,
                })
            }
            0x01 => {
                // MacroSyn
                let shape = reader.read();
                let timbre = reader.read();
                let color = reader.read();
                let degrade = reader.read();
                let redux = reader.read();
                let synth_params = SynthParams::from_reader2(reader, volume, pitch, fine_tune)?;
                finalize();
                Self::MacroSynth(MacroSynth {
                    number,
                    name,
                    transpose,
                    table_tick,
                    synth_params,

                    shape,
                    timbre,
                    color,
                    degrade,
                    redux,
                })
            }
            0x02 => {
                // Sampler
                let play_mode = reader.read();
                let slice = reader.read();
                let start = reader.read();
                let loop_start = reader.read();
                let length = reader.read();
                let degrade = reader.read();
                let synth_params = SynthParams::from_reader2(reader, volume, pitch, fine_tune)?;
                reader.set_pos(start_pos + 0x57);
                let sample_path = reader.read_string(128);
                finalize();
                Self::Sampler(Sampler {
                    number,
                    name,
                    transpose,
                    table_tick,
                    synth_params,

                    sample_path,
                    play_mode,
                    slice,
                    start,
                    loop_start,
                    length,
                    degrade,
                })
            }
            0x03 => {
                // MIDI
                let port = reader.read();
                let channel = reader.read();
                let bank_select = reader.read();
                let program_change = reader.read();
                reader.read_bytes(3); // discard
                let custom_cc: [ControlChange; 8] = arr![ControlChange::from_reader(reader)?; 8];
                let mods = arr![AHDEnv::default().to_mod(); 4];
                finalize();
                Self::MIDIOut(MIDIOut {
                    number,
                    name,
                    transpose,
                    table_tick,

                    port,
                    channel,
                    bank_select,
                    program_change,
                    custom_cc,
                    mods,
                })
            }
            0x04 => {
                // FM
                let algo = reader.read();
                let mut operators: [Operator; 4] = arr![Operator::default(); 4];
                if version.at_least(1, 4) {
                    for i in 0..4 {
                        operators[i].shape = reader.read();
                    }
                }
                for i in 0..4 {
                    operators[i].ratio = reader.read();
                    operators[i].ratio_fine = reader.read();
                }
                for i in 0..4 {
                    operators[i].level = reader.read();
                    operators[i].feedback = reader.read();
                }
                for i in 0..4 {
                    operators[i].mod_a = reader.read();
                }
                for i in 0..4 {
                    operators[i].mod_b = reader.read();
                }
                let mod1 = reader.read();
                let mod2 = reader.read();
                let mod3 = reader.read();
                let mod4 = reader.read();
                let synth_params = SynthParams::from_reader2(reader, volume, pitch, fine_tune)?;
                finalize();

                Self::FMSynth(FMSynth {
                    number,
                    name,
                    transpose,
                    table_tick,
                    synth_params,

                    algo,
                    operators,
                    mod1,
                    mod2,
                    mod3,
                    mod4,
                })
            }
            0xFF => {
                finalize();
                Self::None
            }
            _ => panic!("Instrument type {} not supported", kind),
        })
    }

    pub(crate) fn from_reader3(reader: &Reader, number: u8, version: Version) -> Result<Self> {
        let start_pos = reader.pos();
        // println!("inst start pos: {:02x} ({})", start_pos, start_pos);
        let kind = reader.read();
        let name = reader.read_string(12);
        let transpose = reader.read_bool();
        let table_tick = reader.read();
        let (volume, pitch, fine_tune) = if kind != 3 {
            (reader.read(), reader.read(), reader.read())
        } else {
            (0, 0, 0)
        };

        let finalize = || -> () {
            reader.set_pos(start_pos + Self::SIZE);
        };

        Ok(match kind {
            0x00 => {
                let shape = reader.read();
                let size = reader.read();
                let mult = reader.read();
                let warp = reader.read();
                let mirror = reader.read();
                let synth_params = SynthParams::from_reader3(reader, volume, pitch, fine_tune, 30)?;
                finalize();
                Self::WavSynth(WavSynth {
                    number,
                    name,
                    transpose,
                    table_tick,
                    synth_params,

                    shape,
                    size,
                    mult,
                    warp,
                    mirror,
                })
            }
            0x01 => {
                let shape = reader.read();
                let timbre = reader.read();
                let color = reader.read();
                let degrade = reader.read();
                let redux = reader.read();
                let synth_params = SynthParams::from_reader3(reader, volume, pitch, fine_tune, 30)?;
                finalize();
                Self::MacroSynth(MacroSynth {
                    number,
                    name,
                    transpose,
                    table_tick,
                    synth_params,

                    shape,
                    timbre,
                    color,
                    degrade,
                    redux,
                })
            }
            0x02 => {
                let play_mode = reader.read();
                let slice = reader.read();
                let start = reader.read();
                let loop_start = reader.read();
                let length = reader.read();
                let degrade = reader.read();
                let synth_params = SynthParams::from_reader3(reader, volume, pitch, fine_tune, 29)?;
                reader.set_pos(start_pos + 0x57);
                let sample_path = reader.read_string(128);
                finalize();
                Self::Sampler(Sampler {
                    number,
                    name,
                    transpose,
                    table_tick,
                    synth_params,

                    sample_path,
                    play_mode,
                    slice,
                    start,
                    loop_start,
                    length,
                    degrade,
                })
            }
            0x03 => {
                let port = reader.read();
                let channel = reader.read();
                let bank_select = reader.read();
                let program_change = reader.read();
                reader.read_bytes(3); // discard
                let custom_cc: [ControlChange; 8] = arr![ControlChange::from_reader(reader)?; 8];
                let _discard = reader.read_bytes(25);
                let mods = arr![Mod::from_reader(reader)?; 4];
                finalize();
                Self::MIDIOut(MIDIOut {
                    number,
                    name,
                    transpose,
                    table_tick,

                    port,
                    channel,
                    bank_select,
                    program_change,
                    custom_cc,
                    mods,
                })
            }
            0x04 => {
                let algo = reader.read();
                let mut operators: [Operator; 4] = arr![Operator::default(); 4];
                if version.at_least(1, 4) {
                    for i in 0..4 {
                        operators[i].shape = reader.read();
                    }
                }
                for i in 0..4 {
                    operators[i].ratio = reader.read();
                    operators[i].ratio_fine = reader.read();
                }
                for i in 0..4 {
                    operators[i].level = reader.read();
                    operators[i].feedback = reader.read();
                }
                for i in 0..4 {
                    operators[i].mod_a = reader.read();
                }
                for i in 0..4 {
                    operators[i].mod_b = reader.read();
                }
                let mod1 = reader.read();
                let mod2 = reader.read();
                let mod3 = reader.read();
                let mod4 = reader.read();
                let synth_params = SynthParams::from_reader3(reader, volume, pitch, fine_tune, 2)?;
                finalize();

                Self::FMSynth(FMSynth {
                    number,
                    name,
                    transpose,
                    table_tick,
                    synth_params,

                    algo,
                    operators,
                    mod1,
                    mod2,
                    mod3,
                    mod4,
                })
            }
            0x05 => {
                // HyperSynth

                let chord = arr![reader.read(); 7];
                let scale = reader.read();
                let shift = reader.read();
                let swarm = reader.read();
                let width = reader.read();
                let subosc = reader.read();
                let synth_params = SynthParams::from_reader3(reader, volume, pitch, fine_tune, 23)?;

                finalize();
                Self::HyperSynth(HyperSynth {
                    number,
                    name,
                    transpose,
                    table_tick,
                    synth_params,

                    scale,
                    chord,
                    shift,
                    swarm,
                    width,
                    subosc,
                })
            }
            0x06 => {
                // External
                let input = reader.read();
                let port = reader.read();
                let channel = reader.read();
                let bank = reader.read();
                let program = reader.read();
                let cca = ControlChange::from_reader(reader)?;
                let ccb = ControlChange::from_reader(reader)?;
                let ccc = ControlChange::from_reader(reader)?;
                let ccd = ControlChange::from_reader(reader)?;
                let synth_params = SynthParams::from_reader3(reader, volume, pitch, fine_tune, 22)?;
                finalize();
                Self::External(ExternalInst {
                    number,
                    name,
                    transpose,
                    table_tick,
                    synth_params,

                    input,
                    port,
                    channel,
                    bank,
                    program,
                    cca,
                    ccb,
                    ccc,
                    ccd,
                })
            }
            0xFF => {
                finalize();
                Self::None
            }
            _ => panic!("Instrument type {} not supported", kind),
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct WavSynth {
    pub number: u8,
    pub name: String,
    pub transpose: bool,
    pub table_tick: u8,
    pub synth_params: SynthParams,

    pub shape: u8,
    pub size: u8,
    pub mult: u8,
    pub warp: u8,
    pub mirror: u8,
}

#[derive(PartialEq, Debug, Clone)]
pub struct MacroSynth {
    pub number: u8,
    pub name: String,
    pub transpose: bool,
    pub table_tick: u8,
    pub synth_params: SynthParams,

    pub shape: u8,
    pub timbre: u8,
    pub color: u8,
    pub degrade: u8,
    pub redux: u8,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Sampler {
    pub number: u8,
    pub name: String,
    pub transpose: bool,
    pub table_tick: u8,
    pub synth_params: SynthParams,

    pub sample_path: String,
    pub play_mode: u8,
    pub slice: u8,
    pub start: u8,
    pub loop_start: u8,
    pub length: u8,
    pub degrade: u8,
}

#[derive(PartialEq, Debug, Clone)]
pub struct FMSynth {
    pub number: u8,
    pub name: String,
    pub transpose: bool,
    pub table_tick: u8,
    pub synth_params: SynthParams,

    pub algo: u8,
    pub operators: [Operator; 4],
    pub mod1: u8,
    pub mod2: u8,
    pub mod3: u8,
    pub mod4: u8,
}

#[derive(PartialEq, Debug, Clone)]
pub struct MIDIOut {
    pub number: u8,
    pub name: String,
    pub transpose: bool,
    pub table_tick: u8,

    pub port: u8,
    pub channel: u8,
    pub bank_select: u8,
    pub program_change: u8,
    pub custom_cc: [ControlChange; 8],

    pub mods: [Mod; 4],
}

#[derive(PartialEq, Debug, Clone)]
pub struct HyperSynth {
    pub number: u8,
    pub name: String,
    pub transpose: bool,
    pub table_tick: u8,
    pub synth_params: SynthParams,

    pub scale: u8,
    pub chord: [u8; 7],
    pub shift: u8,
    pub swarm: u8,
    pub width: u8,
    pub subosc: u8,
}

#[derive(PartialEq, Debug, Clone)]
pub struct ExternalInst {
    pub number: u8,
    pub name: String,
    pub transpose: bool,
    pub table_tick: u8,
    pub synth_params: SynthParams,

    pub input: u8,
    pub port: u8,
    pub channel: u8,
    pub bank: u8,
    pub program: u8,
    pub cca: ControlChange,
    pub ccb: ControlChange,
    pub ccc: ControlChange,
    pub ccd: ControlChange,
}

#[derive(PartialEq, Debug, Clone)]
pub struct SynthParams {
    pub volume: u8,
    pub pitch: u8,
    pub fine_tune: u8,

    pub filter_type: u8,
    pub filter_cutoff: u8,
    pub filter_res: u8,

    pub amp: u8,
    pub limit: u8,

    pub mixer_pan: u8,
    pub mixer_dry: u8,
    pub mixer_chorus: u8,
    pub mixer_delay: u8,
    pub mixer_reverb: u8,

    pub mods: [Mod; 4],
}
impl SynthParams {
    fn from_reader2(reader: &Reader, volume: u8, pitch: u8, fine_tune: u8) -> Result<Self> {
        Ok(Self {
            volume,
            pitch,
            fine_tune,

            filter_type: reader.read(),
            filter_cutoff: reader.read(),
            filter_res: reader.read(),

            amp: reader.read(),
            limit: reader.read(),

            mixer_pan: reader.read(),
            mixer_dry: reader.read(),
            mixer_chorus: reader.read(),
            mixer_delay: reader.read(),
            mixer_reverb: reader.read(),

            mods: [
                AHDEnv::from_reader2(reader)?.to_mod(),
                AHDEnv::from_reader2(reader)?.to_mod(),
                LFO::from_reader2(reader)?.to_mod(),
                LFO::from_reader2(reader)?.to_mod(),
            ],
        })
    }

    fn from_reader3(
        reader: &Reader,
        volume: u8,
        pitch: u8,
        fine_tune: u8,
        mod_offset: usize,
    ) -> Result<Self> {
        let filter_type = reader.read();
        let filter_cutoff = reader.read();
        let filter_res = reader.read();

        let amp = reader.read();
        let limit = reader.read();

        let mixer_pan = reader.read();
        let mixer_dry = reader.read();
        let mixer_chorus = reader.read();
        let mixer_delay = reader.read();
        let mixer_reverb = reader.read();
        let _discard = reader.read_bytes(mod_offset);
        let mods = arr![Mod::from_reader(reader)?; 4];

        Ok(Self {
            volume,
            pitch,
            fine_tune,

            filter_type,
            filter_cutoff,
            filter_res,

            amp,
            limit,

            mixer_pan,
            mixer_dry,
            mixer_chorus,
            mixer_delay,
            mixer_reverb,

            mods,
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Mod {
    AHDEnv(AHDEnv),
    ADSREnv(ADSREnv),
    DrumEnv(DrumEnv),
    LFO(LFO),
    TrigEnv(TrigEnv),
    TrackingEnv(TrackingEnv),
}

impl Mod {
    const SIZE: usize = 6;
    fn from_reader(reader: &Reader) -> Result<Self> {
        let start_pos = reader.pos();
        let first_byte = reader.read();
        let ty = first_byte >> 4;
        let dest = first_byte & 0x0F;

        // dbg!(ty, dest, start_pos);
        let r = match ty {
            0 => Mod::AHDEnv(AHDEnv::from_reader3(reader, dest)?),
            1 => Mod::ADSREnv(ADSREnv::from_reader(reader, dest)?),
            2 => Mod::DrumEnv(DrumEnv::from_reader(reader, dest)?),
            3 => Mod::LFO(LFO::from_reader3(reader, dest)?),
            4 => Mod::TrigEnv(TrigEnv::from_reader(reader, dest)?),
            5 => Mod::TrackingEnv(TrackingEnv::from_reader(reader, dest)?),
            x => panic!("Unknown mod type {}", x),
        };

        reader.set_pos(start_pos + Self::SIZE);
        Ok(r)
    }
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct AHDEnv {
    pub dest: u8,
    pub amount: u8,
    pub attack: u8,
    pub hold: u8,
    pub decay: u8,
}
impl AHDEnv {
    fn from_reader2(reader: &Reader) -> Result<Self> {
        let r = Self {
            dest: reader.read(),
            amount: reader.read(),
            attack: reader.read(),
            hold: reader.read(),
            decay: reader.read(),
        };
        reader.read();
        Ok(r)
    }

    fn from_reader3(reader: &Reader, dest: u8) -> Result<Self> {
        Ok(Self {
            dest,
            amount: reader.read(),
            attack: reader.read(),
            hold: reader.read(),
            decay: reader.read(),
        })
    }

    fn to_mod(self) -> Mod {
        Mod::AHDEnv(self)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct LFO {
    pub shape: u8,
    pub dest: u8,
    pub trigger_mode: u8,
    pub freq: u8,
    pub amount: u8,
}
impl LFO {
    fn from_reader2(reader: &Reader) -> Result<Self> {
        let r = Self {
            shape: reader.read(),
            dest: reader.read(),
            trigger_mode: reader.read(),
            freq: reader.read(),
            amount: reader.read(),
        };
        reader.read();
        Ok(r)
    }

    fn from_reader3(reader: &Reader, dest: u8) -> Result<Self> {
        Ok(Self {
            dest,
            amount: reader.read(),
            shape: reader.read(),
            trigger_mode: reader.read(),
            freq: reader.read(),
        })
    }

    fn to_mod(self) -> Mod {
        Mod::LFO(self)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct ADSREnv {
    pub dest: u8,
    pub amount: u8,
    pub attack: u8,
    pub decay: u8,
    pub sustain: u8,
    pub release: u8,
}
impl ADSREnv {
    fn from_reader(reader: &Reader, dest: u8) -> Result<Self> {
        Ok(Self {
            dest,
            amount: reader.read(),
            attack: reader.read(),
            decay: reader.read(),
            sustain: reader.read(),
            release: reader.read(),
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct DrumEnv {
    pub dest: u8,
    pub amount: u8,
    pub peak: u8,
    pub body: u8,
    pub decay: u8,
}
impl DrumEnv {
    fn from_reader(reader: &Reader, dest: u8) -> Result<Self> {
        Ok(Self {
            dest,
            amount: reader.read(),
            peak: reader.read(),
            body: reader.read(),
            decay: reader.read(),
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct TrigEnv {
    pub dest: u8,
    pub amount: u8,
    pub attack: u8,
    pub hold: u8,
    pub decay: u8,
    pub src: u8,
}
impl TrigEnv {
    fn from_reader(reader: &Reader, dest: u8) -> Result<Self> {
        Ok(Self {
            dest,
            amount: reader.read(),
            attack: reader.read(),
            hold: reader.read(),
            decay: reader.read(),
            src: reader.read(),
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct TrackingEnv {
    pub dest: u8,
    pub amount: u8,
    pub src: u8,
    pub lval: u8,
    pub hval: u8,
}
impl TrackingEnv {
    fn from_reader(reader: &Reader, dest: u8) -> Result<Self> {
        Ok(Self {
            dest,
            amount: reader.read(),
            src: reader.read(),
            lval: reader.read(),
            hval: reader.read(),
        })
    }
}

#[derive(PartialEq, Debug, Default, Clone)]
pub struct Operator {
    pub shape: u8,
    pub ratio: u8,
    pub ratio_fine: u8,
    pub level: u8,
    pub feedback: u8,
    pub retrigger: u8,
    pub mod_a: u8,
    pub mod_b: u8,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct ControlChange {
    pub number: u8,
    pub value: u8,
}
impl ControlChange {
    fn from_reader(reader: &Reader) -> Result<Self> {
        Ok(Self {
            number: reader.read(),
            value: reader.read(),
        })
    }
}
