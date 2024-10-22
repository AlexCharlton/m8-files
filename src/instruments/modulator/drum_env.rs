use crate::{writer::Writer, Version};

use super::{M8Result, Reader};


const DRUMENV_COMMAND_NAMES : [[&'static str; 5]; 4] =
  [
    ["EA1", "PK1", "BO1", "DE1", "ET1"],
    ["EA2", "PK2", "BO2", "DE2", "ET2"],
    ["EA3", "PK3", "BO3", "DE3", "ET3"],
    ["EA4", "PK4", "BO4", "DE4", "ET4"],
  ];


#[derive(PartialEq, Debug, Clone)]
pub struct DrumEnv {
    pub dest: u8,
    pub amount: u8,
    pub peak: u8,
    pub body: u8,
    pub decay: u8,
}

impl DrumEnv {
    pub fn command_names(_ver: Version, mod_id: usize) -> &'static[&'static str] {
        &DRUMENV_COMMAND_NAMES[mod_id]
    }

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
