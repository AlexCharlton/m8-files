use crate::reader::*;
use crate::version::*;
use crate::instruments::common::*;
use crate::writer::Writer;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

use super::dests;
use super::CommandPack;

/// Macro synth oscilator modes.
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
    FLUTED,
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
    PARTICLE_NOISE,
    DIGITAL_MOD,
    MORSE_NOISE,
}

const MACRO_SYNTH_COMMANDS : [&'static str;  CommandPack::BASE_INSTRUMENT_COMMAND_COUNT + 1] =
  [
    "VOL",
    "PIT",
    "FIN",
    "OSC",
    "TBR",
    "COL",
    "DEG",
    "RED",
    "FIL",
    "CUT",
    "RES",
    "AMP",
    "LIM",
    "PAN",
    "DRY",

    "SCH",
    "SDL",
    "SRV",

    // EXTRA command
    "TRG"
  ];

const DESTINATIONS : [&'static str; 15] =
    [
        dests::OFF,
        dests::VOLUME,
        dests::PITCH,

        "TIMBRE",
        "COLOR",
        dests::DEGRADE,
        "REDUX",
        dests::CUTOFF,
        dests::RES,
        dests::AMP,
        dests::PAN,
        dests::MOD_AMT,
        dests::MOD_RATE,
        dests::MOD_BOTH,
        dests::MOD_BINV,
    ];

#[derive(PartialEq, Debug, Clone)]
pub struct MacroSynth {
    pub number: u8,
    pub name: String,               // 12
    pub transpose: bool,            // 1
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

    pub fn command_name(&self, _ver: Version) -> &'static [&'static str] {
        &MACRO_SYNTH_COMMANDS 
    }

    pub fn destination_names(&self, _ver: Version) -> &'static [&'static str] {
        &DESTINATIONS
    }

    pub fn human_readable_filter(&self) -> &'static str {
        COMMON_FILTER_TYPES[self.synth_params.filter_type as usize]
    }

    pub fn write(&self, ver: Version, w: &mut Writer) {
        w.write_string(&self.name, 12);
        w.write(TranspEq::from(ver, self.transpose, self.synth_params.associated_eq).into());
        w.write(self.table_tick);
        w.write(self.synth_params.volume);
        w.write(self.synth_params.pitch);
        w.write(self.synth_params.fine_tune);

        w.write(self.shape.into());
        w.write(self.timbre);
        w.write(self.color);
        w.write(self.degrade);
        w.write(self.redux);

        self.synth_params.write(ver, w, MacroSynth::MOD_OFFSET);
    }

    pub fn from_reader(ver: Version, reader: &mut Reader, number: u8, version: Version) -> M8Result<Self> {
        let ms_pos = reader.pos();
        let name = reader.read_string(12);

        let transp_eq = TranspEq::from_version(ver, reader.read());
        let table_tick = reader.read();
        let volume = reader.read();
        let pitch = reader.read();
        let fine_tune = reader.read();

        let ofs_shape = reader.pos();
        let shape = reader.read();
        let timbre = reader.read();
        let color = reader.read();
        let degrade = reader.read();
        let redux = reader.read();

        let synth_params = 
            if version.at_least(3, 0) {
                SynthParams::from_reader3(ver, reader,  volume, pitch, fine_tune, transp_eq.eq, MacroSynth::MOD_OFFSET)?
            } else {
                SynthParams::from_reader2(reader, volume, pitch, fine_tune)?
            };

        let nc = name.clone();
        Ok(MacroSynth {
            number,
            name,
            transpose: transp_eq.transpose,
            table_tick,
            synth_params,

            shape: shape.try_into().map_err(|_| ParseError(format!("I{number:X} Wrong macrosynth@{ms_pos} ({nc}) shape {shape}@0x{ofs_shape}")))?,
            timbre,
            color,
            degrade,
            redux,
        })
    }
}
