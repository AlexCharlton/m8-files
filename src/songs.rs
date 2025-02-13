use std::fmt;

use crate::eq::Equ;
use crate::fx::*;
use crate::instruments::*;
use crate::reader::*;
use crate::remapper::InstrumentMapping;
use crate::remapper::PhraseMapping;
use crate::scale::*;
use crate::settings::*;
use crate::version::*;
use crate::writer::Writer;

use arr_macro::arr;
use byteorder::{ByteOrder, LittleEndian};

pub struct Offsets {
    pub groove: usize,
    pub song: usize,
    pub phrases: usize,
    pub chains : usize,
    pub table : usize,
    pub instruments : usize,
    pub effect_settings : usize,
    pub midi_mapping: usize,
    pub scale: usize,
    pub eq: usize,

    /// Number of eq for the song (different between 4.0 & 4.1)
    pub instrument_eq_count : usize,

    /// For instrument size, where is the EQ information written
    /// (if any)
    pub instrument_file_eq_offset : Option<usize>
}

impl Offsets {
    pub fn eq_count(&self) -> usize {
        // general EQ + 3 for effects + 1 global
        self.instrument_eq_count + 3 + 1
    }
}

pub const V4_OFFSETS : Offsets = Offsets {
    groove: 0xEE,
    song: 0x2EE,
    phrases: 0xAEE,
    chains: 0x9A5E,
    table: 0xBA3E,
    instruments: 0x13A3E,
    effect_settings: 0x1A5C1,
    midi_mapping: 0x1A5FE,
    scale: 0x1AA7E,
    eq: 0x1AD5A + 4,
    instrument_eq_count: 32,
    instrument_file_eq_offset : None
};

pub const V4_1_OFFSETS : Offsets = Offsets {
    groove: 0xEE,
    song: 0x2EE,
    phrases: 0xAEE,
    chains: 0x9A5E,
    table: 0xBA3E,
    instruments: 0x13A3E,
    effect_settings: 0x1A5C1,
    midi_mapping: 0x1A5FE,
    scale: 0x1AA7E,
    eq: 0x1AD5A + 4,
    instrument_eq_count: 0x80,
    instrument_file_eq_offset: Some(0x165)
};


////////////////////////////////////////////////////////////////////////////////////
/// MARK: Song
////////////////////////////////////////////////////////////////////////////////////
#[derive(PartialEq, Clone)]
pub struct Song {
    pub version: Version,
    pub directory: String,
    pub transpose: u8,
    pub tempo: f32,
    pub quantize: u8,
    pub name: String,
    pub key: u8,

    pub song: SongSteps,
    pub phrases: Vec<Phrase>,
    pub chains: Vec<Chain>,
    pub instruments: Vec<Instrument>,
    pub tables: Vec<Table>,
    pub grooves: Vec<Groove>,
    pub scales: Vec<Scale>,

    pub mixer_settings: MixerSettings,
    pub effects_settings: EffectsSettings,
    pub midi_settings: MidiSettings,
    pub midi_mappings: Vec<MidiMapping>,
    pub eqs : Vec<Equ>
}

impl fmt::Debug for Song {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Song")
            .field("version", &self.version)
            .field("directory", &self.directory)
            .field("name", &self.name)
            .field("tempo", &self.tempo)
            .field("transpose", &self.transpose)
            .field("quantize", &self.quantize)
            .field("key", &self.key)
            .field("song", &self.song)
            .field("chains", self.chains.get(0).unwrap_or(&Chain::default()))
            .field("phrases", &self.phrase_view(0))
            .field(
                "instruments",
                self.instruments.get(0).unwrap_or(&Instrument::default()),
            )
            .field("tables", &self.table_view(0))
            .field("grooves", &self.grooves[0])
            .field("scales", &self.scales[0])
            .field("eqs", self.eqs.get(0).unwrap_or(&Equ::default() ) )
            .field("mixer_settings", &self.mixer_settings)
            .field("effects_settings", &self.effects_settings)
            .field("midi_settings", &self.midi_settings)
            .finish()
    }
}

