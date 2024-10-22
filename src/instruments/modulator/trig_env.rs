use crate::Version;

use super::{M8Result, Reader, Writer};


#[derive(PartialEq, Debug, Clone)]
pub struct TrigEnv {
    pub dest: u8,
    pub amount: u8,
    pub attack: u8,
    pub hold: u8,
    pub decay: u8,
    pub src: u8,
}

const TRIGENV_COMMAND_NAMES : [[&'static str; 5]; 4] =
  [
    ["EA1", "AT1", "HO1", "SU1", "ET1"],
    ["EA2", "AT2", "HO2", "SU2", "ET2"],
    ["EA3", "AT3", "HO3", "SU3", "ET3"],
    ["EA4", "AT4", "HO4", "SU4", "ET4"]
  ];

impl TrigEnv {
    pub fn command_name(_ver: Version, mod_id: usize) -> &'static[&'static str] {
        &TRIGENV_COMMAND_NAMES[mod_id]
    }

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
