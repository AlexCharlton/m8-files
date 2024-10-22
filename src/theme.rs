use crate::reader::*;
use crate::version::*;

#[derive(PartialEq, Debug, Clone)]
pub struct Theme {
    pub background: RGB,
    pub text_empty: RGB,
    pub text_info: RGB,
    pub text_default: RGB,
    pub text_value: RGB,
    pub text_title: RGB,
    pub play_marker: RGB,
    pub cursor: RGB,
    pub selection: RGB,
    pub scope_slider: RGB,
    pub meter_low: RGB,
    pub meter_mid: RGB,
    pub meter_peak: RGB,
}
impl Theme {
    const SIZE: usize = 39;

    pub fn read(reader: &mut impl std::io::Read) -> M8Result<Self> {
        let mut buf: Vec<u8> = vec![];
        reader.read_to_end(&mut buf).unwrap();
        let len = buf.len();
        let mut reader = Reader::new(buf);

        if len < Self::SIZE + Version::SIZE {
            return Err(ParseError(
                "File is not long enough to be a M8 Theme".to_string(),
            ));
        }
        Version::from_reader(&mut reader)?;
        Self::from_reader(&mut reader)
    }

    fn from_reader(reader: &mut Reader) -> M8Result<Self> {
        Ok(Self {
            background: RGB::from_reader(reader)?,
            text_empty: RGB::from_reader(reader)?,
            text_info: RGB::from_reader(reader)?,
            text_default: RGB::from_reader(reader)?,
            text_value: RGB::from_reader(reader)?,
            text_title: RGB::from_reader(reader)?,
            play_marker: RGB::from_reader(reader)?,
            cursor: RGB::from_reader(reader)?,
            selection: RGB::from_reader(reader)?,
            scope_slider: RGB::from_reader(reader)?,
            meter_low: RGB::from_reader(reader)?,
            meter_mid: RGB::from_reader(reader)?,
            meter_peak: RGB::from_reader(reader)?,
        })
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    fn from_reader(reader: &mut Reader) -> M8Result<Self> {
        Ok(Self {
            r: reader.read(),
            g: reader.read(),
            b: reader.read(),
        })
    }
}
