use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
struct LightPayload {
    state: String,
    brightness: u8,
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

    /// Turn a light on
    pub async fn light_on(&self, device_name: &str) -> Result<(), rumqttc::ClientError> {
        let topic = format!("zigbee2mqtt/{}/set", device_name);
        let payload = serde_json::to_string(&LightState {
            state: "ON".to_string(),
        })
        .unwrap();

        self.client
            .publish(&topic, QoS::AtLeastOnce, false, payload)
            .await
    }

    /// Turn a light off
    pub async fn light_off(&self, device_name: &str) -> Result<(), rumqttc::ClientError> {
        let topic = format!("zigbee2mqtt/{}/set", device_name);
        let payload = serde_json::to_string(&LightState {
            state: "OFF".to_string(),
        })
        .unwrap();

        self.client
            .publish(&topic, QoS::AtLeastOnce, false, payload)
            .await
    }

    /// Set light brightness (0-254)
    pub async fn set_brightness(
        &self,
        device_name: &str,
        brightness: u8,
    ) -> Result<(), rumqttc::ClientError> {
        let topic = format!("zigbee2mqtt/{}/set", device_name);
        let payload = serde_json::to_string(&LightPayload {
            state: "ON".to_string(),
            brightness,
        })
        .unwrap();

        self.client
            .publish(&topic, QoS::AtLeastOnce, false, payload)
            .await
    }

    /// Toggle a light
    pub async fn toggle(&self, device_name: &str) -> Result<(), rumqttc::ClientError> {
        let topic = format!("zigbee2mqtt/{}/set", device_name);
        let payload = serde_json::to_string(&LightState {
            state: "TOGGLE".to_string(),
        })
        .unwrap();

        self.client
            .publish(&topic, QoS::AtLeastOnce, false, payload)
            .await
    }
}

mod tools;

#[tokio::main]
async fn main() {
    // Connect to Mosquitto
    let controller = ZigbeeController::new("localhost", 1883).await;

    let all_lights: Vec<&str> = vec![
        "hue_kitchen_1",
        "hue_kitchen_2",
        "hue_kitchen_3",
        "hue_bedroom_1",
        "hue_bedroom_2",
        "hue_bedroom_3",
        "hue_livingroom_1",
        "hue_livingroom_2",
        "hue_livingroom_3",
        "hue_livingroom_4",
        "hue_livingroom_5",
        "hue_livingroom_6",
        "ikea_mushroom",
    ];

    for light_name in &all_lights {
        // Turn light on
        println!("Turning light on...");
        controller.light_off(light_name).await.unwrap();
    }

    println!("Sleeping");
    tokio::time::sleep(Duration::from_secs(3)).await;

    for light_name in &all_lights {
        println!("Turning light off...");
        controller.light_on(light_name).await.unwrap();
    }
    println!("Done!");
}
