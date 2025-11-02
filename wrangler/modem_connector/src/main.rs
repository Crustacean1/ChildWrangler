use std::{collections::HashMap, env};

use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use sqlx::{PgPool, postgres::PgListener, types::chrono::DateTime};
use zbus::{
    Connection, Result,
    fdo::ObjectManagerProxy,
    proxy,
    zvariant::{OwnedObjectPath, Type, Value},
};

async fn fetch_and_process<'a, 'b>(pool: &PgPool, connection: Connection) -> Option<()> {
    let mut tr = pool.begin().await.expect("Failed to start transaction");

    let message = sqlx::query!("SELECT * FROM messages WHERE NOT processed AND outgoing LIMIT 1 ")
        .fetch_optional(&mut *tr)
        .await
        .expect("Failed to retrieve message from db")?;

    sqlx::query!(
        "UPDATE messages SET processed = true WHERE id = $1",
        message.id
    )
    .execute(&mut *tr)
    .await
    .expect("Failed to mark message as processed, rollback");

    log::info!(
        "Message {} is to be send: {} to {}",
        message.id,
        message.content,
        message.phone
    );

    send_message(message.phone, message.content, connection)
        .await
        .expect("Failed to enqueue message");

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

    #[zbus(property)]
    fn state(&self) -> Result<u32>;

    fn send(&self) -> Result<()>;
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
struct SmsMessage {
    pub text: String,
    pub number: String,
}

async fn send_message<'a, 'b>(
    phone: String,
    content: String,
    connection: Connection,
) -> Result<()> {
    let messaging_proxy = MessagingProxy::builder(&connection)
        .path("/org/freedesktop/ModemManager1/Modem/0")?
        .build()
        .await?;

    let mut map = HashMap::new();
    map.insert("number", Value::Str(phone.into()));
    map.insert("text", Value::Str(content.into()));
    let sms_object = messaging_proxy.create(map).await?;

    log::info!("Created sms object: {:?}", sms_object.clone());

    let sms_proxy = SmsProxy::builder(&connection)
        .path(sms_object)?
        .build()
        .await?;

    sms_proxy.send().await?;

    Ok(())
}

#[proxy(
    default_service = "org.freedesktop.ModemManager1",
    interface = "org.freedesktop.ModemManager1.Modem.Messaging"
)]
trait Messaging {
    fn list(&self) -> Result<Vec<OwnedObjectPath>>;

    #[zbus(signal)]
    fn added(&self, path: OwnedObjectPath, received: bool) -> Result<()>;

    //fn create(&self, message: SmsMessage) -> Result<OwnedObjectPath>;
    fn create<'a>(&self, message: HashMap<&'static str, Value<'a>>) -> Result<OwnedObjectPath>;
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
    let object_manager = ObjectManagerProxy::new(
        &connection,
        "org.freedesktop.ModemManager1",
        "/org/freedesktop/ModemManager1",
    )
    .await?;

    let (_, db_url) = env::vars()
        .find(|(k, _)| k == "DATABASE_URL")
        .expect("No 'DATABASE_URL' specified");

    let pool = PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to postgres database");

    log::info!("Connected to postgres db with url: {}", db_url);

    let tree = object_manager.get_managed_objects().await?;

    let modem_path = tree
        .keys()
        .find(|m| m.contains("/org/freedesktop/ModemManager1/Modem"))
        .expect("Modem object not exposed by ModemManager, verify that service is enabled and running and modem is plugged in").clone();

    let modem_proxy = ModemProxy::builder(&connection)
        .path(modem_path.clone())?
        .build()
        .await?;

    let unlock_required = modem_proxy.unlock_required().await?;
    let status = modem_proxy.state().await?;

    log::info!(
        "current modem status: {:?} unlock required: {:?}",
        status,
        unlock_required
    );

    if unlock_required == 2 {
        let sim_proxy = SimProxy::new(&connection).await?;
        sim_proxy.send_pin(String::from("6538")).await?;
        log::info!("Send pin");

        let status = modem_proxy.state().await?;
        let unlock_required = modem_proxy.unlock_required().await?;

        log::info!(
            "current modem status: {:?} unlock required: {:?}",
            status,
            unlock_required
        );
    }
    let messaging = MessagingProxy::builder(&connection)
        .path("/org/freedesktop/ModemManager1/Modem/0")?
        .build()
        .await?;

    let modem_task = tokio::spawn({
        let connection = connection.clone();
        let messaging = messaging.clone();
        let pool = pool.clone();
        async move {
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
                let state = sms_object.state().await?;

                let timestamp = format!("{}:00", sms_object.timestamp().await?);
                log::info!(
                    "Received SMS: {:?} from: {:?} at: {:?} state: {:?}",
                    content,
                    phone,
                    timestamp,
                    state
                );

                if state == 3 {
                    if let Ok(timestamp) = DateTime::parse_from_rfc3339(&timestamp) {
                        let timestamp = timestamp.naive_utc();
                        let mut tr = pool
                            .begin()
                            .await
                            .expect("Failed to start postgres transaction");

                        let duplicate = sqlx::query!("SELECT id FROM messages WHERE phone = $1 AND sent = $2 AND content = $3 AND outgoing = false", phone,timestamp, content).fetch_optional(&mut *tr).await.expect("Failed to get existing messages from db").map(|row| row.id);

                        if let Some(id) = duplicate {
                            log::info!("Detected duplicate {}, skipping", id);
                        } else {
                            sqlx::query!("INSERT INTO messages (phone, content, outgoing, sent) VALUES ($1, $2, false, $3)", phone,content, timestamp).execute(&mut *tr).await.expect("Failed to save message into db");
                        }

                        tr.commit().await.expect("Failed to commit transaction");
                    } else {
                        log::error!("Failed to parse sms, invalid timestamp: {}", timestamp);
                    }
                } else {
                    log::info!("Skipping message with incompatible state: {:?}", state);
                }
            }

            Result::Ok(())
        }
    });

    let listener_task = tokio::spawn(async move {
        let mut listener = PgListener::connect_with(&pool)
            .await
            .expect("Failed to connect to postgres events");

        listener
            .listen("sent")
            .await
            .expect("Failed to start listening on 'sent' channel");

        while let Some(_) = fetch_and_process(&pool, connection.clone()).await {
            log::info!("Processed stale message");
        }

        loop {
            match listener.recv().await {
                Ok(event) => {
                    log::info!("Received notification for outgoing message: {:?}", event);
                    while let Some(_) = fetch_and_process(&pool, connection.clone()).await {
                        log::info!("Processed message");
                    }
                    log::info!("Done processing, waiting for next notification");
                }
                Err(e) => {
                    log::warn!("Failure while listening for events: {}", e);
                }
            }
        }
    });

    tokio::try_join!(listener_task, modem_task).expect("Something failed, stopping");

    Ok(())
}
