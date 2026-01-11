use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum LightState {
    On,
    Off,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LightPayload {
    pub state: LightState,
    pub brightness: u8, // 0 => 254
    pub color_temp: u8, // 153 => 454
}

impl LightPayload {
    pub fn on() -> Self {
        Self {
            state: LightState::On,
            brightness: 254,
            color_temp: 200,
        }
    }

    pub fn off() -> Self {
        Self {
            state: LightState::Off,
            brightness: 0,
            color_temp: 200,
        }
    }

    pub fn with_brightness(brightness: u8) -> Self {
        Self {
            state: LightState::On,
            brightness,
            color_temp: 200,
        }
    }
}

pub struct LightsController {}
