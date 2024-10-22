use crate::{writer::Writer, Version};

use super::{M8Result, Mod, Reader};


#[derive(PartialEq, Debug, Clone, Default)]
pub struct AHDEnv {
    pub dest: u8,
    pub amount: u8,
    pub attack: u8,
    pub hold: u8,
    pub decay: u8,
}

const AHDENV_COMMAND_NAMES : [[&'static str; 5]; 4] =
  [
    ["EA1", "AT1", "HO1", "DE1", "ET1"],
    ["EA2", "AT2", "HO2", "DE2", "ET2"],
    ["EA3", "AT3", "HO3", "DE3", "ET3"],
    ["EA4", "AT4", "HO4", "DE4", "ET4"]
  ];

impl AHDEnv {
    pub fn command_names(_ver: Version, mod_id: usize) -> &'static[&'static str] {
        &AHDENV_COMMAND_NAMES[mod_id]
    }

    pub fn from_reader2(reader: &mut Reader) -> M8Result<Self> {
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

    pub fn from_reader3(reader: &mut Reader, dest: u8) -> M8Result<Self> {
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

    pub fn to_mod(self) -> Mod {
        Mod::AHDEnv(self)
    }
}
