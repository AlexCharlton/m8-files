use crate::instruments::common::*;
use crate::reader::*;
use crate::version::*;
use crate::writer::Writer;

use arr_macro::arr;

use super::dests;
use super::params;
use super::CommandPack;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct ControlChange {
    /// CC number (target)
    pub number: u8,

    /// Value to be sent via MIDI CC message
    pub value: u8,
}

impl ControlChange {
    pub fn write(self, writer: &mut Writer) {
        writer.write(self.number);
        writer.write(self.value);
    }

    pub fn from_reader(reader: &mut Reader) -> M8Result<Self> {
        Ok(Self {
            number: reader.read(),
            value: reader.read(),
        })
    }
}

#[rustfmt::skip] // Keep constats with important order vertical for maintenance
const MIDI_OUT_COMMAND_NAMES : [&'static str; CommandPack::BASE_INSTRUMENT_COMMAND_COUNT - 2] = [
    "VOL",
    "PIT",
    "MPG",
    "MPB",
    "ADD",
    "CHD",
    "CCA",
    "CCB",
    "CCC",
    "CCD",
    "CCE",
    "CCF",
    "CCG",
    "CCH",
    "CCI",
    "CCJ",
];

#[rustfmt::skip] // Keep constats with important order vertical for maintenance
const DESTINATIONS : [&'static str; 15] = [
    dests::OFF,
    params::CCA,
    params::CCB,
    params::CCC,
    params::CCD,
    "CCE",
    "CCF",
    "CCG",
    "CCH",
    "CCI",
    "CCJ",
    dests::MOD_AMT,
    dests::MOD_RATE,
    dests::MOD_BOTH,
    dests::MOD_BINV,
];

#[rustfmt::skip] // Keep constats with important order vertical for maintenance
const PORTS : [&'static str; 4] = [
    "MIDI + USB",
    "MIDI",
    "USB",
    "INTERNAL"
];

#[derive(PartialEq, Debug, Clone)]
pub struct MIDIOut {
    pub number: u8,
    pub name: String,
    pub transpose: bool,
    pub table_tick: u8,

    pub port: u8,
    pub channel: u8,
    pub bank_select: u8,
    pub program_change: u8,
    pub custom_cc: [ControlChange; 10],

    pub mods: SynthParams,
}

impl MIDIOut {
    const MOD_OFFSET: usize = 21;

    pub fn command_name(&self, _ver: Version) -> &'static [&'static str] {
        &MIDI_OUT_COMMAND_NAMES
    }

    pub fn destination_names(&self, _ver: Version) -> &'static [&'static str] {
        &DESTINATIONS
    }

    pub fn human_readable_port(&self) -> &'static str {
        PORTS[self.port as usize]
    }

    pub fn write(&self, _ver: Version, w: &mut Writer) {
        w.write_string(&self.name, 12);
        w.write(if self.transpose { 1 } else { 0 });
        w.write(self.table_tick);
        w.write(self.port);
        w.write(self.channel);
        w.write(self.bank_select);
        w.write(self.program_change);

        w.skip(3);

        for cc in self.custom_cc {
            cc.write(w);
        }

        self.mods.write_modes(w, MIDIOut::MOD_OFFSET)
    }

    pub fn from_reader(
        _ver: Version,
        reader: &mut Reader,
        number: u8,
        version: Version,
    ) -> M8Result<Self> {
        let name = reader.read_string(12);
        let transpose = reader.read_bool();
        let table_tick = reader.read();

        let port = reader.read();
        let channel = reader.read();
        let bank_select = reader.read();
        let program_change = reader.read();
        reader.read_bytes(3); // discard
        let custom_cc = arr![ControlChange::from_reader(reader)?; 10];
        let mods = if version.at_least(3, 0) {
            SynthParams::mod_only3(reader, MIDIOut::MOD_OFFSET)?
        } else {
            SynthParams::mod_only2(reader)?
        };

        Ok(MIDIOut {
            number,
            name,
            transpose,
            table_tick,

            port,
            channel,
            bank_select,
            program_change,
            custom_cc,
            mods,
        })
    }
}
