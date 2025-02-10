use crate::reader::*;
use crate::version::*;
use crate::instruments::common::*;
use crate::writer::Writer;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

use super::dests;
use super::CommandPack;

/// Wavsynth wave shape
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

const WAVSYNTH_COMMAND_NAMES : [&'static str; CommandPack::BASE_INSTRUMENT_COMMAND_COUNT] =
  [
    "VOL",
    "PIT",
    "FIN",
    "OSC",
    "SIZ",
    "MUL",
    "WRP",
    "MIR",
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
  ];

const DESTINATIONS : [&'static str; 15] =
    [
        dests::OFF,
        dests::VOLUME,
        dests::PITCH,

        "SIZE",
        "MULT",
        "WARP",
        "SCAN",
        dests::CUTOFF,
        dests::RES,
        dests::AMP,
        dests::PAN,
        dests::MOD_AMT,
        dests::MOD_RATE,
        dests::MOD_BOTH,
        dests::MOD_BINV,
    ];

const WAVSYNTH_FILTER_TYPES : [&'static str; 12] =
    [
        "OFF",
        "LOWPASS",
        "HIGHPAS",
        "BANDPAS",
        "BANDSTP",
        "LP > HP",
        "ZDF LP",
        "ZDF HP",
        "WAV LP",
        "WAV HP",
        "WAV BP",
        "WAV BS"
    ];

impl WavSynth {
    pub const MOD_OFFSET : usize = 30;

    pub fn command_name(&self, _ver: Version) -> &'static [&'static str] {
        &WAVSYNTH_COMMAND_NAMES
    }

    pub fn destination_names(&self, _ver: Version) -> &'static [&'static str] {
        &DESTINATIONS
    }

    pub fn human_readable_filter(&self) -> &'static str {
        WAVSYNTH_FILTER_TYPES [self.synth_params.filter_type as usize]
    }

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