impl Song {
    const SIZE_PRIOR_TO_2_5: usize = 0x1A970;
    const SIZE: usize = 0x1AD09;
    pub const N_PHRASES: usize = 255;
    pub const N_CHAINS: usize = 255;
    pub const N_INSTRUMENTS: usize = 128;
    pub const N_TABLES: usize = 256;
    pub const N_GROOVES: usize = 32;
    pub const N_SCALES: usize = 16;

    pub const N_MIDI_MAPPINGS: usize = 128;

    pub fn phrase_view(&self, ix: usize) -> PhraseView {
        PhraseView {
            phrase: &self.phrases[ix],
            instruments: &self.instruments
        }
    }

    pub fn offsets(&self) -> &'static Offsets {
        if self.version.at_least(4, 1) {
            &V4_1_OFFSETS
        } else {
            &V4_OFFSETS
        }
    }

    pub fn eq_count(&self) -> usize {
        self.offsets().eq_count()
    }

    pub fn table_view(&self, ix: usize) -> TableView {
        TableView {
            table: &self.tables[ix],
            instrument:
                if ix < Song::N_INSTRUMENTS {
                    self.instruments[ix].instr_command_text(self.version)
                } else {
                    CommandPack::default()
                }
        }
    }

    pub fn eq_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.eqs.iter()).finish()
    }

    pub fn read(reader: &mut impl std::io::Read) -> M8Result<Self> {
        let mut buf: Vec<u8> = vec![];
        reader.read_to_end(&mut buf).unwrap();
        let mut reader = Reader::new(buf);
        Self::read_from_reader(&mut reader)
    }

    pub fn read_from_reader(mut reader: &mut Reader) -> M8Result<Self> {
        if reader.len() < Self::SIZE_PRIOR_TO_2_5 + Version::SIZE {
            return Err(ParseError(
                "File is not long enough to be a M8 song".to_string(),
            ));
        }
        let version = Version::from_reader(&mut reader)?;
        if version.at_least(2, 5) && reader.len() < Self::SIZE + Version::SIZE {
            return Err(ParseError(
                "File is not long enough to be a M8 song".to_string(),
            ));
        }

        println!("Version: {}.{}", version.major, version.minor);

        Self::from_reader(&mut reader, version)
    }

    pub fn write(&self, w: &mut Writer) -> Result<(), String> {
        if !self.version.at_least(4, 0) {
            Err(String::from("Only version 4.0 or above song can be rewritten"))
        } else {
            self.write_patterns(V4_OFFSETS, w);
            Ok(())
        }
    }

    fn write_patterns(&self, ofs : Offsets, w : &mut Writer) {
        w.seek(ofs.song);
        w.write_bytes(&self.song.steps);

        w.seek(ofs.phrases);
        for ph in &self.phrases {
            ph.write(w);
        }

        w.seek(ofs.chains);
        for ch in &self.chains {
            ch.write(w);
        }

        w.seek(ofs.table);
        for table in &self.tables {
            table.write(w);
        }

        w.seek(ofs.instruments);
        for instr in &self.instruments {
            let pos = w.pos();
            instr.write(self.version, w);
            w.seek(pos + Instrument::INSTRUMENT_MEMORY_SIZE);
        }

        w.seek(ofs.eq);
        for eq in &self.eqs { eq.write(w); }
    }

    fn from_reader(reader: &mut Reader, version: Version) -> M8Result<Self> {
        // TODO read groove, scale
        let directory = reader.read_string(128);
        let transpose = reader.read();
        let tempo = LittleEndian::read_f32(reader.read_bytes(4));
        let quantize = reader.read();
        let name = reader.read_string(12);
        let midi_settings = MidiSettings::try_from(&mut *reader)?;
        let key = reader.read();
        reader.read_bytes(18); // Skip
        let mixer_settings = MixerSettings::from_reader(reader)?;

        let grooves = (0..Self::N_GROOVES)
            .map(|i| Groove::from_reader(reader, i as u8))
            .collect::<M8Result<Vec<Groove>>>()?;
        let song = SongSteps::from_reader(reader)?;
        let phrases = (0..Self::N_PHRASES)
            .map(|_| Phrase::from_reader(reader, version))
            .collect::<M8Result<Vec<Phrase>>>()?;
        let chains = (0..Self::N_CHAINS)
            .map(|_| Chain::from_reader(reader))
            .collect::<M8Result<Vec<Chain>>>()?;
        let tables = (0..Self::N_TABLES)
            .map(|_| Table::from_reader(reader, version))
            .collect::<M8Result<Vec<Table>>>()?;

        println!("Instrument {}", reader.pos());
        let instruments = (0..Self::N_INSTRUMENTS)
            .map(|i| Instrument::from_reader(reader, i as u8, version))
            .collect::<M8Result<Vec<Instrument>>>()?;

        reader.read_bytes(3); // Skip
        let effects_settings = EffectsSettings::from_reader(reader, version)?;
        reader.set_pos(0x1A5FE);
        let midi_mappings = (0..Self::N_MIDI_MAPPINGS)
            .map(|_| MidiMapping::from_reader(reader))
            .collect::<M8Result<Vec<MidiMapping>>>()?;

        let scales: Vec<Scale> = if version.at_least(2, 5) {
            reader.set_pos(V4_OFFSETS.scale);
            (0..Self::N_SCALES)
                .map(|i| Scale::from_reader(reader, i as u8))
                .collect::<M8Result<Vec<Scale>>>()?
        } else {
            (0..Self::N_SCALES)
                .map(|i| -> Scale {
                    let mut s = Scale::default();
                    s.number = i as u8;
                    s
                })
                .collect()
        };

        let eqs = if version.at_least(4, 0) {
            let ofs = if version.at_least(4, 1) {
                &V4_1_OFFSETS
            } else {
                &V4_OFFSETS
            };

            reader.set_pos(ofs.eq);
            (0..ofs.instrument_eq_count)
                .map(|_i| Equ::from_reader(reader))
                .collect::<Vec<Equ>>()
        } else {
            vec!()
        };

        Ok(Self {
            version,
            directory,
            transpose,
            tempo,
            quantize,
            name,
            midi_settings,
            key,
            mixer_settings,
            grooves,
            song,
            phrases,
            chains,
            tables,
            instruments,
            scales,
            effects_settings,
            midi_mappings,
            eqs
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////
// MARK: SongSteps
////////////////////////////////////////////////////////////////////////////////////
#[derive(PartialEq, Clone)]
pub struct SongSteps {
    pub steps: [u8; SongSteps::TRACK_COUNT * SongSteps::ROW_COUNT],
}

impl SongSteps {
    pub const TRACK_COUNT : usize = 8;
    pub const ROW_COUNT : usize = 0x100;

    pub fn print_screen(&self) -> String {
        self.print_screen_from(0)
    }

    pub fn print_screen_from(&self, start: u8) -> String {
        (start..start + 16).fold("   1  2  3  4  5  6  7  8  \n".to_string(), |s, row| {
            s + &self.print_row(row) + "\n"
        })
    }

    pub fn print_row(&self, row: u8) -> String {
        let start = row as usize * 8;
        (start..start + 8).fold(format!("{row:02x} "), |s, b| -> String {
            let v = self.steps[b];
            let repr = if v == 255 {
                format!("-- ")
            } else {
                format!("{:02x} ", v)
            };
            s + &repr
        })
    }

    fn from_reader(reader: &mut Reader) -> M8Result<Self> {
        Ok(Self {
            steps: reader.read_bytes(2048).try_into().unwrap(),
        })
    }
}

impl fmt::Display for SongSteps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SONG\n\n{}", self.print_screen())
    }
}
impl fmt::Debug for SongSteps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

////////////////////////////////////////////////////////////////////////////////////
// MARK: Chains
////////////////////////////////////////////////////////////////////////////////////
#[derive(PartialEq, Clone, Default)]
pub struct Chain {
    pub steps: [ChainStep; 16],
}

impl Chain {
    pub const V4_SIZE : usize = ChainStep::V4_SIZE * 16;

    pub fn is_empty(&self) -> bool {
        self.steps.iter().all(|s| s.is_empty())
    }

    pub fn clear(&mut self) {
        let dflt = ChainStep::default();

        for s in &mut self.steps{
            *s = dflt;
        }
    }

    pub fn print_screen(&self) -> String {
        (0..16).fold("  PH TSP\n".to_string(), |s, row| {
            s + &self.steps[row].print(row as u8) + "\n"
        })
    }

    pub fn map(&self, mapping: &PhraseMapping) -> Self {
        let mut nc = self.clone();

        for i in 0 .. 16 {
            nc.steps[i] = nc.steps[i].map(mapping);
        }

        nc
    }

    pub fn write(&self, w: &mut Writer) {
        for cs in &self.steps {
            cs.write(w)
        }
    }

    pub fn from_reader(reader: &mut Reader) -> M8Result<Self> {
        Ok(Self { steps: arr![ChainStep::from_reader(reader)?; 16] })
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CHAIN\n\n{}", self.print_screen())
    }
}
impl fmt::Debug for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct ChainStep {
    pub phrase: u8,
    pub transpose: u8,
}

impl Default for ChainStep {
    fn default() -> Self {
        Self { phrase: 255, transpose: 0, }
    }
}

impl ChainStep {
    pub const V4_SIZE : usize = 2;

    pub fn is_empty(self) -> bool {
        self.phrase == 0xFF
    }

    pub fn print(&self, row: u8) -> String {
        if self.is_empty() {
            format!("{:x} -- 00", row)
        } else {
            format!("{:x} {:02x} {:02x}", row, self.phrase, self.transpose)
        }
    }

    pub fn map(&self, mapping: &PhraseMapping) -> Self {
        let phrase_ix = self.phrase as usize;
        let phrase =
            if phrase_ix >= Song::N_PHRASES { self.phrase }
            else { mapping.mapping[phrase_ix] };

        Self { phrase, transpose: self.transpose }
    }

    pub fn write(&self, w: &mut Writer) {
        w.write(self.phrase);
        w.write(self.transpose);
    }

    fn from_reader(reader: &mut Reader) -> M8Result<Self> {
        Ok(Self {
            phrase: reader.read(),
            transpose: reader.read(),
        })
    }
}


////////////////////////////////////////////////////////////////////////////////////
// MARK: Phrase
////////////////////////////////////////////////////////////////////////////////////
#[derive(PartialEq, Clone, Default)]
pub struct Phrase {
    pub steps: [Step; 16],
    version: Version,
}

impl Phrase {
    pub const V4_SIZE : usize = 16 * Step::V4_SIZE;

    pub fn is_empty(&self) -> bool {
        self.steps.iter().all(|s| s.is_empty())
    }

    pub fn clear(&mut self) {
        for s in &mut self.steps {
            s.clear();
        }
    }

    pub fn print_screen(&self, instruments: &[Instrument]) -> String {
        let mut cmd_pack = CommandPack::default();
        let fx_commands = FX::fx_command_names(self.version);
        let mut acc = String::from("  N   V  I  FX1   FX2   FX3  \n");


        for i in 0 .. 16 {
            let step = &self.steps[i];
            let instrument = step.instrument as usize;

            if instrument < Song::N_INSTRUMENTS {
                cmd_pack = instruments[instrument].instr_command_text(self.version);
            }

            acc += &step.print(i as u8, fx_commands, cmd_pack);
            acc += "\n";
        }

        acc
    }

    pub fn map_instruments(&self, instr_map: &InstrumentMapping) -> Self {
        let mut steps = self.steps.clone();
        for i in 0 .. steps.len() {
            steps[i] = steps[i].map_instr(&instr_map);
        }

        Self {
            steps,
            version: self.version
        }
    }

    pub fn write(&self, w: &mut Writer) {
        for s in &self.steps {
            s.write(w);
        }
    }

    pub fn from_reader(reader: &mut Reader,  version: Version) -> M8Result<Self> {
        Ok(Self {
            steps: arr![Step::from_reader(reader)?; 16],
            version,
        })
    }
}

pub struct PhraseView<'a> {
    phrase: &'a Phrase,
    instruments: &'a [Instrument]
}

