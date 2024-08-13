use vizia::prelude::*;

use strum::EnumString;
use strum::VariantNames;

#[derive(Lens)]
pub struct SettingsData {
    pub selected_page: SettingsPage,

    pub audio_driver: Vec<String>,
    pub input_device: Vec<String>,
    pub output_device: Vec<String>,

    pub selected_audio_driver: usize,
    pub selected_input_device: usize,
    pub selected_output_device: usize,
}

impl SettingsData {
    pub fn dummy() -> Self {
        Self {
            selected_page: SettingsPage::General,

            audio_driver: vec![String::from("WASAPI"), String::from("ASIO")],
            input_device: vec![String::from("Microphone 1"), String::from("USB Microphone")],
            output_device: vec![String::from("Speakers"), String::from("USB Headphones")],

            selected_audio_driver: 0,
            selected_input_device: 0,
            selected_output_device: 0,
        }
    }
}

pub enum SettingsEvent {
    ShowGeneral,
    ShowUserInterface,
    ShowAudio,
}

#[derive(Debug, EnumString, VariantNames, Data, Clone, Copy, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case")]
pub enum SettingsPage {
    General,
    UserInterface,
    Audio,
}

impl Model for SettingsData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|settings_event, _| match settings_event {
            SettingsEvent::ShowGeneral => self.selected_page = SettingsPage::General,
            SettingsEvent::ShowUserInterface => self.selected_page = SettingsPage::UserInterface,
            SettingsEvent::ShowAudio => self.selected_page = SettingsPage::Audio,
        })
    }
}
