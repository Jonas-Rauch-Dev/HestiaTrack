use std::{thread::sleep, time::Duration};

use accesspoint_strengths::get_accesspoint_strengths;
use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::prelude::Peripherals, mqtt::client::QoS};
use anyhow::Result;
use mqtt::MQTTClient;
use wifi::wifi;

mod wifi;
mod mqtt;
mod accesspoint_strengths;

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

    // Start and Connect wifi
    let mut wifi = wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem, sysloop
    )?;

    // Create mqtt client
    let mut mqtt_client = MQTTClient::new(
        app_config.mqtt_host, 
        app_config.mqtt_user, 
        app_config.mqtt_pass
    )?;

    // Main programm loop
    loop {
        match get_accesspoint_strengths(&mut wifi) {
            Ok(accesspoint_strengths_string) => {
                mqtt_client.enqueue_mqtt_message(
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