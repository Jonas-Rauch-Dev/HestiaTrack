use esp_idf_svc::wifi::{AccessPointInfo, EspWifi};
use anyhow::Result;

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

pub fn get_accesspoint_strengths(wifi: &mut Box<EspWifi<'static>>) -> Result<String> {
    let accesspoint_infos = wifi.scan()?;

    let accesspoint_strengths: Vec<AccessPointSignalStrength> = accesspoint_infos
        .into_iter()
        .map(AccessPointSignalStrength::from)
        .collect();
    
    let serialized_accesspoint_strengths = serde_json::to_string(&accesspoint_strengths)?;

    Ok(serialized_accesspoint_strengths)
}