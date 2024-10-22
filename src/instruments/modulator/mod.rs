use crate::{reader::*, writer::Writer};
use adsr_env::ADSREnv;
use ahd_env::AHDEnv;
use drum_env::DrumEnv;
use lfo::LFO;
use tracking_env::TrackingEnv;
use trig_env::TrigEnv;

use super::Version;

pub mod ahd_env;
pub mod lfo;
pub mod adsr_env;
pub mod drum_env;
pub mod trig_env;
pub mod tracking_env;

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
    /// Size in bytes of the modulator
    const SIZE: usize = 6;

    /// Number of commands associated to each mode
    pub const COMMAND_PER_MOD : usize = 5;

    pub fn command_name(&self, ver: Version, mod_id: usize) -> &'static[&'static str] {
        match self {
            Mod::AHDEnv(_)  => AHDEnv::command_names(ver, mod_id),
            Mod::ADSREnv(_) => ADSREnv::command_name(ver, mod_id),
            Mod::DrumEnv(_) => DrumEnv::command_names(ver, mod_id),
            Mod::LFO(_) => LFO::command_name(ver, mod_id),
            Mod::TrigEnv(_) => TrigEnv::command_name(ver, mod_id),
            Mod::TrackingEnv(_) => TrackingEnv::command_name(ver, mod_id),
        }
    }

    pub fn from_reader(reader: &mut Reader) -> M8Result<Self> {
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
            x =>
                return Err(ParseError(format!("Unknown mod type {}", x))),
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

        w.seek(start + Self::SIZE);
    }
}
