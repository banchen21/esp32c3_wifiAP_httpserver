use core::str;

use esp_idf_svc::{
    eventloop::EspSystemEventLoop, 
    hal::prelude::Peripherals, http::{client::{self, Configuration as HttpConfiguration, EspHttpConnection}, Method},
     wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration as WifiConfiguration, EspWifi}};
use log::info;
use anyhow::{bail, Result};

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    let ssid="Redmi Turbo 3";
    let pass="";

    let auth_method = AuthMethod::None;
    let mut esp_wifi = EspWifi::new(peripherals.modem, sysloop.clone(), None)?;
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;
    wifi.set_configuration(&WifiConfiguration::Client(ClientConfiguration::default()))?;

    info!("Starting wifi...");

    wifi.start()?;
    info!("Scanning...");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == ssid);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            ssid, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            ssid
        );
        None
    };

    wifi.set_configuration(&WifiConfiguration::Client(ClientConfiguration {
        ssid: ssid
            .try_into()
            .expect("Could not parse the given SSID into WiFi config"),
        password: pass
            .try_into()
            .expect("Could not parse the given password into WiFi config"),
        channel,
        auth_method,
        ..Default::default()
    }))?;

    info!("连接wifi中");

    wifi.connect()?;

    info!("向wifi获取DHCP中");

    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    info!("Wifi DHCP 信息: {:?}", ip_info);
    get("http://neverssl.com/")?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
fn get(url: impl AsRef<str>) -> Result<()> {
    let config = HttpConfiguration::default();
    let connection = EspHttpConnection::new(&config)?;
    let mut client = embedded_svc::http::client::Client::wrap(connection);
    let headers = [("accept", "text/plain")];

    let request=client.request(Method::Get, url.as_ref(), &headers)?;
    let response = request.submit()?;
    let status = response.status();
    println!("Response code: {}\n", status);
    match status {
        200..=299 => {
            let mut buf = [0_u8; 256];
            let mut offset = 0;
            let mut total = 0;
            let mut reader = response;
            loop {
                if let Ok(size) = embedded_svc::io::Read::read(&mut reader, &mut buf[offset..]) {
                    if size == 0 {
                        break;
                    }
                    total += size;
                    let size_plus_offset = size + offset;
                    match str::from_utf8(&buf[..size_plus_offset]) {
                        Ok(text) => {
                            print!("{}", text);
                            offset = 0;
                        }
                        Err(error) => {
                            let valid_up_to = error.valid_up_to();
                            unsafe {
                                print!("{}", str::from_utf8_unchecked(&buf[..valid_up_to]));
                            }
                            buf.copy_within(valid_up_to.., 0);
                            offset = size_plus_offset - valid_up_to;
                        }
                    }
                }
            }
            println!("Total: {} bytes", total);
        }
        _ => bail!("Unexpected response code: {}", status),
    }

    Ok(())
}