use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LightState {
    On,
    Off,
}

#[derive(Serialize)]
struct LightPayload {
    state: LightState,
    brightness: u8, // 0 => 254
    color_temp: u8, // 153 => 454
}

pub struct LightsController {}
