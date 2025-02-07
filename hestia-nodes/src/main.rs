use std::{thread::sleep, time::Duration};

use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::{prelude::Peripherals, temp_sensor}, mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS}, wifi::{AccessPointInfo, EspWifi}};
use anyhow::Result;
use wifi::wifi;

mod wifi;

#[toml_cfg::toml_config]
struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default("")]
    mqtt_user: &'static str,
    #[default("")]
    mqtt_pass: &'static str,
    #[default("")]
    mqtt_host: &'static str,
}

fn main() -> Result<()>{
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;


    let app_config = CONFIG;

    // Create wifi
    let mut wifi = wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem, sysloop
    )?;

    // Create mqtt client
    let mut mqtt_client_config = MqttClientConfiguration::default();
    mqtt_client_config.username = Some(app_config.mqtt_user);
    mqtt_client_config.password = Some(app_config.mqtt_pass);

    let mut mqtt_client = EspMqttClient::new_cb(app_config.mqtt_host, &mqtt_client_config, move |message_event| {
        log::info!("Message Event: {:?}", message_event.payload())
    })?;

    // Main programm loop
    loop {
        match get_accesspoint_strengths(&mut wifi) {
            Ok(accesspoint_strengths_string) => {
                enqueue_mqtt_message(
                    &mut mqtt_client,
                    "test/topic",
                    QoS::AtMostOnce,
                    true,
                    accesspoint_strengths_string.as_bytes()
                );
            },
            Err(e) => {
                log::error!("ERROR: Failed to get accesspoint strengths: {}", e);
            }
        }
        sleep(Duration::from_secs(5));
    }
}

#[derive(serde::Serialize)]
struct AccessPointSignalStrength {
    ssid: heapless::String<32>,
    signal_strength: i8,
}

impl AccessPointSignalStrength {
    fn from(acces_point_info: AccessPointInfo) -> Self {
        Self {
            ssid: acces_point_info.ssid,
            signal_strength: acces_point_info.signal_strength
        }
    }
}

fn get_accesspoint_strengths(wifi: &mut Box<EspWifi<'static>>) -> Result<String> {
    let accesspoint_infos = wifi.scan()?;

    let accesspoint_strengths: Vec<AccessPointSignalStrength> = accesspoint_infos
        .into_iter()
        .map(AccessPointSignalStrength::from)
        .collect();
    
    let serialized_accesspoint_strengths = serde_json::to_string(&accesspoint_strengths)?;

    Ok(serialized_accesspoint_strengths)
}

fn enqueue_mqtt_message(
    mqtt_client: &mut EspMqttClient<'static>,
    topic: &str,
    qos: QoS,
    retain: bool,
    payload: &[u8]
) {
    if let Err(e) = mqtt_client.enqueue(topic, qos, retain, payload) {
        log::error!(
            "ERROR: Failed to enqueue mqtt message into topic '{}' with error: {}",
            topic,
            e
        );
    }
}