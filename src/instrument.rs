use crate::reader::*;
use crate::version::*;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

use arr_macro::arr;

#[derive(PartialEq, Debug, Clone, Default)]
pub enum Instrument {
    WavSynth(WavSynth),
    MacroSynth(MacroSynth),
    Sampler(Sampler),
    MIDIOut(MIDIOut),
    FMSynth(FMSynth),
    HyperSynth(HyperSynth),
    External(ExternalInst),
    #[default]
    None
}

const INSTRUMENT_MEMORY_SIZE : usize = 215;
// const MOD_OFFSET : usize = 0x3B;

impl Instrument {
    pub fn write(&self, w: &mut Writer) {
        match self {
            Instrument::WavSynth(ws)     => { w.write(0); ws.write(w); }
            Instrument::MacroSynth(ms) => { w.write(1); ms.write(w); }
            Instrument::Sampler(s)        => { w.write(2); s.write(w); }
            Instrument::MIDIOut(mo)       => { w.write(3); mo.write(w); }
            Instrument::FMSynth(fs)       => { w.write(4); fs.write(w); }
            Instrument::HyperSynth(hs) => { w.write(5); hs.write(w); }
            Instrument::External(ex) => { w.write(6); ex.write(w); }
            Instrument::None => w.write(0xFF),
        }
    }

    pub fn read(reader: &mut impl std::io::Read) -> M8Result<Self> {
        let mut buf: Vec<u8> = vec![];
        reader.read_to_end(&mut buf).unwrap();
        let len = buf.len();
        let mut reader = Reader::new(buf);

        if len < INSTRUMENT_MEMORY_SIZE + Version::SIZE {
            return Err(ParseError(
                "File is not long enough to be a M8 Instrument".to_string(),
            ));
        }

        let version = Version::from_reader(&mut reader)?;
        Self::from_reader(&mut reader, 0, version)
    }

    pub fn from_reader(reader: &mut Reader, number: u8, version: Version) -> M8Result<Self> {
        let start_pos = reader.pos();
        let kind = reader.read();

        let instr = match kind {
            0x00 => Self::WavSynth(WavSynth::from_reader(reader, number, version)?),
            0x01 => Self::MacroSynth(MacroSynth::from_reader(reader, number, version)?),
            0x02 => Self::Sampler(Sampler::from_reader(reader, start_pos, number, version)?),
            0x03 => Self::MIDIOut(MIDIOut::from_reader(reader, number, version)?),
            0x04 => Self::FMSynth(FMSynth::from_reader(reader, number, version)?),
            0x05 if version.at_least(3, 0) => Self::HyperSynth(HyperSynth::from_reader(reader, number)?),
            0x06 if version.at_least(3, 0) => Self::External(ExternalInst::from_reader(reader, number)?),
            0xFF => Self::None,
            _ => panic!("Instrument type {} not supported", kind),
        };

        reader.set_pos(start_pos + INSTRUMENT_MEMORY_SIZE);

        Ok(instr)
    }
}

/// Type storing transpose field and eq number
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub struct TranspEq {
    pub transpose : bool,
    pub eq : u8
}

impl From<TranspEq> for u8 {
    fn from(value: TranspEq) -> Self {
        (if value.transpose { 1 } else { 0 }) |
        (value.eq << 1)
    }
}

