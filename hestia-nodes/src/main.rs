use std::{thread::sleep, time::Duration};

use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::{prelude::Peripherals, temp_sensor}, mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS}};
use anyhow::{Ok, Result};
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

    let mut wifi = wifi(app_config.wifi_ssid, app_config.wifi_psk, peripherals.modem, sysloop)?;

    let (access_point_infos, number_of_access_points) = wifi.scan_n::<10>()?;

    access_point_infos.iter().for_each(|api| log::info!("AccessPoint: {:?}", api));

    let mut mqtt_client_config = MqttClientConfiguration::default();
    mqtt_client_config.username = Some(app_config.mqtt_user);
    mqtt_client_config.password = Some(app_config.mqtt_pass);

    let mut mqtt_client = EspMqttClient::new_cb(app_config.mqtt_host, &mqtt_client_config, move |message_event| {
        log::info!("Message Event: {:?}", message_event.payload())
    })?;


    loop {
        sleep(Duration::from_secs(5));
        mqtt_client.enqueue("test/topic", QoS::AtMostOnce, true, &[78, 73, 74])?;
    }
}