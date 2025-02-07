use std::time::Duration;

use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS};
use anyhow::Result;

pub struct MQTTClient {
    esp_mqtt_client: EspMqttClient<'static>,
    client_id: &'static str,
}

impl MQTTClient {
    pub fn new(
        mqtt_host: &'static str,
        mqtt_user: &'static str,
        mqtt_pass: &'static str,
        client_id: &'static str,
    ) -> Result<Self> {
        let mut mqtt_client_config = MqttClientConfiguration::default();
        mqtt_client_config.username = Some(mqtt_user);
        mqtt_client_config.password = Some(mqtt_pass);
        mqtt_client_config.keep_alive_interval = Some(Duration::from_secs(5));
        mqtt_client_config.client_id = Some(client_id);

        let mqtt_client = EspMqttClient::new_cb(
            mqtt_host,
            &mqtt_client_config,
            move |message_event| {
                log::info!("Message Event: {:?}", message_event.payload())
            }
        )?;

        Ok(Self {
            esp_mqtt_client: mqtt_client,
            client_id
        })
    }

    pub fn enqueue_mqtt_message(
        &mut self,
        topic: Topic,
        payload: &[u8]
    ) {
        let send_result = self.esp_mqtt_client.enqueue(
            &topic.get_string(self.client_id),
            topic.get_qos(),
            topic.get_retain(),
            payload
        );

        if let Err(e) = send_result{
            log::error!(
                "ERROR: Failed to enqueue mqtt message into topic '{:?}' with error: {}",
                topic,
                e
            );
        }
    }
}


#[derive(Debug)]
pub enum Topic {
    AccesspointStrengths,
    SensorReadings,
}

impl Topic {
    fn get_string(&self, client_id: &str) -> String {
        format!("{}/{}", match self {
            Self::AccesspointStrengths => "accesspoint_strengths",
            Self::SensorReadings => "sensor_readings",
        }, client_id)
    }

    fn get_qos(&self) -> QoS {
        match self {
            Self::AccesspointStrengths => QoS::AtMostOnce,
            Self::SensorReadings => QoS::AtMostOnce,
        }
    }

    fn get_retain(&self) -> bool {
        match self {
            Self::AccesspointStrengths => true,
            Self::SensorReadings => true,
        }
    }
}