impl From<u8> for TranspEq {
    fn from(value: u8) -> Self {
        Self {
            transpose: (value & 1) != 0,
            eq: value >> 1
        }
    }
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum WavShape {
    #[default]
    PULSE12,
    PULSE25,
    PULSE50,
    PULSE75,
    SAW,
    TRIANGLE,
    SINE,
    NOISE_PITCHED,
    NOISE,
    WT_CRUSH,
    WT_FOLDING,
    WT_FREQ,
    WT_FUZZY,
    WT_GHOST,
    WT_GRAPHIC,
    WT_LFOPLAY,
    WT_LIQUID,
    WT_MORPHING,
    WT_MYSTIC,
    WT_STICKY,
    WT_TIDAL,
    WT_TIDY,
    WT_TUBE,
    WT_UMBRELLA,
    WT_UNWIND,
    WT_VIRAL,
    WT_WAVES,
    WT_DRIP,
    WT_FROGGY,
    WT_INSONIC,
    WT_RADIUS,
    WT_SCRATCH,
    WT_SMOOTH,
    WT_WOBBLE,
    WT_ASIMMTRY,
    WT_BLEEN,
    WT_FRACTAL,
    WT_GENTLE,
    WT_HARMONIC,
    WT_HYPNOTIC,
    WT_ITERATIV,
    WT_MICROWAV,
    WT_PLAITS01,
    WT_PLAITS02,
    WT_RISEFALL,
    WT_TONAL,
    WT_TWINE,
    WT_ALIEN,
    WT_CYBERNET,
    WT_DISORDR,
    WT_FORMANT,
    WT_HYPER,
    WT_JAGGED,
    WT_MIXED,
    WT_MULTIPLY,
    WT_NOWHERE,
    WT_PINBALL,
    WT_RINGS,
    WT_SHIMMER,
    WT_SPECTRAL,
    WT_SPOOKY,
    WT_TRANSFRM,
    WT_TWISTED,
    WT_VOCAL,
    WT_WASHED,
    WT_WONDER,
    WT_WOWEE,
    WT_ZAP,
    WT_BRAIDS,
    WT_VOXSYNTH
}

#[derive(PartialEq, Debug, Clone)]
pub struct WavSynth {
    pub number: u8,
    pub name: String,
    pub transp_eq: TranspEq,
    pub table_tick: u8,
    pub synth_params: SynthParams,

    pub shape: WavShape,
    pub size: u8,
    pub mult: u8,
    pub warp: u8,
    pub scan: u8,
}

impl WavSynth {
    pub const MOD_OFFSET : usize = 30;

    pub fn write(&self, w: &mut Writer) {
        w.write_string(&self.name[..], 12);
        w.write(self.transp_eq.into());
        w.write(self.table_tick);
        w.write(self.synth_params.volume);
        w.write(self.synth_params.pitch);
        w.write(self.synth_params.fine_tune);

        w.write(self.shape.into());
        w.write(self.size);
        w.write(self.mult);
        w.write(self.warp);
        w.write(self.scan);
        self.synth_params.write(w, WavSynth::MOD_OFFSET);
    }

    pub fn from_reader(reader: &mut Reader, number: u8, version: Version) -> M8Result<Self> {
        let name = reader.read_string(12);
        let transp_eq = reader.read().into();
        let table_tick = reader.read();
        let volume = reader.read();
        let pitch = reader.read();
        let fine_tune = reader.read();

        let shape = reader.read();
        let size = reader.read();
        let mult = reader.read();
        let warp = reader.read();
        let scan = reader.read();
        let synth_params = 
            if version.at_least(3, 0) {
                SynthParams::from_reader3(reader, volume, pitch, fine_tune, WavSynth::MOD_OFFSET)?
            } else {
                SynthParams::from_reader2(reader, volume, pitch, fine_tune)?
            };

        Ok(WavSynth {
            number,
            name,
            transp_eq,
            table_tick,
            synth_params,

            shape: shape
                .try_into()
                .map_err(|_| ParseError(format!("Invalid wavsynth shape")))?,
            size,
            mult,
            warp,
            scan,
        })
    }
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum MacroSynthOsc {
    #[default]
    CSAW,
    MORPH,
    SAW_SQUARE,
    SINE_TRIANGLE,
    BUZZ,
    SQUARE_SUB,
    SAW_SUB,
    SQUARE_SYNC,
    SAW_SYNC,
    TRIPLE_SAW,
    TRIPLE_SQUARE,
    TRIPLE_TRIANGLE,
    TRIPLE_SIN,
    TRIPLE_RNG,
    SAW_SWARM,
    SAW_COMB,
    TOY,
    DIGITAL_FILTER_LP,
    DIGITAL_FILTER_PK,
    DIGITAL_FILTER_BP,
    DIGITAL_FILTER_HP,
    VOSIM,
    VOWEL,
    VOWEL_FOF,
    HARMONICS,
    FM,
    FEEDBACK_FM,
    CHAOTIC_FEEDBACK_FM,
    PLUCKED,
    BOWED,
    BLOWN,
    STRUCK_BELL,
    STRUCK_DRUM,
    KICK,
    CYMBAL,
    SNARE,
    WAVETABLES,
    WAVE_MAP,
    WAV_LINE,
    WAV_PARAPHONIC,
    FILTERED_NOISE,
    TWIN_PEAKS_NOISE,
    CLOCKED_NOISE,
    GRANULAR_CLOUD,
    PARTICLE_NOISE
}

#[derive(PartialEq, Debug, Clone)]
pub struct MacroSynth {
    pub number: u8,
    pub name: String,               // 12
    pub transp_eq: TranspEq,        // 1
    pub table_tick: u8,             // 1
    pub synth_params: SynthParams,  // 10

