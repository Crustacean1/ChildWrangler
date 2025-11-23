use std::{collections::HashMap, env};

use chrono::{DateTime, NaiveDateTime, NaiveTime, Utc};
use dto::messages::MessageData;
use futures::stream::{self, StreamExt};
use simple_logger::SimpleLogger;
use sqlx::PgPool;
use tokio::sync::mpsc;
use uuid::Uuid;
use zbus::proxy;
use zbus::zvariant::NoneValue;
use zbus::{
    Connection, Result,
    fdo::ObjectManagerProxy,
    zvariant::{OwnedObjectPath, Value},
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

    #[zbus(property)]
    fn signal_quality(&self) -> Result<(u32, bool)>;

    #[zbus(property)]
    fn own_numbers(&self) -> Result<Vec<String>>;

    #[zbus(property)]
    fn sim(&self) -> Result<OwnedObjectPath>;

    #[zbus(signal)]
    fn state_signal(&self, old: i32, new: i32, reason: u32) -> Result<()>;
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

enum SmsState {
    Unknown = 0,
    Stored = 1,
    Receiving = 2,
    Received = 3,
    Sending = 4,
    Sent = 5,
}

#[proxy(
    default_service = "org.freedesktop.ModemManager1",
    interface = "org.freedesktop.ModemManager1.Modem.Messaging"
)]
trait Messaging {
    fn list(&self) -> Result<Vec<OwnedObjectPath>>;

