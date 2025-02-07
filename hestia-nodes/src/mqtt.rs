use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS};
use anyhow::Result;

pub struct MQTTClient {
    esp_mqtt_client: EspMqttClient<'static>
}

impl MQTTClient {
    pub fn new(
        mqtt_host: &str,
        mqtt_user: &str,
        mqtt_pass: &str,
    ) -> Result<Self> {
        // Create mqtt client
        let mut mqtt_client_config = MqttClientConfiguration::default();
        mqtt_client_config.username = Some(mqtt_user);
        mqtt_client_config.password = Some(mqtt_pass);

        let mqtt_client = EspMqttClient::new_cb(
            mqtt_host,
            &mqtt_client_config,
            move |message_event| {
                log::info!("Message Event: {:?}", message_event.payload())
            }
        )?;

        Ok(Self {
            esp_mqtt_client: mqtt_client
        })
    }

    pub fn enqueue_mqtt_message(
        &mut self,
        topic: &str,
        qos: QoS,
        retain: bool,
        payload: &[u8]
    ) {
        if let Err(e) = self.esp_mqtt_client.enqueue(topic, qos, retain, payload) {
            log::error!(
                "ERROR: Failed to enqueue mqtt message into topic '{}' with error: {}",
                topic,
                e
            );
        }
    }
}
