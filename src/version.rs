use crate::reader::*;

use std::fmt;

#[derive(PartialEq, Clone, Copy)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl Default for Version {
    fn default() -> Self {
        Self {
            major: 3,
            minor: 0,
            patch: 0,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

impl Version {
    pub(crate) const SIZE: usize = 14;

    pub(crate) fn from_reader(reader: &Reader) -> Result<Self> {
        let _version_string = reader.read_bytes(10);
        let lsb = reader.read();
        let msb = reader.read();
        let major = msb & 0x0F;
        let minor = (lsb >> 4) & 0x0F;
        let patch = lsb & 0x0F;

        reader.read_bytes(2); // Skip
        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    pub fn at_least(&self, major: u8, minor: u8) -> bool {
        self.major > major || (self.major == major && self.minor >= minor)
    }
}
