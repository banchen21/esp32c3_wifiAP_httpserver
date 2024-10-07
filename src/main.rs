use core::str;

use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::prelude::Peripherals,
    http::{
        server::{Configuration as ServerConfiguration, EspHttpServer},
        Method,
    },
    io::{EspIOError, Write},
    wifi::{
        AccessPointConfiguration, AuthMethod, BlockingWifi, Configuration as WifiConfiguration,
        EspWifi,
    },
};
use log::info;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    
    let mut esp_wifi = EspWifi::new(peripherals.modem, sysloop.clone(), None)?;
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;
    wifi.set_configuration(&WifiConfiguration::AccessPoint(
        AccessPointConfiguration::default(),
    ))?;
    wifi.start()?;

    let conf = ServerConfiguration::default();
    let mut esp_httpserver = EspHttpServer::new(&conf)?;
    esp_httpserver.fn_handler(
        "/",
        Method::Get,
        |request| -> core::result::Result<(), EspIOError> {
            let html = index_html();
            let mut response = request.into_ok_response()?;
            response.write_all(html.as_bytes())?;
            Ok(())
        },
    )?;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn templated(content: impl AsRef<str>) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">
        <title>esp-rs web server</title>
    </head>
    <body>
        {}
    </body>
</html>
"#,
        content.as_ref()
    )
}

fn index_html() -> String {
    templated("Hello from mcu!")
}
