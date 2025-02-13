use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{reader::*, writer::Writer};

#[repr(u8)]
#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum EqType {
    #[default]
    LowCut = 0,
    LowShelf = 1,
    Bell = 2,
    BandPass = 3,
    HiShelf = 4,
    HiCut = 5
}

const EQ_TYPE_STR : [&'static str; 6] =
    [
        "LOWCUT",
        "LOWSHELF",
        "BELL",
        "BANDPASS",
        "HI.SHELF",
        "HI.CUT"
    ];

#[repr(u8)]
#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum EqMode {
    #[default]
    Stereo = 0,
    Mid = 1,
    Side = 2,
    Left = 3,
    Right = 4
}

const EQ_MODE_STR : [&'static str; 5] =
    [
        "STEREO",
        "MID",
        "SIDE",
        "LEFT",
        "RIGHT"
    ];

#[derive(PartialEq, Eq, Clone, Debug, Copy, Default)]
pub struct EqModeType(pub u8);

impl EqModeType {
    pub fn new(ty: EqType, mode : EqMode) -> EqModeType {
        EqModeType(ty as u8 | ((mode as u8) << 5))
    }

    pub fn eq_mode_hex(&self) -> u8 {
        (self.0 >> 5)& 0x7
    }

    pub fn eq_type(&self) -> EqType {
        EqType::try_from(self.eq_type_hex())
            .unwrap_or(EqType::Bell)
    }

    pub fn eq_type_hex(&self) -> u8 { self.0 & 0x7 }

    pub fn eq_mode(&self) -> EqMode {
        EqMode::try_from(self.eq_mode_hex())
            .unwrap_or(EqMode::Stereo)
    }

    pub fn mode_str(&self) -> &'static str {
        let index = self.eq_mode_hex() as usize;
        EQ_MODE_STR.get(index).unwrap_or(&"")
    }

    pub fn type_str(&self) -> &'static str {
        let index = self.eq_type_hex() as usize;
        EQ_TYPE_STR.get(index).unwrap_or(&"")
    }
}

#[derive(PartialEq, Clone, Debug, Default)]
pub struct EqBand {
    pub mode      : EqModeType,

    pub freq_fin  : u8,
    pub freq      : u8,

    pub level_fin : u8,
    pub level     : u8,

    pub q         : u8
}

impl EqBand {
    const V4_SIZE : usize = 6;

    pub fn default_low() -> EqBand {
        let freq = 100 as usize;
        EqBand {
            mode: EqModeType::new(EqType::LowShelf, EqMode::Stereo),
            freq: (freq >> 8) as u8,
            freq_fin: (freq & 0xFF) as u8,

            level_fin: 0,
            level: 0,

            q: 50
        }
    }

    pub fn default_mid() -> EqBand {
        let freq = 1000 as usize;
        EqBand {
            mode: EqModeType::new(EqType::Bell, EqMode::Stereo),
            freq: (freq >> 8) as u8,
            freq_fin: (freq & 0xFF) as u8,

            level_fin: 0,
            level: 0,

            q: 50
        }
    }

    pub fn default_high() -> EqBand {
        let freq = 5000 as usize;
        EqBand {
            mode: EqModeType::new(EqType::HiShelf, EqMode::Stereo),
            freq: (freq >> 8) as u8,
            freq_fin: (freq & 0xFF) as u8,

            level_fin: 0,
            level: 0,

            q: 50
        }
    }

    pub fn is_empty(&self) -> bool {
        self.level == 0 && self.level_fin == 0
    }

    pub fn gain(&self) -> f64 {
        let int_gain = ((self.level as i16) << 8) | (self.level_fin as i16);
        (int_gain as f64) / 100.0
    }

    pub fn frequency(&self) -> usize {
        ((self.freq as usize) << 8) | self.freq_fin as usize
    }

    pub fn write(&self, w: &mut Writer) {
        w.write(self.mode.0);
        w.write(self.freq_fin);
        w.write(self.freq);
        w.write(self.level_fin);
        w.write(self.level);
        w.write(self.q);
    }

    pub fn from_reader(reader: &mut Reader) -> EqBand {
        let mode = EqModeType(reader.read());
        let freq_fin = reader.read();
        let freq = reader.read();
        let level_fin = reader.read();
        let level = reader.read();
        let q = reader.read();

        Self { level, level_fin, freq, freq_fin, mode, q }
    }
}

#[derive(PartialEq, Clone, Debug, Default)]
pub struct Equ {
    pub low : EqBand,
    pub mid : EqBand,
    pub high : EqBand
}

impl Equ {
    pub const V4_SIZE : usize = 3 * EqBand::V4_SIZE;

    pub fn is_empty(&self) -> bool {
        self.low == EqBand::default_low() &&
        self.mid == EqBand::default_mid() &&
        self.high == EqBand::default_high()
    }

    pub fn clear(&mut self) {
        self.low = EqBand::default_low();
        self.mid = EqBand::default_mid();
        self.high = EqBand::default_high();
    }

    pub fn write(&self, w: &mut Writer) {
        self.low.write(w);
        self.mid.write(w);
        self.high.write(w);
    }

    pub fn from_reader(reader: &mut Reader) -> Equ {
        let low = EqBand::from_reader(reader);
        let mid = EqBand::from_reader(reader);
        let high = EqBand::from_reader(reader);
        Self { low, mid, high }
    }
}
