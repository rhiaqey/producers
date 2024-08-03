use log::{debug, info};
use std::any::{Any, TypeId};

use quickfix::dictionary_item::{
    ConnectionType, DataDictionary, EndTime, FileStorePath, HeartBtInt, ReconnectInterval,
    SocketAcceptPort, SocketConnectHost, SocketConnectPort, StartTime,
};
use quickfix::{
    Application, ApplicationCallback, ConnectionHandler, Dictionary, FieldMap,
    FileMessageStoreFactory, LogFactory, Message, MsgFromAdminError, MsgFromAppError,
    MsgToAppError, QuickFixError, SessionId, SessionSettings, SocketInitiator, StdLogger,
};

// this is valid
use quickfix_msg44::field_id::PASSWORD;
use quickfix_msg44::field_id::USERNAME;
use quickfix_msg44::field_types::MsgType;

use rhiaqey_sdk_rs::producer::{
    Producer, ProducerConfig, ProducerMessage, ProducerMessageReceiver,
};
use rhiaqey_sdk_rs::settings::Settings;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
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

impl ApplicationCallback for CTrader {
    // Implement whatever callback you need

    fn on_create(&self, session: &SessionId) {
        // Do whatever you want here.
        info!("-------------------- created {:?}", session);
    }

    fn on_logon(&self, session: &SessionId) {
        info!("-------------------- on login {:?}", session);
    }

    fn on_logout(&self, session: &SessionId) {
        info!("-------------------- on logout {:?}", session);
    }

    // https://github.com/arthurlm/quickfix-rs/blob/main/examples/coinbase-fix-utils/src/logon_utils.rs#L13
    fn on_msg_to_admin(&self, msg: &mut Message, session: &SessionId) {
        // Intercept a login message automatically sent by quickfix library
        let msg_type = msg
            .with_header(|h| h.get_field(35))
            .and_then(|x| MsgType::from_const_bytes(x.as_bytes()).ok());

        if msg_type == Some(MsgType::Logon) {
            info!("????????????????????????????????????????????????????????");

            let username = "4363372";
            let password = "07650765";

            msg.set_field(USERNAME, username)
                .expect("failed to set username");
            msg.set_field(PASSWORD, password)
                .expect("failed to set password");

            info!("=======================================================");
        }

        // if let Some(msg_type) = msg_type {
        // if MsgType::Logon == msg_type {
        //
        // }
        // }

        /*
        // Set password
        msg.set_field(USERNAME, "s.nakamoto")
            .expect("Fail to set password");
        msg.set_field(PASSWORD, config.api_passphrase.as_str())
            .expect("Fail to set password");
        */

        info!(
            "-------------------- message to admin {:?} - {:?} - {:?}",
            msg, session, msg_type
        );
    }

    fn on_msg_to_app(&self, msg: &mut Message, session: &SessionId) -> Result<(), MsgToAppError> {
        info!(
            "-------------------- message to app {:?} - {:?}",
            msg, session
        );
        Ok(())
    }

    fn on_msg_from_admin(
        &self,
        msg: &Message,
        session: &SessionId,
    ) -> Result<(), MsgFromAdminError> {
        info!(
            "-------------------- message from admin {:?} - {:?}",
            msg, session
        );
        Ok(())
    }

    fn on_msg_from_app(&self, msg: &Message, session: &SessionId) -> Result<(), MsgFromAppError> {
        info!(
            "-------------------- message from app {:?} - {:?}",
            msg, session
        );
        Ok(())
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
        let host = "demo-uk-eqx-01.p.ctrader.com";
        let port = 5201; // 5211 ssl
        let target_comp_id = "cServer";
        let sender_comp_id = "demo.ctrader.4363372";

        let initiator_session =
            SessionId::try_new("FIX.4.4", sender_comp_id, target_comp_id, "").unwrap();

        let acceptor_session =
            SessionId::try_new("FIX.4.4", target_comp_id, sender_comp_id, "").unwrap();

        let mut settings = SessionSettings::new();

        settings
            .set(
                None,
                Dictionary::try_from_items(&[&ConnectionType::Initiator]).unwrap(),
            )
            .unwrap();

        settings
            .set(
                Some(&initiator_session),
                Dictionary::try_from_items(&[
                    &ConnectionType::Initiator,
                    &StartTime("00:00:00"),
                    &EndTime("23:59:59"),
                    &SocketConnectHost(host),
                    &SocketConnectPort(port),
                    &HeartBtInt(20),
                    &ReconnectInterval(60),
                    &FileStorePath("store"),
                    &DataDictionary("fix44.xml"),
                ])
                .unwrap(),
            )
            .unwrap();

        settings
            .set(
                Some(&acceptor_session),
                Dictionary::try_from_items(&[
                    &ConnectionType::Acceptor,
                    &StartTime("00:00:00"),
                    &EndTime("23:59:59"),
                    &SocketAcceptPort(port),
                    &HeartBtInt(20),
                    &ReconnectInterval(60),
                    &FileStorePath("store"),
                    &DataDictionary("fix44.xml"),
                ])
                .unwrap(),
            )
            .unwrap();

        let store_factory = FileMessageStoreFactory::try_new(&settings).unwrap();
        let log_factory = LogFactory::try_new(&StdLogger::Stdout).unwrap();
        let app = Application::try_new(self).unwrap();
        let mut socket =
            SocketInitiator::try_new(&settings, &app, &store_factory, &log_factory).unwrap();

        socket.start().unwrap();

        while !socket.is_logged_on().unwrap() {
            // tokio::time::sleep(Duration::from_secs(250)).await;
            // thread::sleep(250.into());
        }

        info!("started...");
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

fn server_loop<C: ConnectionHandler>(mut connection_handler: C) -> Result<(), QuickFixError> {
    info!(">> connection handler START");
    connection_handler.start()
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
