use lib::config::settings::Env;
use lib::config::settings::Settings;
use lib::repos::db::db_connect;
use lib::repos::db::AppState;
use lib::repos::migration::migrate;
use lib::routes::api::router;
use std::net::SocketAddr;

use clutch_wallet_lib::utils::wallet_lib::*;

fn clutch_wallet() -> WalletLib {
    let mut wallet_lib = WalletLib::new(
        "http://localhost:8545",
        "http://localhost:3000/rpc",
        "0x6eca9bac37ba92908805c68c2de7106dd15fde28",
        "0xc4b4f2df5a4936aeda4df93ec203d6c6100bdb7f",
        "0xf16e8831312c0a4b884e49a639083c2ec9cfd4f1",
        "0x861adf70d644dfe2038775f648d2509190ee7579",
        "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
        "0x240c9cebe72a7f3010b40b5ef166be1ed56ddf44",
        1337,
    );
    wallet_lib
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
