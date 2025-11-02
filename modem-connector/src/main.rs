use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use zbus::{
    Connection, Result,
    fdo::{ObjectManagerProxy, PropertiesProxy},
    names::InterfaceName,
    proxy,
    zvariant::{ObjectPath, OwnedObjectPath, Type},
};

#[proxy(
    default_service = "org.freedesktop.ModemManager1",
    interface = "org.freedesktop.ModemManager1.Modem"
)]
trait Modem {
    fn enable(&self, state: bool) -> Result<()>;
    #[zbus(property)]
    fn state(&self) -> Result<i32>;
    #[zbus(property)]
    fn unlock_required(&self) -> Result<u32>;
}

#[proxy(
    default_service = "org.freedesktop.ModemManager1",
    interface = "org.freedesktop.ModemManager1.Sms"
)]
trait Sms {
    #[zbus(property)]
    fn text(&self) -> Result<String>;

    #[zbus(property)]
    fn timestamp(&self) -> Result<String>;

    #[zbus(property)]
    fn number(&self) -> Result<String>;
}

#[proxy(
    default_service = "org.freedesktop.ModemManager1",
    interface = "org.freedesktop.ModemManager1.Modem.Messaging"
)]
trait Messaging {
    fn list(&self) -> Result<Vec<OwnedObjectPath>>;
    #[zbus(signal)]
    fn added(&self, path: OwnedObjectPath, received: bool) -> Result<()>;
}

#[proxy(
    default_service = "org.freedesktop.ModemManager1",
    default_path = "/org/freedesktop/ModemManager1/SIM/0",
    interface = "org.freedesktop.ModemManager1.Sim"
)]
trait Sim {
    fn send_pin(&self, pin: String) -> Result<()>;
}

#[tokio::main]
async fn main() -> Result<()> {
    SimpleLogger::new().init().unwrap();
    let connection = Connection::system().await?;

    log::info!("Connected to system bus");
    let modem_proxy = ObjectManagerProxy::new(
        &connection,
        "org.freedesktop.ModemManager1",
        "/org/freedesktop/ModemManager1",
    )
    .await?;

    let modem_properties = PropertiesProxy::new(
        &connection,
        "org.freedesktop.ModemManager1",
        "/org/freedesktop/ModemManager1/Modem/0",
    )
    .await?;

    log::info!("Created modem proxy");
    let tree = modem_proxy.get_managed_objects().await?;

    for (key1, val1) in tree.clone() {
        log::info!("{:?}", key1);
        for (key2, val2) in val1 {
            log::info!("\t{:?}", key2);
            for (key3, val3) in val2 {
                log::info!("\t\t{:?} {:?}", key3, val3);
            }
        }
    }

    log::info!("Modem properties");

    for key in tree.keys() {
        log::info!("Key: {:?}", key);
    }

    let modem_path = tree
        .keys()
        .find(|m| m.contains("/org/freedesktop/ModemManager1/Modem"));

    if let Some(modem_path) = modem_path {
        let modem_proxy = ModemProxy::builder(&connection)
            .path(modem_path)?
            .build()
            .await?;

        let props = modem_properties
            .get_all(InterfaceName::try_from("org.freedesktop.ModemManager1.Modem").unwrap())
            .await?;

        for prop in props {
            log::info!("Prop: {:?}", prop);
        }

        let sim_proxy = SimProxy::new(&connection).await?;

        let status = modem_proxy.state().await?;
        let unlock_required = modem_proxy.unlock_required().await?;
        //modem_proxy.enable(true).await?;
        //log::info!("modem enabled");
        log::info!(
            "current modem status: {:?} unlock required: {:?}",
            status,
            unlock_required
        );

        if unlock_required == 2 {
            sim_proxy.send_pin(String::from("6538")).await?;
            log::info!("Send pin");

            let status = modem_proxy.state().await?;
            let unlock_required = modem_proxy.unlock_required().await?;
            //modem_proxy.enable(true).await?;
            //log::info!("modem enabled");
            log::info!(
                "current modem status: {:?} unlock required: {:?}",
                status,
                unlock_required
            );
        } else {
            let messaging = MessagingProxy::builder(&connection)
                .path(modem_path)?
                .build()
                .await?;
            let list = messaging.list().await?;
            log::info!("List of messages: {:?}", list);

            let new_

            for sms_path in list {
                let sms_object = SmsProxy::builder(&connection)
                    .path(sms_path)?
                    .build()
                    .await?;

                let phone = sms_object.number().await?;
                let content = sms_object.text().await?;
                let timestamp = sms_object.timestamp().await?;

                log::info!("Sms: {:?} {:?} {:?}", phone, content, timestamp);
            }
        }
    }

    Ok(())
}