    pub shape: MacroSynthOsc,       // 1
    pub timbre: u8,                 // 1
    pub color: u8,                  // 1
    pub degrade: u8,                // 1
    pub redux: u8,                  // 1
}

impl MacroSynth {
    pub const MOD_OFFSET : usize = 30;

    pub fn write(&self, w: &mut Writer) {
        w.write_string(&self.name, 12);
        w.write(self.transp_eq.into());
        w.write(self.table_tick);
        w.write(self.synth_params.volume);
        w.write(self.synth_params.pitch);
        w.write(self.synth_params.fine_tune);

        w.write(self.shape.into());
        w.write(self.timbre);
        w.write(self.color);
        w.write(self.degrade);
        w.write(self.redux);

        self.synth_params.write(w, MacroSynth::MOD_OFFSET);
    }

    pub fn from_reader(reader: &mut Reader, number: u8, version: Version) -> M8Result<Self> {
        let name = reader.read_string(12);

        let transp_eq = reader.read().into();
        let table_tick = reader.read();
        let volume = reader.read();
        let pitch = reader.read();
        let fine_tune = reader.read();

        let shape = reader.read();
        let timbre = reader.read();
        let color = reader.read();
        let degrade = reader.read();
        let redux = reader.read();

        let synth_params = 
            if version.at_least(3, 0) {
                SynthParams::from_reader3(reader, volume, pitch, fine_tune, MacroSynth::MOD_OFFSET)?
            } else {
                SynthParams::from_reader2(reader, volume, pitch, fine_tune)?
            };

        Ok(MacroSynth {
            number,
            name,
            transp_eq,
            table_tick,
            synth_params,

            shape: shape.try_into().map_err(|_| ParseError(format!("Wrong shape")))?,
            timbre,
            color,
            degrade,
            redux,
        })
    }
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum SamplePlayMode {
  #[default]
  FWD,
  REV,
  FWDLOOP,
  REVLOOP,
  FWD_PP,
  REV_PP,
  OSC,
  OSC_REV,
  OSC_PP
}

#[derive(PartialEq, Debug, Clone)]
pub struct Sampler {
    pub number: u8,
    pub name: String,
    pub transp_eq: TranspEq,
    pub table_tick: u8,
    pub synth_params: SynthParams,

    pub sample_path: String,
    pub play_mode: SamplePlayMode,
    pub slice: u8,
    pub start: u8,
    pub loop_start: u8,
    pub length: u8,
    pub degrade: u8,
}

impl Sampler {
    pub const MOD_OFFSET : usize = 29;

    pub fn write(&self, w: &mut Writer) {
        let pos = w.pos();
        w.write_string(&self.name, 12);
        w.write(self.transp_eq.into());
        w.write(self.table_tick);
        w.write(self.synth_params.volume);
        w.write(self.synth_params.pitch);
        w.write(self.synth_params.fine_tune);

        w.write(self.play_mode.into());
        w.write(self.slice);
        w.write(self.start);
        w.write(self.loop_start);
        w.write(self.length);
        w.write(self.degrade);

        self.synth_params.write(w, Sampler::MOD_OFFSET);

        w.write(((pos + 0x57) - w.pos()) as u8);
        w.write_string(&self.sample_path, 128);
    }

