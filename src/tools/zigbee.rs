use std::time::Duration;

use rumqttc::{AsyncClient, ClientError, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_repr::Serialize_repr;
use strum::IntoEnumIterator;

#[derive(Debug, Clone, strum::EnumIter)]
pub enum DeviceName {
    HueKitchen1,
    HueKitchen2,
    HueKitchen3,
    HueBedroom1,
    HueBedroom2,
    HueBedroom3,
    HueLivingroom1,
    HueLivingroom2,
    HueLivingroom3,
    HueLivingroom4,
    HueLivingroom5,
    HueLivingroom6,
    IkeaMushroom,
    IkeaDonut,
    LightBulb,
    SofaLight,
    BallLight,
}

impl DeviceName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HueKitchen1 => "hue_kitchen_1",
            Self::HueKitchen2 => "hue_kitchen_2",
            Self::HueKitchen3 => "hue_kitchen_3",
            Self::HueBedroom1 => "hue_bedroom_1",
            Self::HueBedroom2 => "hue_bedroom_2",
            Self::HueBedroom3 => "hue_bedroom_3",
            Self::HueLivingroom1 => "hue_livingroom_1",
            Self::HueLivingroom2 => "hue_livingroom_2",
            Self::HueLivingroom3 => "hue_livingroom_3",
            Self::HueLivingroom4 => "hue_livingroom_4",
            Self::HueLivingroom5 => "hue_livingroom_5",
            Self::HueLivingroom6 => "hue_livingroom_6",
            Self::IkeaMushroom => "ikea_mushroom",
            Self::IkeaDonut => "ikea_donut",
            Self::LightBulb => "light_bulb",
            Self::SofaLight => "sofa_light",
            Self::BallLight => "ball_light",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum LightState {
    On,
    Off,
}

/// Range: 250-454
#[repr(u16)]
#[derive(Debug, Serialize_repr)]
pub enum ColorTemp {
    Blue = 250,
    White = 352,
    Warm = 454,
}

/// Range: 0-254
#[repr(u8)]
#[derive(Debug, Serialize_repr)]
pub enum Brightness {
    Min = 8,
    Low = 64,
    Medium = 127,
    High = 191,
    Max = 254,
}

#[derive(Serialize, Debug)]
pub struct LightPayload {
    pub state: LightState,
    pub brightness: Brightness,
    pub color_temp: ColorTemp,
}

pub struct ZigbeeController {
    client: AsyncClient,
}

impl ZigbeeController {
    pub async fn new(broker_host: &str, broker_port: u16) -> Self {
        let mut mqttoptions = MqttOptions::new("son-of-anton", broker_host, broker_port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

        // Spawn the event loop
        tokio::spawn(async move {
            loop {
                if let Err(e) = eventloop.poll().await {
                    eprintln!("MQTT Error: {:?}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        });

        Self { client }
    }

    async fn turn_off(&self, device_name: &DeviceName) -> Result<(), ClientError> {
        let topic = format!("zigbee2mqtt/{}/set", device_name.as_str());
        let payload = json!({ "state": "OFF" }).to_string();

        self.client
            .publish(&topic, QoS::AtLeastOnce, false, payload)
            .await
    }

    async fn send_payload(
        &self,
        device_name: &DeviceName,
        payload: &LightPayload,
    ) -> Result<(), ClientError> {
        let topic = format!("zigbee2mqtt/{}/set", device_name.as_str());
        let payload = serde_json::to_string(payload).unwrap(); // TODO FIX

        self.client
            .publish(&topic, QoS::AtLeastOnce, false, payload)
            .await
    }

    pub async fn turn_all_off(&self) -> Result<(), ClientError> {
        for device_name in DeviceName::iter() {
            tokio::time::sleep(Duration::from_millis(100)).await;
            self.turn_off(&device_name).await?;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    pub async fn turn_all_on(
        &self,
        brightness: Brightness,
        color_temp: ColorTemp,
    ) -> Result<(), ClientError> {
        let payload = LightPayload {
            state: LightState::On,
            brightness,
            color_temp,
        };

        for device_name in DeviceName::iter() {
            tokio::time::sleep(Duration::from_millis(100)).await;
            self.send_payload(&device_name, &payload).await?;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    pub async fn night_mode(&self) -> Result<(), ClientError> {
        todo!();
    }

    pub async fn movie_mode(&self) -> Result<(), ClientError> {
        todo!();
    }
}

#[cfg(test)]
pub mod tests {
    use crate::tools::zigbee::{Brightness, ColorTemp, ZigbeeController};

    async fn setup_controller() -> ZigbeeController {
        ZigbeeController::new("localhost", 1883).await
    }

    #[tokio::test]
    async fn turn_all_on_success() {
        let controller = setup_controller().await;
        let result = controller
            .turn_all_on(Brightness::Medium, ColorTemp::White)
            .await;

        assert!(
            result.is_ok(),
            "Controller failed to turn on all lights: {}",
            result.err().unwrap().to_string(),
        );

        let result = controller
            .turn_all_on(Brightness::Max, ColorTemp::Warm)
            .await;

        assert!(
            result.is_ok(),
            "Controller failed to turn on all lights: {}",
            result.err().unwrap().to_string(),
        );
    }

    #[tokio::test]
    async fn turn_all_off_success() {
        let controller = setup_controller().await;
        let result = controller.turn_all_off().await;

        assert!(
            result.is_ok(),
            "Controller failed to turn off all lights: {}",
            result.err().unwrap().to_string(),
        );
    }
}
