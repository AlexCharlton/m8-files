use crate::reader::*;
use crate::version::*;
use crate::instruments::common::*;
use crate::writer::Writer;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

use super::dests;
use super::CommandPack;

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
    pub transpose: bool,
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

const SAMPLER_FX_COMMANDS : [&'static str; CommandPack::BASE_INSTRUMENT_COMMAND_COUNT + 1] =
  [
    "VOL",
    "PIT",
    "FIN",
    "PLY",
    "STA",
    "LOP",
    "LEN",
    "DEG",
    "FLT",
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
    "SLI"
  ];

const DESTINATIONS : [&'static str; 14] =
    [
        dests::OFF,
        dests::VOLUME,
        dests::PITCH,

        "LOOP ST",
        "LENGTH",
        dests::DEGRADE,
        dests::CUTOFF,
        dests::RES,
        dests::AMP,
        dests::PAN,
        dests::MOD_AMT,
        dests::MOD_RATE,
        dests::MOD_BOTH,
        dests::MOD_BINV,
    ];

impl Sampler {
    pub const MOD_OFFSET : usize = 29;

    pub fn command_name(&self, _ver: Version) -> &'static[&'static str] {
        &SAMPLER_FX_COMMANDS 
    }

    pub fn destination_names(&self, _ver: Version) -> &'static [&'static str] {
        &DESTINATIONS
    }

    pub fn write(&self, ver: Version, w: &mut Writer) {
        let pos = w.pos();
        w.write_string(&self.name, 12);
        w.write(TranspEq::from(ver, self.transpose, self.synth_params.associated_eq).into());
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

        self.synth_params.write(ver, w, Sampler::MOD_OFFSET);

        w.seek(pos + 0x56);
        w.write_string(&self.sample_path, 128);
    }

    pub fn from_reader(ver: Version, reader: &mut Reader, start_pos: usize, number: u8, version: Version) -> M8Result<Self> {
        let name = reader.read_string(12);

        let transp_eq = TranspEq::from_version(ver, reader.read());
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
                SynthParams::from_reader3(ver, reader, volume, pitch, fine_tune, transp_eq.eq, Sampler::MOD_OFFSET)?
            } else {
                SynthParams::from_reader2(reader, volume, pitch, fine_tune)?
            };

        reader.set_pos(start_pos + 0x57);
        let sample_path = reader.read_string(128);

        Ok(Sampler {
            number,
            name,
            transpose: transp_eq.transpose,
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
