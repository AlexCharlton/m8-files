use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{writer::Writer, Version};

use super::{M8Result, Mod, ParseError, Reader};


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

const LFO_COMMAND_NAMES : [[&'static str; 5]; 4] =
  [
    ["LA1", "LO1", "LS1", "LF1", "LT1"],
    ["LA2", "LO2", "LS2", "LF2", "LT2"],
    ["LA3", "LO3", "LS3", "LF3", "LT3"],
    ["LA4", "LO4", "LS4", "LF4", "LT4"],
  ];

impl LFO {
    pub fn command_name(_ver: Version, mod_id: usize) -> &'static[&'static str] {
        &LFO_COMMAND_NAMES[mod_id]
    }

    pub fn from_reader2(reader: &mut Reader) -> M8Result<Self> {
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

    pub fn from_reader3(reader: &mut Reader, dest: u8) -> M8Result<Self> {
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

    pub fn to_mod(self) -> Mod { Mod::LFO(self) }
}
