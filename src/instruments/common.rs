use std::fmt;

use super::modulator::*;
use crate::reader::*;
use crate::writer::Writer;
use crate::Version;
use arr_macro::arr;

/// Type storing transpose field and eq number
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub struct TranspEq {
    pub transpose: bool,
    pub eq: u8,
}

impl TranspEq {
    pub fn from(ver: Version, transpose: bool, eq: u8) -> TranspEq {
        if ver.at_least(4, 1) {
            Self {
                transpose,
                eq: 0x00,
            }
        } else {
            Self { transpose, eq }
        }
    }

    pub fn from_version(ver: Version, value: u8) -> Self {
        if ver.at_least(4, 1) {
            Self {
                transpose: (value & 1) != 0,
                eq: 0x00,
            }
        } else {
            Self {
                transpose: (value & 1) != 0,
                eq: value >> 1,
            }
        }
    }
}

impl From<TranspEq> for u8 {
    fn from(value: TranspEq) -> Self {
        (if value.transpose { 1 } else { 0 }) | (value.eq << 1)
    }
}

#[rustfmt::skip] // Keep constats with important order vertical for maintenance
const LIMIT_TYPE : [&str; 8] = [
    "CLIP",
    "SIN",
    "FOLD",
    "WRAP",
    "POST",
    "POSTAD",
    "POST:W1",
    "POST:W2"
];

#[derive(PartialEq, Clone, Copy)]
pub struct LimitType(pub u8);

impl fmt::Debug for LimitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(LIMIT_TYPE[self.0 as usize])
    }
}

impl TryFrom<u8> for LimitType {
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

    pub associated_eq: u8,

    pub mods: [Mod; SynthParams::MODULATOR_COUNT],
}

#[rustfmt::skip] // Keep constats with important order vertical for maintenance
pub(crate) const COMMON_FILTER_TYPES : [&'static str; 8] = [
    "OFF",
    "LOWPASS",
    "HIGHPAS",
    "BANDPAS",
    "BANDSTP",
    "LP > HP",
    "ZDF LP",
    "ZDF HP",
];

impl SynthParams {
    pub const MODULATOR_COUNT: usize = 4;

    pub fn set_eq(&mut self, eq: u8) {
        self.associated_eq = eq
    }

    pub fn mod_only2(_reader: &mut Reader) -> M8Result<Self> {
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

            associated_eq: 0xFF,
            mods: arr![AHDEnv::default().to_mod(); 4],
        })
    }

    pub fn mod_only3(reader: &mut Reader, mod_offset: usize) -> M8Result<Self> {
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
            associated_eq: 0xFF,

            mods,
        })
    }

    pub fn from_reader2(
        reader: &mut Reader,
        volume: u8,
        pitch: u8,
        fine_tune: u8,
    ) -> M8Result<Self> {
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

            associated_eq: 0xFF,

            mods: [
                AHDEnv::from_reader2(reader)?.to_mod(),
                AHDEnv::from_reader2(reader)?.to_mod(),
                LFO::from_reader2(reader)?.to_mod(),
                LFO::from_reader2(reader)?.to_mod(),
            ],
        })
    }

    pub fn write(&self, ver: Version, w: &mut Writer, mod_offset: usize) {
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

        let writer_pos = w.pos();
        if ver.at_least(4, 1) {
            w.seek(writer_pos + mod_offset - 1);
            w.write(self.associated_eq);
        }

        w.seek(writer_pos + mod_offset);
        for m in &self.mods {
            m.write(w);
        }
    }

    pub fn write_modes(&self, w: &mut Writer, mod_offset: usize) {
        w.seek(w.pos() + mod_offset);
        for m in &self.mods {
            m.write(w);
        }
    }

    pub fn from_reader3(
        version: Version,
        reader: &mut Reader,
        volume: u8,
        pitch: u8,
        fine_tune: u8,
        eq: u8,
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

        let reader_pos = reader.pos();
        let associated_eq = if version.at_least(4, 1) {
            reader.set_pos(reader_pos + mod_offset - 1);
            reader.read()
        } else if version.at_least(4, 0) {
            eq
        } else {
            0xFF
        };

        reader.set_pos(reader_pos + mod_offset);

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

            associated_eq,

            mods,
        })
    }
}
