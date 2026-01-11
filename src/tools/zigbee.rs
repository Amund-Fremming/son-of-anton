use std::time::Duration;

use rumqttc::{AsyncClient, ClientError, MqttOptions, QoS};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum DeviceName {
    #[serde(rename = "hue_kitchen_1")]
    HueKitchen1,
    #[serde(rename = "hue_kitchen_2")]
    HueKitchen2,
    #[serde(rename = "hue_kitchen_3")]
    HueKitchen3,
    #[serde(rename = "hue_bedroom_1")]
    HueBedroom1,
    #[serde(rename = "hue_bedroom_2")]
    HueBedroom2,
    #[serde(rename = "hue_bedroom_3")]
    HueBedroom3,
    #[serde(rename = "hue_livingroom_1")]
    HueLivingroom1,
    #[serde(rename = "hue_livingroom_2")]
    HueLivingroom2,
    #[serde(rename = "hue_livingroom_3")]
    HueLivingroom3,
    #[serde(rename = "hue_livingroom_4")]
    HueLivingroom4,
    #[serde(rename = "hue_livingroom_5")]
    HueLivingroom5,
    #[serde(rename = "hue_livingroom_6")]
    HueLivingroom6,
    #[serde(rename = "ikea_mushroom")]
    IkeaMushroom,
}

impl DeviceName {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceName::HueKitchen1 => "hue_kitchen_1",
            DeviceName::HueKitchen2 => "hue_kitchen_2",
            DeviceName::HueKitchen3 => "hue_kitchen_3",
            DeviceName::HueBedroom1 => "hue_bedroom_1",
            DeviceName::HueBedroom2 => "hue_bedroom_2",
            DeviceName::HueBedroom3 => "hue_bedroom_3",
            DeviceName::HueLivingroom1 => "hue_livingroom_1",
            DeviceName::HueLivingroom2 => "hue_livingroom_2",
            DeviceName::HueLivingroom3 => "hue_livingroom_3",
            DeviceName::HueLivingroom4 => "hue_livingroom_4",
            DeviceName::HueLivingroom5 => "hue_livingroom_5",
            DeviceName::HueLivingroom6 => "hue_livingroom_6",
            DeviceName::IkeaMushroom => "ikea_mushroom",
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
#[derive(Serialize, Deserialize, Debug)]
pub enum ColorTemp {
    Blue = 250,
    White = 352,
    Warm = 454,
}

/// Range: 0-254
#[derive(Serialize, Deserialize, Debug)]
pub enum Brightness {
    Min = 8,
    Low = 64,
    Medium = 127,
    High = 191,
    Max = 254,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LightPayload {
    pub state: LightState,
    pub brightness: Brightness,
    pub color_temp: ColorTemp,
}

impl LightPayload {
    pub fn on() -> Self {
        Self {
            state: LightState::On,
            brightness: Brightness::High,
            color_temp: ColorTemp::White,
        }
    }

    pub fn off() -> Self {
        Self {
            state: LightState::Off,
            brightness: Brightness::Low,
            color_temp: ColorTemp::White,
        }
    }
}

pub struct ZigbeeController {
    topic: String,
    client: AsyncClient,
    living_room: Vec<DeviceName>,
    kitchen: Vec<DeviceName>,
    bedroom: Vec<DeviceName>,
}

impl ZigbeeController {
    pub async fn new(broker_host: &str, broker_port: u16, topic: &str) -> Self {
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

        let living_room = vec![
            DeviceName::HueLivingroom1,
            DeviceName::HueLivingroom2,
            DeviceName::HueLivingroom3,
            DeviceName::HueLivingroom4,
            DeviceName::HueLivingroom5,
            DeviceName::HueLivingroom6,
            DeviceName::IkeaMushroom,
        ];

        let kitchen = vec![
            DeviceName::HueKitchen1,
            DeviceName::HueKitchen2,
            DeviceName::HueKitchen3,
        ];

        let bedroom = vec![
            DeviceName::HueBedroom1,
            DeviceName::HueBedroom2,
            DeviceName::HueBedroom3,
        ];

        Self {
            topic: topic.to_string(),
            client,
            living_room,
            kitchen,
            bedroom,
        }
    }

    pub async fn turn_off(&self, device_name: DeviceName) -> Result<(), ClientError> {
        let topic = format!("zigbee2mqtt/{}/set", device_name.as_str());
        let payload = serde_json::to_string(&LightPayload::off()).unwrap();

        self.client
            .publish(&topic, QoS::AtLeastOnce, false, payload)
            .await
    }

    pub fn turn_all_on(&self) {
        for device in self.bedroom {
            //
        }
    }

    pub fn turn_all_off(&self) {}
}