    #[zbus(signal)]
    fn added(&self, path: OwnedObjectPath, received: bool) -> Result<()>;

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

enum ModemEvent {
    StateChanged(i32, i32),
    SignalQualityChanged(u32),
    RequiresUnlock(bool),
    MessageEnqueued(Uuid),
    MessageAdded(OwnedObjectPath),
}

async fn handle_modem(
    pool: &PgPool,
    connection: &Connection,
    modem_path: OwnedObjectPath,
    pin: &str,
) -> Result<()> {
    let modem_proxy = ModemProxy::builder(connection)
        .path(modem_path.clone())?
        .build()
        .await?;

    let messaging_proxy = MessagingProxy::builder(connection)
        .path(modem_path.clone())?
        .build()
        .await?;

    let own_numbers = modem_proxy.own_numbers().await?;
    log::info!("Got the following numbers: {:?}", own_numbers);

    let (signal_quality, recent) = modem_proxy.signal_quality().await?;
    let state = modem_proxy.state().await?;

    log::info!(
        "Obtained signal and state info: {:?} {:?}",
        signal_quality,
        state
    );

    sqlx::query!(
            "INSERT INTO phones (number, signal, state) VALUES ($1,$2,$3) ON CONFLICT (number) DO UPDATE SET number = $1, signal = $2",
            "N/A",
            signal_quality as i32,
            state as i32
        ).execute(pool)
        .await
        .expect("Failed to update phone in database");

    log::info!("Updated info in db");

    let (tx, mut rx) = mpsc::unbounded_channel::<ModemEvent>();

    let mut is_locked = modem_proxy.unlock_required().await? == 2;
    tx.send(ModemEvent::RequiresUnlock(is_locked))
        .expect("Failed to send unlock data into channel");

    let mut unlock_stream = modem_proxy.receive_unlock_required_changed().await;
    let unlock_task = tokio::spawn({
        let tx = tx.clone();
        async move {
            while let Some(unlock) = unlock_stream.next().await {
                let unlock = unlock.get().await;
                if let Ok(unlock) = unlock {
                    tx.send(ModemEvent::RequiresUnlock(unlock == 2))
                        .expect("Failed to send unlock info accros channel");
                } else {
                    log::error!("Failed to decode unlock property change");
                }
            }
        }
    });

    let mut state_stream = modem_proxy.receive_state_signal().await?;
    tx.send(ModemEvent::StateChanged(state, state))
        .expect("Failed to send state into channel");

    let state_task = tokio::spawn({
        let tx = tx.clone();
        async move {
            while let Some(state) = state_stream.next().await {
                let args: StateSignalArgs = state
                    .args()
                    .expect("Failed to deserialize state from modem manager");
                tx.send(ModemEvent::StateChanged(args.old, args.new))
                    .expect("Failed to send state change through channel");
            }
        }
    });

    let mut quality_stream = modem_proxy.receive_signal_quality_changed().await;

    let quality_task = tokio::spawn({
        let tx = tx.clone();
        async move {
            while let Some(quality) = quality_stream.next().await {
                let val = quality
                    .get()
                    .await
                    .expect("Failed to parse quality change notification");
                tx.send(ModemEvent::SignalQualityChanged(val.0))
                    .expect("Failed to send state change through channel");
            }
        }
    });

    let mut has_attempted_unlock = false;

    let mut current_state = state;

    let message_list_stream = messaging_proxy.receive_added().await?;
    let mut message_stream = message_list_stream.map(|element| {
        element
            .args()
            .expect("Failed to parse incoming message")
            .path
    });

    let msg_task = tokio::spawn({
        let tx = tx.clone();
        async move {
            while let Some(msg) = message_stream.next().await {
                log::info!("Processing message");
                tx.send(ModemEvent::MessageAdded(msg))
                    .expect("Failed to send new message notification on the channel");
            }
        }
    });

    while let Some(event) = rx.recv().await {
        match event {
            ModemEvent::StateChanged(old, new) => {
                log::info!("Modem state changed from {} to {}", old, new);
                match new {
                    3 => {
                        log::info!("Enabling modem");
                        modem_proxy.enable(true).await?;
                    }
                    7 => {
                        log::info!("Modem ready for work, getting message list");
                        let message_list = messaging_proxy.list().await?;
                        for message in message_list {
                            tx.send(ModemEvent::MessageAdded(message))
                                .expect("Failed to add messages to channel");
                        }
                    }
                    state => {
                        log::info!("Unknown state: {}, ignoring", state);
                    }
                }
                current_state = new;
            }
            ModemEvent::SignalQualityChanged(signal) => {
                log::info!("Updating phone signal info");
                sqlx::query!(
                    "UPDATE phones SET signal = $2 WHERE number = $1 ",
                    "N/A",
                    signal as i32
                )
                .execute(pool)
                .await
                .expect("Failed to update signal quality in db");
            }
            ModemEvent::RequiresUnlock(locked) => {
                if current_state == 2 {
                    if locked {
                        if has_attempted_unlock {
                            log::warn!(
                                "Phone status changed to locked, even though unlock took place, quitting"
                            );
                            panic!();
                        } else {
                            has_attempted_unlock = true;
                        }

                        let sim_path = modem_proxy.sim().await?;
                        let sim = SimProxy::builder(connection)
                            .path(sim_path)?
                            .build()
                            .await?;
                        sim.send_pin("1631".into()).await?;

                        log::info!("Sent pind to unlock phone");
                    } else {
                        if is_locked {
                            log::info!("Phone unlocked");
                        }
                    }
                }
                is_locked = locked;
            }
            ModemEvent::MessageEnqueued(id) => {
                let mut tr = pool.begin().await.expect("Failed to start transaction");

                let message = sqlx::query!("SELECT * FROM messages WHERE NOT processed AND outgoing AND id = $1 FOR UPDATE SKIP LOCKED", id).fetch_optional(&mut *tr).await.expect("Failed to fetch message from db");

                if let Some(message) = message {
                    let msg = [
                        ("number", Value::Str(message.phone.into())),
                        ("text", Value::Str(message.content.into())),
                    ]
                    .into_iter()
                    .collect();

                    let sms_path = messaging_proxy.create(msg).await?;
                    let sms = SmsProxy::builder(connection)
                        .path(sms_path)?
                        .build()
                        .await?;

                    sms.send().await?;

                    sqlx::query!("UPDATE messages SET processed = true WHERE id = $1", id)
                        .execute(pool)
                        .await
                        .expect("Failed to mark message as processed");

                    tokio::spawn({
                        let pool = pool.clone();
                        async move {
                            let mut state_stream = sms.receive_state_changed().await;
                            while let Some(state) = state_stream.next().await {
                                let state = state.get().await;
                                if state == Ok(5) {
                                    sqlx::query!(
                                        "UPDATE messages SET sent = $2 WHERE id = $1",
                                        id,
                                        Utc::now().naive_local()
                                    )
                                    .execute(&pool)
                                    .await
                                    .expect("Failed to set message delivery time");
                                }
                            }
                        }
                    });
                }

                tr.commit().await.expect("Failed to commit transaction");
            }
            ModemEvent::MessageAdded(path) => {
                on_message_added(connection, pool, path)
                    .await
                    .expect("Failed to process added message");
            }
        }
    }
    Ok(())
}

pub async fn on_message_added(
    connection: &Connection,
    pool: &PgPool,
    sms_path: OwnedObjectPath,
) -> Result<()> {
    let sms_proxy = SmsProxy::builder(connection)
        .path(sms_path)?
        .build()
        .await?;

    let message_added = process_added_message(pool, &sms_proxy).await?;

    if !message_added {
        let mut state_stream = sms_proxy.receive_state_changed().await;
        tokio::spawn({
            let pool = pool.clone();
            async move {
                while let Some(_) = state_stream.next().await {
                    if let Ok(result) = process_added_message(&pool, &sms_proxy).await
                        && result
                    {
                        break;
                    }
                }
            }
        });
    }

    Ok(())
}

async fn process_added_message<'a>(pool: &PgPool, sms_proxy: &SmsProxy<'a>) -> Result<bool> {
    let content = sms_proxy.text().await?;
    let phone = sms_proxy.number().await?;
    let state = sms_proxy.state().await?;
    let sent = sms_proxy.timestamp().await?;
    let sent = DateTime::parse_from_str(&sent, "%Y-%m-%dT%H:%M:%S%#z")
        .expect("Failed to parse timestamp")
        .naive_local();

    log::info!("Streaming state: {}", state);

    if state == 3 {
        let db_message = sqlx::query!(
            "SELECT * FROM messages WHERE sent = $1 AND phone = $2 AND content = $3",
            sent,
            phone,
            content
        )
        .fetch_optional(pool)
        .await
        .expect("Failed to select message");

        if let Some(duplicate) = db_message {
            log::info!(
                "Message {} is duplicate of {}, ignoring",
                content,
                duplicate.id
            );
        } else {
            sqlx::query!(
                "INSERT INTO messages (phone, content, sent) VALUES ($1,$2,$3)",
                phone,
                content,
                sent
            )
            .execute(pool)
            .await
            .expect("Failed to insert message");
        }
        Ok(true)
    } else if state == 5 {
        log::info!(
            "Message is recognized as already sent to: '{}' content: '{}'",
            phone,
            content
        );
        Ok(true)
    } else {
        log::info!("Unknown sms state: {:?}", state);
        Ok(false)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    SimpleLogger::new().init().unwrap();
    let connection = Connection::system().await?;
    log::info!("Connected to system bus");

    let modem_manager_interface = "org.freedesktop.ModemManager1";
    let db_url = std::env::var("DATABASE_URL").expect("No 'DATABASE_URL' specified");
    let pin = std::env::var("MODEM_PIN").expect("No 'MODEM_PIN' specified");

    let object_manager = ObjectManagerProxy::new(
        &connection,
        modem_manager_interface,
        "/org/freedesktop/ModemManager1",
    )
    .await?;

    let pool = PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to postgres database");

    log::info!("Connected to postgres db with url: {}", db_url);

    let tree = object_manager.get_managed_objects().await?;

    let modem_path = tree
        .keys()
        .find(|m| m.contains("/org/freedesktop/ModemManager1/Modem"))
        .expect("Modem object not exposed by ModemManager, verify that service is enabled and running and modem is plugged in").clone();

    handle_modem(&pool, &connection, modem_path, &pin)
        .await
        .expect("Modem handling failed");

    Ok(())
}
