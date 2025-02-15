use crate::reader::*;
use crate::version::*;

use std::fmt;

use arr_macro::arr;
use byteorder::{ByteOrder, LittleEndian};

#[derive(PartialEq, Clone)]
pub struct Scale {
    pub number: u8,
    pub name: String,
    pub notes: [NoteOffset; 12], // Offsets for notes C-B
}

impl Scale {
    const SIZE: usize = 32;

    pub fn read(reader: &mut impl std::io::Read) -> M8Result<Self> {
        let mut buf: Vec<u8> = vec![];
        reader.read_to_end(&mut buf).unwrap();
        let len = buf.len();
        let mut reader = Reader::new(buf);

        if len < Self::SIZE + Version::SIZE {
            return Err(ParseError(
                "File is not long enough to be a M8 Scale".to_string(),
            ));
        }
        Version::from_reader(&mut reader)?;
        Self::from_reader(&mut reader, 0)
    }

    pub(crate) fn from_reader(reader: &mut Reader, number: u8) -> M8Result<Self> {
        let map = LittleEndian::read_u16(reader.read_bytes(2));
        let mut notes = arr![NoteOffset::default(); 12];

        for (i, note) in notes.iter_mut().enumerate() {
            note.enabled = ((map >> i) & 0x1) == 1;
            let offset = f32::from(reader.read()) + (f32::from(reader.read()) / 100.0);
            note.semitones = offset;
        }

        let name = reader.read_string(16);
        Ok(Self {
            number,
            name,
            notes,
        })
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self {
            number: 0,
            name: "CHROMATIC".to_string(),
            notes: arr![NoteOffset::default(); 12],
        }
    }
}

impl fmt::Display for Scale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let notes = vec![
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        let offsets = self
            .notes
            .iter()
            .zip(notes.iter())
            .map(|(offset, note)| -> String {
                let s = if offset.enabled {
                    let sign = if offset.semitones < 0.0 { "-" } else { " " };
                    format!(" ON{}{:02.2}", sign, offset.semitones.abs())
                } else {
                    " -- -- --".to_string()
                };
                format!("{:<2}{}", note, &s)
            })
            .collect::<Vec<String>>()
            .join("\n");

        write!(
            f,
            "Scale {}\nKEY   C\n\n   EN OFFSET\n{}\n\nNAME  {}",
            self.number, offsets, &self.name
        )
    }
}
impl fmt::Debug for Scale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct NoteOffset {
    pub enabled: bool,
    pub semitones: f32, // Semitones.cents: -24.0-24.0
}
impl NoteOffset {
    fn default() -> Self {
        Self {
            enabled: true,
            semitones: 0.0,
        }
    }
}
