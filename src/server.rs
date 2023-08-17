use lib::config::settings::Env;
use lib::config::settings::Settings;
use lib::repos::db::db_connect;
use lib::repos::db::AppState;
use lib::repos::migration::migrate;
use lib::routes::api::router;
use std::net::SocketAddr;

use clutch_wallet_lib::utils::wallet_lib::*;

fn clutch_wallet(setting: &Settings) -> WalletLib {
    let wallet_lib = WalletLib::new(
        "http://localhost:8545",
        "http://localhost:3000/rpc",
        "0x6eca9bac37ba92908805c68c2de7106dd15fde28",
        "0xc4b4f2df5a4936aeda4df93ec203d6c6100bdb7f",
        "0x9cef0d6889154f56fc266c9e54250cbc5c0c9bfe",
        "0x861adf70d644dfe2038775f648d2509190ee7579",
        "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
        "0x240c9cebe72a7f3010b40b5ef166be1ed56ddf44",
        1337,
    );

    // let wallet_lib = WalletLib::new(
    //     &setting.rpc(),
    //     "http://localhost:3000/rpc",
    //     "0x2a83dbe5f2100d196486baa58ad740030dad653a",
    //     "0xc4b4f2df5a4936aeda4df93ec203d6c6100bdb7f",
    //     "0x9cef0d6889154f56fc266c9e54250cbc5c0c9bfe",
    //     "0x5748f0a6a5d251e0f511470af60fec8a55291217",
    //     "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
    //     "0x370f9c8e06c2a6ae986fc050d36d0c6a0475bb99",
    //     setting.chain_id(),
    // );
    wallet_lib
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let settings = &Settings::new(Env::Dev).unwrap();

    migrate(&settings.database.url);

    let app_state = AppState {
        settings: settings.to_owned(),
        database: db_connect(settings.db_connection_url()).await,
        wallet_lib: clutch_wallet(settings),
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