    pub fn from_reader(reader: &mut Reader, start_pos: usize, number: u8, version: Version) -> M8Result<Self> {
        let name = reader.read_string(12);

        let transp_eq = reader.read().into();
        let table_tick = reader.read();
        let volume = reader.read();
        let pitch = reader.read();
        let fine_tune = reader.read();

        let play_mode = reader.read();
        let slice = reader.read();
        let start = reader.read();
        let loop_start = reader.read();
        let length = reader.read();
        let degrade = reader.read();

        let synth_params =
            if version.at_least(3, 0) {
                SynthParams::from_reader3(reader, volume, pitch, fine_tune, Sampler::MOD_OFFSET)?
            } else {
                SynthParams::from_reader2(reader, volume, pitch, fine_tune)?
            };

        reader.set_pos(start_pos + 0x57);
        let sample_path = reader.read_string(128);

        Ok(Sampler {
            number,
            name,
            transp_eq,
            table_tick,
            synth_params,

            sample_path,
            play_mode: play_mode
                .try_into()
                .map_err(|_| ParseError(format!("Invalid play mode")))?,
            slice,
            start,
            loop_start,
            length,
            degrade,
        })
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct FmAlgo(u8);

const FM_ALGO_STRINGS : [&str; 0x0C] =
    [
        "A>B>C>D",
        "[A+B]>C>D",
        "[A>B+C]>D",
        "[A>B+A>C]>D",
        "[A+B+C]>D",
        "[A>B>C]+D",
        "[A>B>C]+[A>B>D]",
        "[A>B]+[C>D]",
        "[A>B]+[A>C]+[A>D]",
        "[A>B]+[A>C]+D",
        "[A>B]+C+D",
        "A+B+C+D"
    ];

impl TryFrom<u8> for FmAlgo {
    type Error = ParseError;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        if (value as usize) < FM_ALGO_STRINGS.len() {
            Ok(FmAlgo(value))
        } else {
            Err(ParseError(format!("Invalid fm algo {}", value)))
        }
    }
}

impl FmAlgo {
    pub fn id(self) -> u8 {
        let FmAlgo(v) = self;
        v
    }

    pub fn str(self) -> &'static str {
        FM_ALGO_STRINGS[self.id() as usize]
    }
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum FMWave {
    #[default]
    SIN,
    SW2,
    SW3,
    SW4,
    SW5,
    SW6,
    TRI,
    SAW,
    SQR,
    PUL,
    IMP,
    NOI
}

#[derive(PartialEq, Debug, Clone)]
pub struct FMSynth {
    pub number: u8,
    pub name: String,
    pub transp_eq: TranspEq,
    pub table_tick: u8,
    pub synth_params: SynthParams,

    pub algo: FmAlgo,
    pub operators: [Operator; 4],
    pub mod1: u8,
    pub mod2: u8,
    pub mod3: u8,
    pub mod4: u8,
}

impl FMSynth {
    const MOD_OFFSET : usize = 2;

    pub fn write(&self, w: &mut Writer) {
        w.write_string(&self.name, 12);
        w.write(self.transp_eq.into());
        w.write(self.table_tick);
        w.write(self.synth_params.volume);
        w.write(self.synth_params.pitch);
        w.write(self.synth_params.fine_tune);

        w.write(self.algo.0);

        for op in &self.operators {
            w.write(op.shape.into());
        }

        for op in &self.operators {
            w.write(op.ratio);
            w.write(op.ratio_fine);
        }

        for op in &self.operators {
            w.write(op.level);
            w.write(op.feedback);
        }

        for op in &self.operators {
            w.write(op.mod_a);
        }

        for op in &self.operators {
            w.write(op.mod_b);
        }

        w.write(self.mod1);
        w.write(self.mod2);
        w.write(self.mod3);
        w.write(self.mod4);

        self.synth_params.write(w, FMSynth::MOD_OFFSET);
    }

