use sea_orm::entity::prelude::*;
use sea_orm::Database;

use crate::config::settings::Settings;
use clutch_wallet_lib::utils::wallet_lib::WalletLib;

#[derive(Debug, Clone)]
pub struct AppState {
    pub settings: Settings,
    pub database: DatabaseConnection,
    pub wallet_lib: WalletLib,
}

pub async fn db_connect(connection_url: String) -> DatabaseConnection {
    Database::connect(connection_url)
        .await
        .expect("failed to connect to database")
}
