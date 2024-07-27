use log::{debug, info};
use rhiaqey_sdk_rs::producer::{
    Producer, ProducerConfig, ProducerMessage, ProducerMessageReceiver,
};
use rhiaqey_sdk_rs::settings::Settings;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex;

#[derive(Deserialize, Clone, Debug)]
enum Port {
    SSL = 5211,
    PLAIN = 5201,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CTraderSettings {
    hostname: String,
    port: Port,
    password: String,
    sender_comp_id: String,
    target_comp_id: String,
    sender_sub_id: String,
}

impl Default for CTraderSettings {
    fn default() -> Self {
        CTraderSettings {
            hostname: String::from(""),
            port: Port::PLAIN,
            password: String::from(""),
            sender_comp_id: String::from(""),
            target_comp_id: String::from(""),
            sender_sub_id: String::from(""),
        }
    }
}

impl Settings for CTraderSettings {
    //
}

#[derive(Debug)]
pub struct CTrader {
    settings: Arc<Mutex<CTraderSettings>>,
}

impl Default for CTrader {
    fn default() -> Self {
        Self {
            settings: Arc::new(Default::default()),
        }
    }
}

impl Producer<CTraderSettings> for CTrader {
    async fn setup(
        &mut self,
        _config: ProducerConfig,
        settings: Option<CTraderSettings>,
    ) -> ProducerMessageReceiver {
        info!("setting up ctrader producer");

        if let Some(cfg) = settings {
            debug!("setting found");
            self.set_settings(cfg).await;
        }

        let (_sender, receiver) = unbounded_channel::<ProducerMessage>();

        Ok(receiver)
    }

    async fn set_settings(&mut self, settings: CTraderSettings) {
        let mut locked_settings = self.settings.lock().await;
        *locked_settings = settings;
        debug!("new settings updated");
    }

    async fn start(&mut self) {
        todo!()
    }

    fn schema() -> Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {},
            "required": [],
            "additionalProperties": false
        })
    }

    fn kind() -> String {
        String::from("ctrader")
    }
}

/*
// use futures::TryFutureExt;
use log::debug;
// use quickfix::dictionary_item::{EndTime, StartTime};
use quickfix::{
    Application, ApplicationCallback, ConnectionHandler, Dictionary, FileMessageStoreFactory,
    LogFactory, SessionId, SessionSettings, SocketAcceptor, StdLogger,
};
use rhiaqey_sdk_rs::producer::{Producer, ProducerMessage, ProducerMessageReceiver};
use rhiaqey_sdk_rs::settings::Settings;
use serde::Deserialize;
use serde_json::{json, Value};
use std::error::Error;
// use std::io::{stdin, Read};
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex;

#[derive(Deserialize, Clone, Debug)]
enum Port {
    SSL = 5211,
    PLAIN = 5201,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CTraderSettings {
    hostname: String,
    port: Port,
    password: String,
    sender_comp_id: String,
    target_comp_id: String,
    sender_sub_id: String,
}

/**

Host name: demo-uk-eqx-01.p.ctrader.com
(Current IP address 99.83.135.211 can be changed without notice)
Port: 5211 (SSL), 5201 (Plain text).
Password: (a/c 4363372 password)
SenderCompID: demo.ctrader.4363372
TargetCompID: cServer
SenderSubID: QUOTE

Host name: demo-uk-eqx-01.p.ctrader.com
(Current IP address 99.83.135.211 can be changed without notice)
Port: 5211 (SSL), 5201 (Plain text).
Password: (a/c 4363372 password)
SenderCompID: demo.ctrader.4363372
TargetCompID: cServer
SenderSubID: QUOTE*/
impl Default for CTraderSettings {
    fn default() -> Self {
        CTraderSettings {
            hostname: String::from("demo-uk-eqx-01.p.ctrader.com"),
            port: Port::PLAIN,
            password: String::from("welcome"),
            sender_comp_id: String::from("demo.ctrader.4363372"),
            target_comp_id: String::from("cServer"),
            sender_sub_id: String::from("QUOTE"),
        }
    }
}

impl Settings for CTraderSettings {
    //
}

#[derive(Debug)]
pub struct CTrader {
    settings: Arc<Mutex<CTraderSettings>>,
    file_message_store: FileMessageStoreFactory,
    /*
    socket_acceptor: Option<
        Arc<Mutex<SocketAcceptor<'static, MyApplication, StdLogger, FileMessageStoreFactory>>>,
    >,*/
}

#[derive(Default)]
pub struct MyApplication;

impl ApplicationCallback for MyApplication {
    // Implement whatever callback you need

    fn on_create(&self, _session: &SessionId) {
        // Do whatever you want here 😁
    }
}

impl Producer<CTraderSettings> for CTrader {
    fn create() -> Result<Self, Box<dyn Error>> {
        let _log_factory = LogFactory::try_new(&StdLogger::Stdout)?;
        let _app = Application::try_new(&MyApplication)?;
        let settings = SessionSettings::new();
        let file_message_store = FileMessageStoreFactory::try_new(&settings)?;

        Ok(Self {
            file_message_store,
            settings: Default::default(),
        })
    }

    fn setup(&mut self, _settings: Option<CTraderSettings>) -> ProducerMessageReceiver {
        /*
        if settings.is_none() {
            return Err(Box::from("Settings are not available"));
        }

        let setup_settings = settings.unwrap();

        let session_id = SessionId::try_new(
            "FIX.4.4",
            setup_settings.sender_comp_id.as_str(),
            setup_settings.target_comp_id.as_str(),
            "",
        )?;

        let mut settings = SessionSettings::new();
        settings.set(
            Some(&session_id),
            Dictionary::try_from_items(&[&StartTime("00:00:01"), &EndTime("23:59:59")])?,
        )?;

        // self.socket_acceptor = Some(Arc::new(Mutex::new(acceptor)));
        */
        let (_sender, receiver) = unbounded_channel::<ProducerMessage>();

        Ok(receiver)
    }

    async fn set_settings(&mut self, settings: CTraderSettings) {
        debug!("setting settings {:?}", settings);
    }

    async fn start(&mut self) {
        /*
        if let Some(acceptor) = self.socket_acceptor.clone() {
            let mut lock = acceptor.lock().await;
            info!("found acceptor");
            lock.start().unwrap();
            info!("acceptor started");

            let mut stdin = stdin().lock();
            let mut stdin_buf = [0];
            loop {
                let _ = stdin.read_exact(&mut stdin_buf);
                if stdin_buf[0] == b'q' {
                    break;
                }
            }
        }*/
        tokio::task::spawn(async move { loop {} });
    }

    fn schema() -> serde_json::value::Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {},
            "required": [],
            "additionalProperties": false
        })
    }

    async fn metrics(&self) -> Value {
        json!({})
    }

    fn kind() -> String {
        String::from("ctrader")
    }
}
*/