    pub fn from_reader(reader: &mut Reader, number: u8, version: Version) -> M8Result<Self> {
        let name = reader.read_string(12);
        let transp_eq = reader.read().into();
        let table_tick = reader.read();
        let volume = reader.read();
        let pitch = reader.read();
        let fine_tune = reader.read();

        let algo = reader.read();
        let mut operators: [Operator; 4] = arr![Operator::default(); 4];
        if version.at_least(1, 4) {
            for i in 0..4 {
                operators[i].shape = FMWave::try_from(reader.read()).map_err(|_| ParseError(format!("Invalid fm wave")))?;
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

        let synth_params =
            if version.at_least(3, 0) {
                SynthParams::from_reader3(reader, volume, pitch, fine_tune, FMSynth::MOD_OFFSET)?
            } else {
                SynthParams::from_reader2(reader, volume, pitch, fine_tune)?
            };

        Ok(FMSynth {
            number,
            name,
            transp_eq,
            table_tick,
            synth_params,

            algo: FmAlgo(algo),
            operators,
            mod1,
            mod2,
            mod3,
            mod4,
        })
    }
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

    pub mods: SynthParams,
}

impl MIDIOut {
    const MOD_OFFSET : usize = 25;

    pub fn write(&self, w: &mut Writer) {
        w.write_string(&self.name, 12);
        w.write(if self.transpose { 1 } else { 0 });
        w.write(self.table_tick);
        w.write(self.port);
        w.write(self.channel);
        w.write(self.bank_select);
        w.write(self.program_change);

        w.write(0);
        w.write(0);
        w.write(0);

        for cc in self.custom_cc {
            cc.write(w);
        }

        self.mods.write(w, MIDIOut::MOD_OFFSET)
    }

    pub fn from_reader(reader: &mut Reader, number: u8, version: Version) -> M8Result<Self> {
        let name = reader.read_string(12);
        let transpose = reader.read_bool();
        let table_tick = reader.read();

        let port = reader.read();
        let channel = reader.read();
        let bank_select = reader.read();
        let program_change = reader.read();
        reader.read_bytes(3); // discard
        let custom_cc: [ControlChange; 8] = arr![ControlChange::from_reader(reader)?; 8];
        let mods =
            if version.at_least(3, 0) {
                SynthParams::mod_only3(reader, MIDIOut::MOD_OFFSET)?
            } else {
                SynthParams::mod_only2(reader)?
            };

        Ok(MIDIOut {
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
}

#[derive(PartialEq, Debug, Clone)]
pub struct HyperSynth {
    pub number: u8,
    pub name: String,
    pub transp_eq: TranspEq,
    pub table_tick: u8,
    pub synth_params: SynthParams,

    pub scale: u8,
    pub default_chord: [u8; 7],
    pub shift: u8,
    pub swarm: u8,
    pub width: u8,
    pub subosc: u8,

    pub chords: [[u8; 6]; 0x10]
}

impl HyperSynth {
    const MOD_OFFSET : usize = 23;

    pub fn write(&self, w: &mut Writer) {
        w.write_string(&self.name, 12);
        w.write(self.transp_eq.into());
        w.write(self.table_tick);
        w.write(self.synth_params.volume);
        w.write(self.synth_params.pitch);
        w.write(self.synth_params.fine_tune);

        for c in self.default_chord {
            w.write(c);
        }

        w.write(self.shift);
        w.write(self.swarm);
        w.write(self.width);
        w.write(self.subosc);

        self.synth_params.write(w, HyperSynth::MOD_OFFSET);

        for chd in self.chords {
            w.write(0xFF);
            for k in chd { w.write(k); }
        }
    }

    fn load_chord(reader: &mut Reader) -> [u8; 6] {
        // padding
        let _ = reader.read();
        arr![reader.read(); 6]
    }

    pub fn from_reader(reader: &mut Reader, number: u8) -> M8Result<Self> {
        let name = reader.read_string(12);
        let transp_eq = reader.read().into();
        let table_tick = reader.read();
        let volume = reader.read();
        let pitch = reader.read();
        let fine_tune = reader.read();

        let default_chord = arr![reader.read(); 7];
        let scale = reader.read();
        let shift = reader.read();
        let swarm = reader.read();
        let width = reader.read();
        let subosc = reader.read();
        let synth_params =
            SynthParams::from_reader3(reader, volume, pitch, fine_tune, HyperSynth::MOD_OFFSET)?;

        let chords =
            arr![HyperSynth::load_chord(reader); 0x10];

        Ok(HyperSynth {
            number,
            name,
            transp_eq,
            table_tick,
            synth_params,

            scale,
            default_chord,
            shift,
            swarm,
            width,
            subosc,
            chords
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct ExternalInst {
    pub number: u8,
    pub name: String,
    pub transp_eq: TranspEq,
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

impl ExternalInst {
    const MOD_OFFSET: usize = 22;

    pub fn write(&self, w: &mut Writer) {
        w.write_string(&self.name, 12);
        w.write(self.transp_eq.into());
        w.write(self.table_tick);
        w.write(self.synth_params.volume);
        w.write(self.synth_params.pitch);
        w.write(self.synth_params.fine_tune);

        w.write(self.input);
        w.write(self.port);
        w.write(self.channel);
        w.write(self.bank);
        w.write(self.program);

        self.cca.write(w);
        self.ccb.write(w);
        self.ccc.write(w);
        self.ccd.write(w);

        self.synth_params.write(w, ExternalInst::MOD_OFFSET);
    }

    pub fn from_reader(reader: &mut Reader, number: u8) -> M8Result<Self> {

        let name = reader.read_string(12);
        let transp_eq = reader.read().into();
        let table_tick = reader.read();
        let volume = reader.read();
        let pitch = reader.read();
        let fine_tune = reader.read();

        let input = reader.read();
        let port = reader.read();
        let channel = reader.read();
        let bank = reader.read();
        let program = reader.read();
        let cca = ControlChange::from_reader(reader)?;
        let ccb = ControlChange::from_reader(reader)?;
        let ccc = ControlChange::from_reader(reader)?;
        let ccd = ControlChange::from_reader(reader)?;

        let synth_params =
            SynthParams::from_reader3(reader, volume, pitch, fine_tune, ExternalInst::MOD_OFFSET)?;

        Ok(ExternalInst {
            number,
            name,
            transp_eq,
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
}

const LIMIT_TYPE : [&str; 8] =
    [
       "CLIP",
       "SIN",
       "FOLD",
       "WRAP",
       "POST",
       "POSTAD",
       "POST:W1",
       "POST:W2"
    ];

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct LimitType(u8);

impl TryFrom<u8> for LimitType{
    type Error = ParseError;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        if (value as usize) < LIMIT_TYPE.len() {
            Ok(LimitType(value))
        } else {
            Err(ParseError(format!("Invalid fm wave {}", value)))
        }
    }
}

impl LimitType {
    pub fn id(self) -> u8 {
        let LimitType(v) = self;
        v
    }

    pub fn str(self) -> &'static str {
        LIMIT_TYPE[self.id() as usize]
    }
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
    pub limit: LimitType,

    pub mixer_pan: u8,
    pub mixer_dry: u8,
    pub mixer_chorus: u8,
    pub mixer_delay: u8,
    pub mixer_reverb: u8,

    pub mods: [Mod; 4],
}

impl SynthParams {
    fn mod_only2(_reader: &mut Reader) -> M8Result<Self>{
        Ok(Self {
            volume: 0,
            pitch: 0,
            fine_tune: 0,

            filter_type: 0,
            filter_cutoff: 0,
            filter_res: 0,

            amp: 0,
            limit: LimitType::try_from(0)?,

            mixer_pan: 0,
            mixer_dry: 0,
            mixer_chorus: 0,
            mixer_delay: 0,
            mixer_reverb: 0,

            mods: arr![AHDEnv::default().to_mod(); 4]
        })
    }

    fn mod_only3(reader: &mut Reader, mod_offset: usize) -> M8Result<Self> {
        reader.set_pos(reader.pos() + mod_offset);

        let mods = arr![Mod::from_reader(reader)?; 4];

        Ok(Self {
            volume: 0,
            pitch: 0,
            fine_tune: 0,

            filter_type: 0,
            filter_cutoff: 0,
            filter_res: 0,

            amp: 0,
            limit: LimitType::try_from(0)?,

            mixer_pan: 0,
            mixer_dry: 0,
            mixer_chorus: 0,
            mixer_delay: 0,
            mixer_reverb: 0,

            mods
        })
    }

    fn from_reader2(reader: &mut Reader, volume: u8, pitch: u8, fine_tune: u8) -> M8Result<Self> {
        Ok(Self {
            volume,
            pitch,
            fine_tune,

            filter_type: reader.read(),
            filter_cutoff: reader.read(),
            filter_res: reader.read(),

            amp: reader.read(),
            limit: LimitType::try_from(reader.read())?,

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

    pub fn write(&self, w: &mut Writer, mod_offset: usize) {
        w.write(self.filter_type);
        w.write(self.filter_cutoff);
        w.write(self.filter_res);

        w.write(self.amp);
        w.write(self.limit.0);

        w.write(self.mixer_pan);
        w.write(self.mixer_dry);
        w.write(self.mixer_chorus);
        w.write(self.mixer_delay);
        w.write(self.mixer_reverb);

        self.write_modes(w, mod_offset);
    }

    pub fn write_modes(&self, w: &mut Writer, mod_offset: usize) {
        w.fill_till(0xFF, mod_offset);
        for m in &self.mods { m.write(w); }
    }

    fn from_reader3(
        reader: &mut Reader,
        volume: u8,
        pitch: u8,
        fine_tune: u8,
        mod_offset: usize,
    ) -> M8Result<Self> {
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

        reader.set_pos(reader.pos() + mod_offset);

        let mods = arr![Mod::from_reader(reader)?; 4];

        Ok(Self {
            volume,
            pitch,
            fine_tune,

            filter_type,
            filter_cutoff,
            filter_res,

            amp,
            limit: LimitType::try_from(limit)?,

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

    fn from_reader(reader: &mut Reader) -> M8Result<Self> {
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

    pub fn write(&self, w: &mut Writer) {
        let start = w.pos();

        match self {
            Mod::AHDEnv(env) =>{
                w.write(env.dest);
                env.write(w);
            }
            Mod::ADSREnv(env) => {
                w.write(1 << 4 | env.dest);
                env.write(w);
            }
            Mod::DrumEnv(env) => {
                w.write(2 << 4 | env.dest);
                env.write(w);
            }
            Mod::LFO(lfo) => {
                w.write(3 << 4 | lfo.dest);
                lfo.write(w);
            }
            Mod::TrigEnv(env) => {
                w.write(4 << 4 | env.dest);
                env.write(w);
            }
            Mod::TrackingEnv(env) => {
                w.write(5 << 4 | env.dest);
                env.write(w);
            }
        }

        w.fill_till(0, start + Self::SIZE);
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
    fn from_reader2(reader: &mut Reader) -> M8Result<Self> {
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

    fn from_reader3(reader: &mut Reader, dest: u8) -> M8Result<Self> {
        Ok(Self {
            dest,
            amount: reader.read(),
            attack: reader.read(),
            hold: reader.read(),
            decay: reader.read(),
        })
    }

    pub fn write(&self, w: &mut Writer) {
        w.write(self.amount);
        w.write(self.attack);
        w.write(self.hold);
        w.write(self.decay);
    }

    fn to_mod(self) -> Mod {
        Mod::AHDEnv(self)
    }
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum LfoShape {
  #[default]
  TRI,
  SIN,
  RAMP_DOWN,
  RAMP_UP,
  EXP_DN,
  EXP_UP,
  SQR_DN,
  SQR_UP,
  RANDOM,
  DRUNK,
  TRI_T,
  SIN_T,
  RAMPD_T,
  RAMPU_T,
  EXPD_T,
  EXPU_T,
  SQ_D_T,
  SQ_U_T,
  RAND_T,
  DRNK_T
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum LfoTriggerMode {
  #[default]
  FREE,
  RETRIG,
  HOLD,
  ONCE
}

#[derive(PartialEq, Debug, Clone)]
pub struct LFO {
    pub shape: LfoShape,
    pub dest: u8,
    pub trigger_mode: LfoTriggerMode,
    pub freq: u8,
    pub amount: u8,
    pub retrigger: u8
}

impl LFO {
    fn from_reader2(reader: &mut Reader) -> M8Result<Self> {
        let shape = reader.read();
        let dest = reader.read();
        let trigger = reader.read();
        let r = Self {
            shape: shape
                .try_into()
                .map_err(|_| ParseError(format!("Invalid LFO shape {}", shape)))?,
            dest,
            trigger_mode: trigger
                .try_into()
                .map_err(|_| ParseError(format!("Invalid lfo trigger mode {}", trigger)))?,
            freq: reader.read(),
            amount: reader.read(),
            retrigger: reader.read()
        };

        Ok(r)
    }

    pub fn write(&self, w: &mut Writer) {
        w.write(self.amount);
        w.write(self.shape.into());
        w.write(self.trigger_mode.into());
        w.write(self.freq);
        w.write(self.retrigger);
    }

    fn from_reader3(reader: &mut Reader, dest: u8) -> M8Result<Self> {
        let amount = reader.read();
        let shape = reader.read();
        let trigger_mode = reader.read();
        let freq = reader.read();
        let retrigger = reader.read();

        Ok(Self {
            dest,
            amount,
            shape: shape
                .try_into()
                .map_err(|_| ParseError(format!("Invalid LFO shape {}", shape)))?,
            trigger_mode: trigger_mode
                .try_into()
                .map_err(|_| ParseError(format!("Invalid lfo trigger mode {}", trigger_mode)))?,
            freq,
            retrigger
        })
    }

    fn to_mod(self) -> Mod { Mod::LFO(self) }
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
    pub fn write(&self, w: &mut Writer) {
        w.write(self.amount);
        w.write(self.attack);
        w.write(self.decay);
        w.write(self.sustain);
        w.write(self.release);
    }

    fn from_reader(reader: &mut Reader, dest: u8) -> M8Result<Self> {
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
    pub fn write(&self, w: &mut Writer) {
        w.write(self.amount);
        w.write(self.peak);
        w.write(self.body);
        w.write(self.decay);
    }

    pub fn from_reader(reader: &mut Reader, dest: u8) -> M8Result<Self> {
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
    pub fn write(&self, w: &mut Writer) {
        w.write(self.amount);
        w.write(self.attack);
        w.write(self.hold);
        w.write(self.decay);
        w.write(self.src);
    }

    pub fn from_reader(reader: &mut Reader, dest: u8) -> M8Result<Self> {
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
    pub fn write(&self, writer: &mut Writer) {
        writer.write(self.amount);
        writer.write(self.src);
        writer.write(self.lval);
        writer.write(self.hval);
    }

    fn from_reader(reader: &mut Reader, dest: u8) -> M8Result<Self> {
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
    pub shape: FMWave,
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
    /// CC number (target)
    pub number: u8,

    /// Value to be sent via MIDI CC message
    pub value: u8,
}

impl ControlChange {
    pub fn write(self, writer: &mut Writer) {
        writer.write(self.number);
        writer.write(self.value);
    }

    pub fn from_reader(reader: &mut Reader) -> M8Result<Self> {
        Ok(Self {
            number: reader.read(),
            value: reader.read(),
        })
    }
}
