use crate::reader::*;
use crate::version::*;
use crate::writer::Writer;
use common::SynthParams;
use external_inst::ExternalInst;
use fmsynth::FMSynth;
use hypersynth::HyperSynth;
use macrosynth::MacroSynth;
use midi::MIDIOut;
use modulator::Mod;
use sampler::Sampler;
use wavsynth::WavSynth;

pub mod common;
pub mod modulator;
pub mod external_inst;
pub mod midi;
pub mod macrosynth;
pub mod fmsynth;
pub mod hypersynth;
pub mod sampler;
pub mod wavsynth;

#[derive(PartialEq, Debug, Clone, Default)]
pub enum Instrument {
    WavSynth(WavSynth),
    MacroSynth(MacroSynth),
    Sampler(Sampler),
    MIDIOut(MIDIOut),
    FMSynth(FMSynth),
    HyperSynth(HyperSynth),
    External(ExternalInst),
    #[default]
    None
}


pub mod params {
    pub const NAME : &'static str = "NAME";
    pub const TRANSPOSE : &'static str = "TRANSPOSE";
    pub const TBLTIC : &'static str = "TBL. TIC";
    pub const EQ : &'static str = "EQ";
    pub const SCALE : &'static str = "SCALE";

    pub const CCA : &'static str = "CCA";
    pub const CCB : &'static str = "CCB";
    pub const CCC : &'static str = "CCC";
    pub const CCD : &'static str = "CCD";

    pub const DEST : &'static str = "DEST";
    pub const AMOUNT : &'static str = "AMT";
    pub const ATTACK : &'static str = "ATK";
    pub const DECAY : &'static str = "DEC";
    pub const HOLD : &'static str = "HOLD";
    pub const SUSTAIN : &'static str = "SUS";
    pub const RELEASE : &'static str = "REL";
    pub const PEAK : &'static str = "PEAK";
    pub const BODY : &'static str = "BODY";
    pub const FREQ : &'static str = "FREQ";
    pub const TRIGGER : &'static str = "TRIG";
    pub const LFOSHAPE : &'static str = "OSC";
    pub const SOURCE : &'static str = "SRC";
}

pub mod dests {
    pub const OFF : &'static str = "OFF";
    pub const VOLUME : &'static str = "VOLUME";
    pub const PITCH : &'static str = "PITCH";
    pub const CUTOFF : &'static str = "CUTOFF";
    pub const RES : &'static str = "RES";
    pub const AMP : &'static str = "AMP";
    pub const PAN : &'static str = "PAN";
    pub const DEGRADE : &'static str = "DEGRADE";
    pub const MOD_AMT : &'static str = "MOD AMT";
    pub const MOD_RATE : &'static str = "MOD RATE";
    pub const MOD_BOTH : &'static str = "MOD BOTH";
    pub const MOD_BINV : &'static str = "MOD BINV";
}
/// For every instrument, retrieve the command names
#[derive(Clone, Copy)]
pub struct CommandPack {
    /// Instruments command
    pub instr: &'static [&'static str],

    /// For all the modulators, their respective
    /// command names
    pub mod_commands: [&'static[&'static str]; SynthParams::MODULATOR_COUNT]
}

impl Default for CommandPack {
    fn default() -> Self {
        Self { instr: Default::default(), mod_commands: Default::default() }
    }
}

impl CommandPack {
    pub const BASE_INSTRUMENT_COMMAND_COUNT : usize = 18;
    pub const INSTRUMENT_COMMAND_OFFSET : usize = 0x80;
    pub const BASE_INSTRUMENT_COMMAND_END : usize =
        CommandPack::INSTRUMENT_COMMAND_OFFSET +
            Mod::COMMAND_PER_MOD * SynthParams::MODULATOR_COUNT;

    pub fn accepts(self, cmd: u8) -> bool {
        let cmd = cmd as usize;
        CommandPack::INSTRUMENT_COMMAND_OFFSET <= cmd &&
            cmd <= (CommandPack::BASE_INSTRUMENT_COMMAND_END + self.instr.len())
    }

    pub fn try_render(self, cmd: u8) -> Option<&'static str> {
        if self.instr.len() == 0 { return None }
        if (cmd as usize) < CommandPack::INSTRUMENT_COMMAND_OFFSET { return None }

        let cmd = cmd as usize - CommandPack::INSTRUMENT_COMMAND_OFFSET;

        if  cmd < CommandPack::BASE_INSTRUMENT_COMMAND_COUNT {
            if cmd < self.instr.len() {
                return Some(self.instr[cmd])
            } else {
                return None
            }
        }

        let mod_cmd = cmd - CommandPack::BASE_INSTRUMENT_COMMAND_COUNT;
        let mod_ix = mod_cmd / Mod::COMMAND_PER_MOD;

        if mod_ix < self.mod_commands.len() {
            let ix = mod_cmd - Mod::COMMAND_PER_MOD * mod_ix;
            return Some(self.mod_commands[mod_ix][ix])
        }

        let extra_cmd = cmd - (Mod::COMMAND_PER_MOD * SynthParams::MODULATOR_COUNT);
        if extra_cmd < self.instr.len() {
            return Some(self.instr[extra_cmd])
        }

        None
    }
}

pub const INSTRUMENT_MEMORY_SIZE : usize = 215;
// const MOD_OFFSET : usize = 0x3B;

impl Instrument {
    pub const V4_SIZE : usize = INSTRUMENT_MEMORY_SIZE;

    pub fn is_empty(&self) -> bool {
        match self {
            Instrument::None => true,
            _ => false
        }
    }