impl<'a> fmt::Display for PhraseView<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PHRASE \n\n{}", self.phrase.print_screen(self.instruments))
    }
}

impl<'a> fmt::Debug for PhraseView<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct Step {
    pub note: Note,
    pub velocity: u8,
    pub instrument: u8,
    pub fx1: FX,
    pub fx2: FX,
    pub fx3: FX,
}


impl Step {
    pub const V4_SIZE : usize = 3 + 3 * FX::V4_SIZE;

    pub fn print(&self, row: u8, fx_cmds: FxCommands, cmd_pack: CommandPack ) -> String {
        let velocity = if self.velocity == 255 {
            format!("--")
        } else {
            format!("{:02x}", self.velocity)
        };
        let instrument = if self.instrument == 255 {
            format!("--")
        } else {
            format!("{:02x}", self.instrument)
        };

        format!(
            "{:x} {} {} {} {} {} {}",
            row,
            self.note,
            velocity,
            instrument,
            self.fx1.print(fx_cmds, cmd_pack),
            self.fx2.print(fx_cmds, cmd_pack),
            self.fx3.print(fx_cmds, cmd_pack)
        )
    }

    pub fn clear(&mut self) {
        self.note = Note::default();
        self.velocity = 0xFF;
        self.instrument = 0xFF;
        self.fx1 = FX::default();
        self.fx2 = FX::default();
        self.fx3 = FX::default();
    }

