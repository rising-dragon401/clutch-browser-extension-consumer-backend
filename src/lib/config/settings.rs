use std::{env, path::Path};

use config::{Config, ConfigError, Environment, File};
use once_cell::sync::Lazy;
use securestore::{KeySource, SecretsManager};
use serde::Deserialize;

pub static SETTINGS: Lazy<Settings> = Lazy::new(|| Settings::new(Env::Test).unwrap());

static SECRETS: Lazy<SecretsManager> = Lazy::new(|| {
    let keyfile = Path::new(&SETTINGS.secrets.key);
    SecretsManager::load(&SETTINGS.secrets.vault, KeySource::File(keyfile))
        .expect("Failed to load SecureStore vault!")
});

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct Service {
    pub host: String,
    pub port: i32,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct Wallet {
    pub chain_id: u64,
    pub private: String,
    pub rpc: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Email {
    key: String,
    pub base_url: String,
    pub template_id: i64,
    pub send_limit: i64,
}

impl Email {
    pub fn key(&self) -> String {
        match self.key.clone().as_str() {
            "secret" => SECRETS.get("email:key").unwrap(),
            key => key.to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct Database {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct Secrets {
    pub vault: String,
    pub key: String,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct Jwt {
    key: String,
}

impl Jwt {
    pub fn key(&'_ self) -> String {
        match self.key.clone().as_str() {
            "secret" => SECRETS.get("jwt:key").unwrap(),
            key => key.to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub service: Service,
    pub email: Email,
    pub database: Database,
    pub jwt: Jwt,
    pub secrets: Secrets,
    pub wallet: Wallet,
}

impl Settings {
    pub fn host_and_port(&'_ self) -> String {
        format!("{}:{}", &self.service.host, &self.service.port.to_string())
    }

    pub fn db_connection_url(&'_ self) -> String {
        format!("sqlite://{}", &self.database.url)
    }

    pub fn wallet_private_key(&'_ self) -> String {
        format!("{}", self.wallet.private)
    }

    pub fn chain_id(&'_ self) -> u64 {
        self.wallet.chain_id
    }

    pub fn rpc(&'_ self) -> String {
        self.wallet.rpc.clone()
    }
}

pub enum Env {
    Test,
    Dev,
    Prod,
}

impl Settings {
    pub fn new(fallback_env: Env) -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| match fallback_env {
            Env::Test => "test".into(),
            Env::Dev => "dev".into(),
            Env::Prod => "prod".into(),
        });

        let s = Config::builder()
            .add_source(File::with_name(&format!("config/{}", run_mode)))
            // Add in a local configuration file
            // This file shouldn't be checked in to git
            .add_source(File::with_name("config/local").required(false))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::with_prefix("app"))
            .build()?;

        // Now that we're done, let's access our configuration
        // println!("debug: {:?}", s.get_bool("debug"));
        // println!("database: {:?}", s.get::<String>("database.url"));

        // // You can deserialize (and thus freeze) the entire configuration as
        s.try_deserialize()
    }
}
