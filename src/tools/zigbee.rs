use rumqttc::{AsyncClient, ClientError, Event, MqttOptions, Packet, QoS};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_repr::Serialize_repr;
use std::time::Duration;

#[derive(Debug, Clone, strum::Display)]
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
    Controller,
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
    Low = 50,
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

#[derive(Deserialize, Debug)]
struct ControllerMessage {
    action: Option<String>,
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
        println!("[ZigbeeController] Initializing MQTT client connecting to {}:{}", broker_host, broker_port);
        
        let mut mqttoptions = MqttOptions::new("son-of-anton", broker_host, broker_port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

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

        let controller = Self {
            client: client.clone(),
            livingroom,
            kitchen,
            bedroom,
            sleep_duration,
        };

        // Subscribe to controller topic
        let controller_topic = format!("zigbee2mqtt/{}", DeviceName::Controller.to_string());
        println!("[ZigbeeController] Subscribing to controller topic: {}", controller_topic);
        
        if let Err(e) = client.subscribe(&controller_topic, QoS::AtLeastOnce).await {
            eprintln!("[ZigbeeController] Failed to subscribe to controller topic: {:?}", e);
        } else {
            println!("[ZigbeeController] Successfully subscribed to controller");
        }

        // Spawn the event loop with message handling
        let controller_clone = controller.clone();
        tokio::spawn(async move {
            println!("[ZigbeeController] Event loop started, listening for controller messages...");
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Packet::Publish(publish))) => {
                        // Check if this is from the controller
                        if publish.topic.contains(&DeviceName::Controller.to_string()) {
                            println!("[ZigbeeController] Received message from controller");
                            
                            if let Ok(payload_str) = std::str::from_utf8(&publish.payload) {
                                println!("[ZigbeeController] Payload: {}", payload_str);
                                
                                if let Ok(msg) = serde_json::from_str::<ControllerMessage>(payload_str) {
                                    if let Some(action) = msg.action {
                                        println!("[ZigbeeController] Action detected: {}", action);
                                        controller_clone.handle_controller_action(&action).await;
                                    }
                                } else {
                                    eprintln!("[ZigbeeController] Failed to parse controller message");
                                }
                            }
                        }
                    }
                    Ok(_) => {
                        // Other MQTT events (connections, acks, etc.)
                    }
                    Err(e) => {
                        eprintln!("[ZigbeeController] MQTT Error: {:?}", e);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        });

        println!("[ZigbeeController] Initialization complete");
        controller
    }

    async fn handle_controller_action(&self, action: &str) {
        println!("[ZigbeeController] Handling action: {}", action);
        
        let result = match action {
            // Straight down - all off
            "arrow_down_click" | "off" | "brightness_down_click" => {
                println!("[ZigbeeController] Turning all lights OFF");
                self.turn_all_off().await
            }
            // Straight up - all on
            "arrow_up_click" | "on" | "brightness_up_click" => {
                println!("[ZigbeeController] Turning all lights ON");
                self.turn_all_on(Brightness::Medium, ColorTemp::White).await
            }
            // Left - movie mode
            "arrow_left_click" | "left" => {
                println!("[ZigbeeController] Activating MOVIE MODE");
                self.movie_mode().await
            }
            // Right - party mode
            "arrow_right_click" | "right" => {
                println!("[ZigbeeController] Activating PARTY MODE");
                self.party_mode().await
            }
            _ => {
                println!("[ZigbeeController] Unknown action: {}", action);
                return;
            }
        };

        match result {
            Ok(_) => println!("[ZigbeeController] Action completed successfully"),
            Err(e) => eprintln!("[ZigbeeController] Action failed: {:?}", e),
        }
    }

    async fn turn_on(&self, device_name: &DeviceName) -> Result<(), ClientError> {
        println!("[ZigbeeController] Turning ON: {}", device_name);
        let topic = format!("zigbee2mqtt/{}/set", device_name.to_string());
        let payload = json!({ "state": "ON" }).to_string();
        self.client
            .publish(&topic, QoS::AtLeastOnce, false, payload)
            .await
    }

    async fn turn_off(&self, device_name: &DeviceName) -> Result<(), ClientError> {
        println!("[ZigbeeController] Turning OFF: {}", device_name);
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
        println!("[ZigbeeController] Sending payload to {}: brightness={:?}, color_temp={:?}, state={:?}",
                 device_name, payload.brightness, payload.color_temp, payload.state);
        let topic = format!("zigbee2mqtt/{}/set", device_name.to_string());
        let payload = serde_json::to_string(payload).unwrap(); // TODO FIX
        self.client
            .publish(&topic, QoS::AtLeastOnce, false, payload)
            .await
    }

    pub async fn turn_all_off(&self) -> Result<(), ClientError> {
        println!("[ZigbeeController] === Starting turn_all_off sequence ===");
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
            self.turn_off(device_name).await?;
        }
        self.sleep().await;
        self.turn_off(&DeviceName::IkeaDonut).await?;
        self.sleep().await;
        self.turn_off(&DeviceName::LightBulb).await?;
        for device_name in &self.bedroom {
            self.sleep().await;
            self.turn_off(device_name).await?;
        }
        println!("[ZigbeeController] === Completed turn_all_off sequence ===");
        Ok(())
    }

    pub async fn turn_all_on(
        &self,
        brightness: Brightness,
        color_temp: ColorTemp,
    ) -> Result<(), ClientError> {
        println!("[ZigbeeController] === Starting turn_all_on sequence ===");
        let payload = DevicePayload {
            state: LightState::On,
            brightness,
            color_temp,
        };
        for device_name in &self.kitchen {
            self.sleep().await;
            self.send_payload(device_name, &payload).await?;
        }
        self.sleep().await;
        self.turn_on(&DeviceName::BallLight).await?;
        self.sleep().await;
        self.turn_on(&DeviceName::IkeaMushroom).await?;
        self.sleep().await;
        self.turn_on(&DeviceName::SofaLight).await?;
        for device_name in &self.livingroom {
            self.sleep().await;
            self.send_payload(device_name, &payload).await?;
        }
        self.sleep().await;
        self.turn_on(&DeviceName::IkeaDonut).await?;
        self.sleep().await;
        self.turn_on(&DeviceName::LightBulb).await?;
        for device_name in &self.bedroom {
            self.sleep().await;
            self.send_payload(device_name, &payload).await?;
        }
        println!("[ZigbeeController] === Completed turn_all_on sequence ===");
        Ok(())
    }

    async fn sleep(&self) {
        tokio::time::sleep(Duration::from_millis(self.sleep_duration)).await
    }

    pub async fn _night_mode(&self) -> Result<(), ClientError> {
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
        println!("[ZigbeeController] === Starting MOVIE MODE sequence ===");
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
        println!("[ZigbeeController] === Completed MOVIE MODE sequence ===");
        Ok(())
    }

    pub async fn party_mode(&self) -> Result<(), ClientError> {
        println!("[ZigbeeController] === Starting PARTY MODE sequence ===");
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
        println!("[ZigbeeController] === Completed PARTY MODE sequence ===");
        Ok(())
    }
}
