use rumqttc::{AsyncClient, ClientError, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_repr::Serialize_repr;
use std::time::Duration;
use strum::IntoEnumIterator;

#[derive(Debug, Clone, strum::EnumIter, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum LightState {
    On,
    Off,
}

/// Range: 250-454
#[allow(dead_code)]
#[repr(u16)]
#[derive(Debug, Serialize_repr)]
pub enum ColorTemp {
    Blue = 250,
    White = 352,
    Warm = 454,
}

/// Range: 0-254
#[allow(dead_code)]
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
pub struct DevicePayload {
    pub state: LightState,
    pub brightness: Brightness,
    pub color_temp: ColorTemp,
}

#[derive(Clone)]
pub struct ZigbeeController {
    client: AsyncClient,
    sleep_duration: u64,
    livingroom: Vec<DeviceName>,
    kitchen: Vec<DeviceName>,
    bedroom: Vec<DeviceName>,
}

impl ZigbeeController {
    pub async fn new(broker_host: &str, broker_port: u16, sleep_duration: u64) -> Self {
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

        let livingroom = vec![
            DeviceName::HueLivingroom1,
            DeviceName::HueLivingroom2,
            DeviceName::HueLivingroom3,
            DeviceName::HueLivingroom4,
            DeviceName::HueLivingroom5,
            DeviceName::HueLivingroom6,
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
            client,
            livingroom,
            kitchen,
            bedroom,
            sleep_duration,
        }
    }

    async fn turn_on(&self, device_name: &DeviceName) -> Result<(), ClientError> {
        let topic = format!("zigbee2mqtt/{}/set", device_name.to_string());
        let payload = json!({ "state": "ON" }).to_string();
        self.client
            .publish(&topic, QoS::AtLeastOnce, false, payload)
            .await
    }

    async fn turn_off(&self, device_name: &DeviceName) -> Result<(), ClientError> {
        let topic = format!("zigbee2mqtt/{}/set", device_name.to_string());
        let payload = json!({ "state": "OFF" }).to_string();
        self.client
            .publish(&topic, QoS::AtLeastOnce, false, payload)
            .await
    }

    async fn send_payload(
        &self,
        device_name: &DeviceName,
        payload: &DevicePayload,
    ) -> Result<(), ClientError> {
        let topic = format!("zigbee2mqtt/{}/set", device_name.to_string());
        let payload = serde_json::to_string(payload).unwrap(); // TODO FIX
        self.client
            .publish(&topic, QoS::AtLeastOnce, false, payload)
            .await
    }

    pub async fn turn_all_off(&self) -> Result<(), ClientError> {
        for device_name in DeviceName::iter() {
            self.sleep().await;
            self.turn_off(&device_name).await?;
        }
        self.sleep().await;
        Ok(())
    }

    pub async fn turn_all_on(
        &self,
        brightness: Brightness,
        color_temp: ColorTemp,
    ) -> Result<(), ClientError> {
        let payload = DevicePayload {
            state: LightState::On,
            brightness,
            color_temp,
        };
        for device_name in DeviceName::iter() {
            self.sleep().await;
            self.send_payload(&device_name, &payload).await?;
        }
        self.sleep().await;
        Ok(())
    }

    async fn sleep(&self) {
        tokio::time::sleep(Duration::from_millis(self.sleep_duration)).await
    }

    pub async fn night_mode(&self) -> Result<(), ClientError> {
        for device_name in &self.kitchen {
            self.sleep().await;
            self.turn_off(device_name).await?;
        }
        self.sleep().await;
        self.turn_off(&DeviceName::BallLight).await?;
        self.sleep().await;
        self.turn_off(&DeviceName::IkeaMushroom).await?;
        self.sleep().await;
        self.turn_off(&DeviceName::SofaLight).await?;
        for device_name in &self.livingroom {
            self.sleep().await;
            self.send_payload(
                device_name,
                &DevicePayload {
                    state: LightState::On,
                    brightness: Brightness::Min,
                    color_temp: ColorTemp::Warm,
                },
            )
            .await?;
        }
        self.sleep().await;
        self.turn_off(&DeviceName::IkeaDonut).await?;
        self.sleep().await;
        self.turn_off(&DeviceName::LightBulb).await?;
        for device_name in &self.bedroom {
            self.sleep().await;
            self.turn_off(device_name).await?;
        }
        self.sleep().await;
        Ok(())
    }

    pub async fn movie_mode(&self) -> Result<(), ClientError> {
        for device_name in &self.kitchen {
            self.sleep().await;
            self.turn_off(device_name).await?;
        }
        self.sleep().await;
        self.turn_off(&DeviceName::BallLight).await?;
        self.sleep().await;
        self.turn_off(&DeviceName::IkeaMushroom).await?;
        for device_name in &self.livingroom {
            self.sleep().await;
            self.send_payload(
                device_name,
                &DevicePayload {
                    state: LightState::On,
                    brightness: Brightness::Min,
                    color_temp: ColorTemp::Warm,
                },
            )
            .await?;
        }
        self.sleep().await;
        self.turn_on(&DeviceName::SofaLight).await?;
        self.sleep().await;
        self.turn_on(&DeviceName::IkeaDonut).await?;
        self.sleep().await;
        self.turn_on(&DeviceName::LightBulb).await?;
        for device_name in &self.bedroom {
            self.sleep().await;
            self.turn_off(device_name).await?;
        }
        self.sleep().await;
        Ok(())
    }

    pub async fn party_mode(&self) -> Result<(), ClientError> {
        for device_name in &self.kitchen {
            self.sleep().await;
            self.send_payload(
                device_name,
                &DevicePayload {
                    state: LightState::On,
                    brightness: Brightness::Low,
                    color_temp: ColorTemp::Warm,
                },
            )
            .await?;
        }
        self.sleep().await;
        self.turn_off(&DeviceName::BallLight).await?;
        self.sleep().await;
        self.send_payload(
            &DeviceName::IkeaMushroom,
            &DevicePayload {
                state: LightState::On,
                brightness: Brightness::Medium,
                color_temp: ColorTemp::Warm,
            },
        )
        .await?;
        self.sleep().await;
        self.turn_on(&DeviceName::SofaLight).await?;
        for device_name in &self.livingroom {
            self.sleep().await;
            self.send_payload(
                &device_name,
                &DevicePayload {
                    state: LightState::On,
                    brightness: Brightness::Low,
                    color_temp: ColorTemp::Warm,
                },
            )
            .await?;
        }
        self.sleep().await;
        self.turn_on(&DeviceName::IkeaDonut).await?;
        self.sleep().await;
        self.turn_on(&DeviceName::LightBulb).await?;
        for device_name in &self.bedroom {
            self.sleep().await;
            self.send_payload(
                device_name,
                &DevicePayload {
                    state: LightState::On,
                    brightness: Brightness::Low,
                    color_temp: ColorTemp::Warm,
                },
            )
            .await?;
        }
        self.sleep().await;
        Ok(())
    }
}
