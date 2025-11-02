use std::{collections::HashMap, env};

use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use sqlx::{PgPool, postgres::PgListener};
use zbus::{
    Connection, Result,
    fdo::{ObjectManagerProxy, PropertiesProxy},
    names::InterfaceName,
    proxy,
    zvariant::{ObjectPath, OwnedObjectPath, Type},
};

async fn fetch_and_process(pool: &PgPool) -> Option<()> {
    let mut tr = pool.begin().await.expect("Failed to start transaction");

    let message = sqlx::query!("SELECT * FROM messages WHERE NOT processed LIMIT 1 ")
        .fetch_optional(&mut *tr)
        .await
        .expect("Failed to retrieve message from db")?;

    log::info!(
        "Message {} is to be send: {} to {}",
        message.id,
        message.content,
        message.phone
    );

    tr.commit().await.expect("Failed to commit transaction");
    Some(())
}

#[proxy(
    default_service = "org.freedesktop.ModemManager1",
    interface = "org.freedesktop.ModemManager1.Modem"
)]
trait Modem {
    fn enable(&self, state: bool) -> Result<()>;

    #[zbus(property)]
    fn state(&self) -> Result<i32>;

    //#[zbus(signal)]
    //fn state_changed(&self, old: i32, new: i32, reason: u32) -> Result<()>;

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

    fn create(&self, properties: Vec<(String, String)>) -> Result<()>;

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

    let listener_task = tokio::spawn(async move {
        let (_, db_url) = env::vars()
            .find(|(k, v)| k == "DATABASE_URL")
            .expect("No 'DATABASE_URL' specified");

        let pool = PgPool::connect(&db_url)
            .await
            .expect("Failed to connect to postgres database");

        let mut listener = PgListener::connect_with(&pool)
            .await
            .expect("Failed to connect to postgres events");

        listener
            .listen("sent")
            .await
            .expect("Failed to start listening on 'sent' channel");

        while let Some(_) = fetch_and_process(&pool).await {
            log::info!("Processed stale message");
        }

        loop {
            match listener.recv().await {
                Ok(event) => {
                    log::info!("Received notification for outgoing message: {:?}", event);
                    fetch_and_process(&pool).await;
                }
                Err(e) => {
                    log::warn!("Failure while listening for events: {}", e);
                }
            }
        }
    });

    log::info!("Connected to system bus");
    let object_manager = ObjectManagerProxy::new(
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

    let tree = object_manager.get_managed_objects().await?;

    for (key1, val1) in tree.clone() {
        log::info!("{:?}", key1);
        for (key2, val2) in val1 {
            log::info!("\t{:?}", key2);
            for (key3, val3) in val2 {
                log::info!("\t\t{:?} {:?}", key3, val3);
            }
        }
    }

    let modem_path = tree
        .keys()
        .find(|m| m.contains("/org/freedesktop/ModemManager1/Modem"));

    if let Some(modem_path) = modem_path {
        let modem_proxy = ModemProxy::builder(&connection)
            .path(modem_path.clone())?
            .build()
            .await?;

        let status = modem_proxy.state().await?;
        let unlock_required = modem_proxy.unlock_required().await?;
        //modem_proxy.enable(true).await?;
        //log::info!("modem enabled");
        log::info!(
            "current modem status: {:?} unlock required: {:?}",
            status,
            unlock_required
        );

        let sim_proxy = SimProxy::new(&connection).await?;

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
            let mut sms_stream = stream::iter(list).chain(
                messaging
                    .receive_added()
                    .await?
                    .map(|s| s.args().expect("Failed to parse sms event").path),
            );

            while let Some(sms_path) = sms_stream.next().await {
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
        tokio::spawn(async move {
            let mut state_stream = modem_proxy.receive_state_changed().await;
            while let Some(state) = state_stream.next().await {
                log::info!("My state keep on changin' {}", 1);
            }
        });

        let props = modem_properties
            .get_all(InterfaceName::try_from("org.freedesktop.ModemManager1.Modem").unwrap())
            .await?;

        for prop in props {
            log::info!("Prop: {:?}", prop);
        }
    } else {
        log::error!(
            "Failed to access Modem. Make sure ModemManager service is running and has modem available"
        )
    }

    Ok(())
}
