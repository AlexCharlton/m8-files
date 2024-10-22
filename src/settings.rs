use crate::{reader::*, Version};

#[derive(PartialEq, Debug, Clone)]
pub struct MidiSettings {
    pub receive_sync: bool,
    pub receive_transport: u8,
    pub send_sync: bool,
    pub send_transport: u8,
    pub record_note_channel: u8,
    pub record_note_velocity: bool,
    pub record_note_delay_kill_commands: u8,
    pub control_map_channel: u8,
    pub song_row_cue_channel: u8,
    pub track_input_channel: [u8; 8],
    pub track_input_intrument: [u8; 8],
    pub track_input_program_change: bool,
    pub track_input_mode: u8,
}

impl TryFrom<&mut Reader> for MidiSettings {
    type Error = ParseError;

    fn try_from(reader: &mut Reader) -> M8Result<Self> {
        Ok(Self {
            receive_sync: reader.read_bool(),
            receive_transport: reader.read(),
            send_sync: reader.read_bool(),
            send_transport: reader.read(),
            record_note_channel: reader.read(),
            record_note_velocity: reader.read_bool(),
            record_note_delay_kill_commands: reader.read(),
            control_map_channel: reader.read(),
            song_row_cue_channel: reader.read(),
            track_input_channel: reader.read_bytes(8).try_into().unwrap(),
            track_input_intrument: reader.read_bytes(8).try_into().unwrap(),
            track_input_program_change: reader.read_bool(),
            track_input_mode: reader.read(),
        })

    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct MixerSettings {
    pub master_volume: u8,
    pub master_limit: u8,
    pub track_volume: [u8; 8],
    pub chorus_volume: u8,
    pub delay_volume: u8,
    pub reverb_volume: u8,
    pub analog_input: AnalogInputSettings,
    pub usb_input: InputMixerSettings,
    pub dj_filter: u8,
    pub dj_peak: u8,
    pub dj_filter_type: u8,
}

impl MixerSettings {
    pub(crate) fn from_reader(reader: &mut Reader) -> M8Result<Self> {
        let master_volume = reader.read();
        let master_limit = reader.read();
        let track_volume: [u8; 8] = reader.read_bytes(8).try_into().unwrap();
        let chorus_volume = reader.read();
        let delay_volume = reader.read();
        let reverb_volume = reader.read();
        let analog_input_volume = (reader.read(), reader.read());
        let usb_input_volume = reader.read();
        let analog_input_chorus = (reader.read(), reader.read());
        let analog_input_delay = (reader.read(), reader.read());
        let analog_input_reverb = (reader.read(), reader.read());
        let usb_input_chorus = reader.read();
        let usb_input_delay = reader.read();
        let usb_input_reverb = reader.read();

        let analog_input_l = InputMixerSettings {
            volume: analog_input_volume.0,
            chorus: analog_input_chorus.0,
            delay: analog_input_delay.0,
            reverb: analog_input_reverb.0,
        };

        let analog_input = if analog_input_volume.1 == 255 {
            AnalogInputSettings::Stereo(analog_input_l)
        } else {
            let analog_input_r = InputMixerSettings {
                volume: analog_input_volume.0,
                chorus: analog_input_chorus.0,
                delay: analog_input_delay.0,
                reverb: analog_input_reverb.0,
            };
            AnalogInputSettings::DualMono((analog_input_l, analog_input_r))
        };
        let usb_input = InputMixerSettings {
            volume: usb_input_volume,
            chorus: usb_input_chorus,
            delay: usb_input_delay,
            reverb: usb_input_reverb,
        };

        let dj_filter = reader.read();
        let dj_peak = reader.read();
        let dj_filter_type = reader.read();

        reader.read_bytes(4); // discard
        Ok(Self {
            master_volume,
            master_limit,
            track_volume,
            chorus_volume,
            delay_volume,
            reverb_volume,
            analog_input,
            usb_input,
            dj_filter,
            dj_peak,
            dj_filter_type,
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct InputMixerSettings {
    pub volume: u8,
    pub chorus: u8,
    pub delay: u8,
    pub reverb: u8,
}

#[derive(PartialEq, Debug, Clone)]
pub enum AnalogInputSettings {
    Stereo(InputMixerSettings),
    DualMono((InputMixerSettings, InputMixerSettings)),
}

#[derive(PartialEq, Debug, Clone)]
pub struct EffectsSettings {
    pub chorus_mod_depth: u8,
    pub chorus_mod_freq: u8,
    pub chorus_reverb_send: u8,

    pub delay_hp: u8,
    pub delay_lp: u8,
    pub delay_time_l: u8,
    pub delay_time_r: u8,
    pub delay_feedback: u8,
    pub delay_width: u8,
    pub delay_reverb_send: u8,

    pub reverb_hp: u8,
    pub reverb_lp: u8,
    pub reverb_size: u8,
    pub reverb_damping: u8,
    pub reverb_mod_depth: u8,
    pub reverb_mod_freq: u8,
    pub reverb_width: u8,
}
impl EffectsSettings {
    pub(crate) fn from_reader(reader: &mut Reader, version: Version) -> M8Result<Self> {
        let chorus_mod_depth = reader.read();
        let chorus_mod_freq = reader.read();
        let chorus_reverb_send = reader.read();
        reader.read_bytes(3); //unused

        // THIS likely changed :()
        let (delay_hp, delay_lp) =
            if version.at_least(4, 0) {
                (0, 0)
            } else {
                (reader.read(), reader.read())
            };
        
        let delay_time_l = reader.read();
        let delay_time_r = reader.read();
        let delay_feedback = reader.read();
        let delay_width = reader.read();
        let delay_reverb_send = reader.read();
        reader.read_bytes(1); //unused

        // This likely changed :()
        let (reverb_hp, reverb_lp) =
            if version.at_least(4, 0) {
                (0, 0)
            } else {
                (reader.read(), reader.read())
            };

        let reverb_size = reader.read();
        let reverb_damping = reader.read();
        let reverb_mod_depth = reader.read();
        let reverb_mod_freq = reader.read();
        let reverb_width = reader.read();

        Ok(Self {
            chorus_mod_depth,
            chorus_mod_freq,
            chorus_reverb_send,

            delay_hp,
            delay_lp,
            delay_time_l,
            delay_time_r,
            delay_feedback,
            delay_width,
            delay_reverb_send,

            reverb_hp,
            reverb_lp,
            reverb_size,
            reverb_damping,
            reverb_mod_depth,
            reverb_mod_freq,
            reverb_width,
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct MidiMapping {
    pub channel: u8,
    pub control_number: u8,
    pub value: u8,
    pub typ: u8,
    pub param_index: u8,
    pub min_value: u8,
    pub max_value: u8,
}

impl MidiMapping {
    pub(crate) fn from_reader(reader: &mut Reader) -> M8Result<Self> {
        Ok(Self {
            channel: reader.read(),
            control_number: reader.read(),
            value: reader.read(),
            typ: reader.read(),
            param_index: reader.read(),
            min_value: reader.read(),
            max_value: reader.read(),
        })
    }

    pub fn empty(&self) -> bool {
        self.channel == 0
    }
}