    pub fn is_empty(&self) -> bool {
        self.note.is_empty() &&
            self.velocity == 0xFF &&
            self.instrument == 0xFF &&
            self.fx1.is_empty() &&
            self.fx2.is_empty() &&
            self.fx3.is_empty()
    }

    pub fn map_instr(&self, mapping: &InstrumentMapping) -> Step {
        let instrument =
            if (self.instrument as usize) >= Song::N_INSTRUMENTS { self.instrument }
            else { mapping.mapping[self.instrument as usize] };

        Self {
            note: self.note,
            velocity: self.velocity,
            instrument,
            fx1: self.fx1,
            fx2: self.fx2,
            fx3: self.fx3

        }
    }

    pub fn write(&self, w: &mut Writer) {
        w.write(self.note.0);
        w.write(self.velocity);
        w.write(self.instrument);
        self.fx1.write(w);
        self.fx2.write(w);
        self.fx3.write(w);
    }

    fn from_reader(reader: &mut Reader) -> M8Result<Self> {
        Ok(Self {
            note: Note(reader.read()),
            velocity: reader.read(),
            instrument: reader.read(),
            fx1: FX::from_reader(reader)?,
            fx2: FX::from_reader(reader)?,
            fx3: FX::from_reader(reader)?,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////
// MARK: Note
////////////////////////////////////////////////////////////////////////////////////
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Note(pub u8);

impl Note {
    pub fn is_empty(self) -> bool {
        self.0 == 0xFF
    }
}

impl Default for Note {
    fn default() -> Self {
        Note(255)
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 255 {
            write!(f, "---")
        } else if self.0 >= 0x80 {
            write!(f, "OFF") // This isn't really true for < V3
        } else {
            let oct = (self.0 / 12) + 1;
            let n = match self.0 % 12 {
                0 => "C-",
                1 => "C#",
                2 => "D-",
                3 => "D#",
                4 => "E-",
                5 => "F-",
                6 => "F#",
                7 => "G-",
                8 => "G#",
                9 => "A-",
                10 => "A#",
                11 => "B-",
                _ => "??",
            };
            write!(f, "{}{:X}", n, oct)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////
// MARK: Table
////////////////////////////////////////////////////////////////////////////////////
#[derive(PartialEq, Clone)]
pub struct Table {
    pub steps: [TableStep; 16],
    version: Version,
}
impl Table {
    pub const V4_SIZE : usize = 16 * TableStep::V4_SIZE;

    pub fn is_empty(&self) -> bool {
        self.steps.iter().all(|s| s.is_empty())
    }

    pub fn clear(&mut self) {
        let dflt = TableStep::default();

        for s in &mut self.steps{
            *s = dflt.clone();
        }
    }

    pub fn print_screen(&self, cmd: CommandPack) -> String {
        let fx_cmd = FX::fx_command_names(self.version);
        let mut acc = String::from("  N  V  FX1   FX2   FX3  \n");

        for i in 0 .. 16 {
            let step = &self.steps[i];

            acc += &step.print(i as u8, fx_cmd, cmd);
            acc += "\n";
        }

        acc
    }

    pub fn write(&self, w: &mut Writer) {
        for ts in &self.steps {
            ts.write(w);
        }
    }

    pub fn from_reader(reader: &mut Reader, version: Version) -> M8Result<Self> {
        Ok(Self { steps: arr![TableStep::from_reader(reader)?; 16], version })
    }
}

pub struct TableView<'a> {
    table: &'a Table,
    instrument: CommandPack
}

impl<'a> fmt::Display for TableView<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TABLE\n\n{}", self.table.print_screen(self.instrument))
    }
}

impl<'a> fmt::Debug for TableView<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct TableStep {
    pub transpose: u8,
    pub velocity: u8,
    pub fx1: FX,
    pub fx2: FX,
    pub fx3: FX,
}

impl Default for TableStep {
    fn default() -> Self {
        Self {
            transpose: 0,
            velocity: 0xFF,
            fx1: Default::default(),
            fx2: Default::default(),
            fx3: Default::default()
        }
    }
}

impl TableStep {
    pub const V4_SIZE : usize = 2 + 3 * FX::V4_SIZE;

    pub fn is_empty(&self) -> bool {
        self.transpose == 0 &&
            self.velocity == 0xFF &&
            self.fx1.is_empty() &&
            self.fx2.is_empty() &&
            self.fx3.is_empty()
    }

    pub fn print(&self, row: u8, fx_cmd: FxCommands, cmds: CommandPack) -> String {
        let transpose = if self.transpose == 255 {
            format!("--")
        } else {
            format!("{:02x}", self.transpose)
        };
        let velocity = if self.velocity == 255 {
            format!("--")
        } else {
            format!("{:02x}", self.velocity)
        };
        format!(
            "{:x} {} {} {} {} {}",
            row,
            transpose,
            velocity,
            self.fx1.print(fx_cmd, cmds),
            self.fx2.print(fx_cmd, cmds),
            self.fx3.print(fx_cmd, cmds)
        )
    }

    pub fn write(&self, w: &mut Writer) {
        w.write(self.transpose);
        w.write(self.velocity);
        self.fx1.write(w);
        self.fx2.write(w);
        self.fx3.write(w);
    }

    fn from_reader(reader: &mut Reader) -> M8Result<Self> {
        Ok(Self {
            transpose: reader.read(),
            velocity: reader.read(),
            fx1: FX::from_reader(reader)?,
            fx2: FX::from_reader(reader)?,
            fx3: FX::from_reader(reader)?,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////
// MARK: Groove
////////////////////////////////////////////////////////////////////////////////////
#[derive(PartialEq, Clone)]
pub struct Groove {
    pub number: u8,
    pub steps: [u8; 16],
}
impl Groove {
    fn from_reader(reader: &mut Reader, number: u8) -> M8Result<Self> {
        Ok(Self {
            number,
            steps: reader.read_bytes(16).try_into().unwrap(),
        })
    }

    pub fn write(&self, w: &mut Writer) {
        w.write_bytes(&self.steps);
    }

    pub fn active_steps(&self) -> &[u8] {
        let end = (&self.steps).iter().position(|&x| x == 255).unwrap_or(15);
        &self.steps[0..end]
    }
}

impl fmt::Display for Groove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Groove {}:{:?}", self.number, self.active_steps())
    }
}
impl fmt::Debug for Groove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

////////////////////////////////////////////////////////////////////////////////////
// MARK: Tests
////////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use crate::songs::*;
    use std::fs::File;

    fn test_file() -> Song {
        let mut f = File::open("./examples/songs/TEST-FILE.m8s").expect("Could not open TEST-FILE");
        Song::read(&mut f).expect("Could not parse TEST-FILE")
    }

    #[test]
    fn test_instrument_reading() {
        let test_file = test_file();
        // dbg!(&test_file.instruments[0..8]);
        assert!(match &test_file.instruments[0] {
            Instrument::None => true,
            _ => false,
        });
        assert!(
            match &test_file.instruments[1] {
                Instrument::WavSynth(s) => {
                    assert_eq!(s.transpose, true);
                    assert_eq!(s.size, 0x20);
                    assert_eq!(s.synth_params.mixer_reverb, 0xD0);
                    assert!(match s.synth_params.mods[0] {
                        Mod::AHDEnv(_) => true,
                        _ => false,
                    });
                    assert!(match s.synth_params.mods[1] {
                        Mod::ADSREnv(_) => true,
                        _ => false,
                    });
                    assert!(match s.synth_params.mods[2] {
                        Mod::DrumEnv(_) => true,
                        _ => false,
                    });
                    assert!(match s.synth_params.mods[3] {
                        Mod::LFO(_) => true,
                        _ => false,
                    });

                    true
                }
                _ => false,
            },
            "Should be a WavSynth"
        );
        assert!(match &test_file.instruments[2] {
            Instrument::MacroSynth(s) => {
                assert_eq!(s.transpose, false);
                assert!(match s.synth_params.mods[0] {
                    Mod::TrigEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[1] {
                    Mod::TrackingEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[2] {
                    Mod::LFO(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[3] {
                    Mod::LFO(_) => true,
                    _ => false,
                });

                true
            }
            _ => false,
        });
        assert!(match &test_file.instruments[3] {
            Instrument::Sampler(s) => {
                assert!(match s.synth_params.mods[0] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[1] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[2] {
                    Mod::LFO(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[3] {
                    Mod::LFO(_) => true,
                    _ => false,
                });

                assert_eq!(&s.name, "SAMP");
                assert_eq!(
                    &s.sample_path,
                    "/Samples/Drums/Hits/TR505/bass drum 505.wav"
                );

                true
            }
            _ => false,
        });
        assert!(match &test_file.instruments[4] {
            Instrument::FMSynth(s) => {
                assert!(match s.synth_params.mods[0] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[1] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[2] {
                    Mod::LFO(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[3] {
                    Mod::LFO(_) => true,
                    _ => false,
                });

                true
            }
            _ => false,
        });
        assert!(match &test_file.instruments[5] {
            Instrument::HyperSynth(s) => {
                assert!(match s.synth_params.mods[0] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[1] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[2] {
                    Mod::LFO(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[3] {
                    Mod::LFO(_) => true,
                    _ => false,
                });
                assert_eq!(s.scale, 0xFF);
                assert_eq!(s.default_chord[0], 0x01);
                assert_eq!(s.default_chord[6], 0x3C);

                true
            }
            _ => false,
        });
        assert!(match &test_file.instruments[6] {
            Instrument::MIDIOut(s) => {
                assert!(match s.mods.mods[0] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.mods.mods[1] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.mods.mods[2] {
                    Mod::LFO(_) => true,
                    _ => false,
                });
                assert!(match s.mods.mods[3] {
                    Mod::LFO(_) => true,
                    _ => false,
                });
                true
            }
            _ => false,
        });
        assert!(match &test_file.instruments[7] {
            Instrument::External(s) => {
                assert!(match s.synth_params.mods[0] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[1] {
                    Mod::AHDEnv(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[2] {
                    Mod::LFO(_) => true,
                    _ => false,
                });
                assert!(match s.synth_params.mods[3] {
                    Mod::LFO(_) => true,
                    _ => false,
                });

                assert_eq!(s.cca.number, 1);
                assert_eq!(s.ccb.number, 2);
                assert_eq!(s.ccd.number, 4);
                true
            }
            _ => false,
        });
    }

    #[test]
    fn test_mixer_reading() {
        let test_file = test_file();
        // dbg!(&test_file.mixer_settings);
        assert_eq!(test_file.mixer_settings.track_volume[0], 0xE0);
        assert_eq!(test_file.mixer_settings.track_volume[7], 0xE0);
        assert_eq!(test_file.mixer_settings.dj_filter, 0x80);
        assert_eq!(test_file.mixer_settings.dj_filter_type, 0x02);
    }

    #[test]
    fn test_song_reading() {
        let test_file = test_file();
        // dbg!(&test_file);
        assert_eq!(test_file.name, "TEST-FILE");
        assert_eq!(test_file.tempo, 120.0);
        assert_eq!(test_file.transpose, 0x0C);
        assert_eq!(test_file.quantize, 0x02);
    }
}