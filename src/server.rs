use lib::config::settings::Env;
use lib::config::settings::Settings;
use lib::repos::db::db_connect;
use lib::repos::db::AppState;
use lib::repos::migration::migrate;
use lib::routes::api::router;
use std::net::SocketAddr;

use clutch_wallet_lib::utils::wallet_lib::*;

fn clutch_wallet() -> WalletLib {
    WalletLib::new(
        "http://localhost:8545",
        "http://localhost:3000/rpc",
        "0x721ebda8f508e9de26d0a522d29679df34c7872b",
        "0x9670a43e5e820e920c10d3bb2f018571fedb9b6e",
        "0x6c3a9f19aa9c3c659fbf0ad6721ed48aba48f239",
        "0x9670a43e5e820e920c10d3bb2f018571fedb9b6e",
    )
}

#[tokio::main]
async fn main() {
    // env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let settings = &Settings::new(Env::Test).unwrap();

    migrate(&settings.database.url);

    let app_state = AppState {
        settings: settings.to_owned(),
        database: db_connect(settings.db_connection_url()).await,
        wallet_lib: clutch_wallet(),
    };

    let router = router(app_state);

    let address: SocketAddr = settings
        .host_and_port()
        .parse()
        .expect("Unable to parse socket address");

    log::info!("listening on {}", address);
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("expect tokio signal ctrl-c");
    log::info!("signal shutdown");
}