    pub fn instr_command_text(&self, ver: Version) -> CommandPack {
        let (commands, mods) =
            match self {
                Instrument::WavSynth(ws)     => (ws.command_name(ver), &ws.synth_params.mods),
                Instrument::MacroSynth(ms) => (ms.command_name(ver), &ms.synth_params.mods),
                Instrument::Sampler(s)        => (s.command_name(ver), &s.synth_params.mods),
                Instrument::MIDIOut(mo)       => (mo.command_name(ver), &mo.mods.mods),
                Instrument::FMSynth(fs)       => (fs.command_name(ver), &fs.synth_params.mods),
                Instrument::HyperSynth(hs) => (hs.command_name(ver), &hs.synth_params.mods),
                Instrument::External(ex) => (ex.command_name(ver), &ex.synth_params.mods),
                Instrument::None => return CommandPack::default()
            };

        CommandPack {
            instr: commands,
            mod_commands: [
                mods[0].command_name(ver, 0),
                mods[1].command_name(ver, 1),
                mods[2].command_name(ver, 2),
                mods[3].command_name(ver, 3)
            ]
        }
    }

    pub fn write(&self, w: &mut Writer) {
        match self {
            Instrument::WavSynth(ws)     => { w.write(0); ws.write(w); }
            Instrument::MacroSynth(ms) => { w.write(1); ms.write(w); }
            Instrument::Sampler(s)        => { w.write(2); s.write(w); }
            Instrument::MIDIOut(mo)       => { w.write(3); mo.write(w); }
            Instrument::FMSynth(fs)       => { w.write(4); fs.write(w); }
            Instrument::HyperSynth(hs) => { w.write(5); hs.write(w); }
            Instrument::External(ex) => { w.write(6); ex.write(w); }
            Instrument::None => w.write(0xFF),
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Instrument::WavSynth(ws)     => Some(&ws.name),
            Instrument::MacroSynth(ms) => Some(&ms.name),
            Instrument::Sampler(s)        => Some(&s.name),
            Instrument::MIDIOut(_)                  => None,
            Instrument::FMSynth(fs)       => Some(&fs.name),
            Instrument::HyperSynth(hs) => Some(&hs.name),
            Instrument::External(ex) => Some(&ex.name),
            Instrument::None => None,
        }
    }

    pub fn set_name(&mut self, name: String) {
        match self {
            Instrument::WavSynth(ws)     => ws.name = name,
            Instrument::MacroSynth(ms) => ms.name = name,
            Instrument::Sampler(s)        => s.name = name,
            Instrument::MIDIOut(mo)       => mo.name = name,
            Instrument::FMSynth(fs)       => fs.name = name,
            Instrument::HyperSynth(hs) => hs.name = name,
            Instrument::External(ex) => ex.name = name,
            Instrument::None => {}
        }
    }

    pub fn equ(&self) -> Option<u8> {
        match self {
            Instrument::WavSynth(ws)     => Some(ws.transp_eq.eq),
            Instrument::MacroSynth(ms) => Some(ms.transp_eq.eq),
            Instrument::Sampler(s)        => Some(s.transp_eq.eq),
            Instrument::MIDIOut(_)                  => None,
            Instrument::FMSynth(fs)       => Some(fs.transp_eq.eq),
            Instrument::HyperSynth(hs) => Some(hs.transp_eq.eq),
            Instrument::External(ex) => Some(ex.transp_eq.eq),
            Instrument::None => None,
        }
    }

    pub fn set_eq(&mut self, eq_ix: u8) {
        match self {
            Instrument::WavSynth(ws)     => ws.transp_eq.set_eq(eq_ix),
            Instrument::MacroSynth(ms) => ms.transp_eq.set_eq(eq_ix),
            Instrument::Sampler(s)        => s.transp_eq.set_eq(eq_ix),
            Instrument::MIDIOut(_)                      => {},
            Instrument::FMSynth(fs)       => fs.transp_eq.set_eq(eq_ix),
            Instrument::HyperSynth(hs) => hs.transp_eq.set_eq(eq_ix),
            Instrument::External(ex) => ex.transp_eq.set_eq(eq_ix),
            Instrument::None => {},
        }
    }

    pub fn read(reader: &mut impl std::io::Read) -> M8Result<Self> {
        let mut buf: Vec<u8> = vec![];
        reader.read_to_end(&mut buf).unwrap();
        let len = buf.len();
        let mut reader = Reader::new(buf);

        if len < INSTRUMENT_MEMORY_SIZE + Version::SIZE {
            return Err(ParseError(
                "File is not long enough to be a M8 Instrument".to_string(),
            ));
        }

        let version = Version::from_reader(&mut reader)?;
        Self::from_reader(&mut reader, 0, version)
    }

    pub fn from_reader(reader: &mut Reader, number: u8, version: Version) -> M8Result<Self> {
        let start_pos = reader.pos();
        let kind = reader.read();

        let instr = match kind {
            0x00 => Self::WavSynth(WavSynth::from_reader(reader, number, version)?),
            0x01 => Self::MacroSynth(MacroSynth::from_reader(reader, number, version)?),
            0x02 => Self::Sampler(Sampler::from_reader(reader, start_pos, number, version)?),
            0x03 => Self::MIDIOut(MIDIOut::from_reader(reader, number, version)?),
            0x04 => Self::FMSynth(FMSynth::from_reader(reader, number, version)?),
            0x05 if version.at_least(3, 0) => Self::HyperSynth(HyperSynth::from_reader(reader, number)?),
            0x06 if version.at_least(3, 0) => Self::External(ExternalInst::from_reader(reader, number)?),
            0xFF => Self::None,
            _ => return Err(ParseError(format!("Instrument type {} not supported", kind))),
        };

        reader.set_pos(start_pos + INSTRUMENT_MEMORY_SIZE);

        Ok(instr)
    }
}
