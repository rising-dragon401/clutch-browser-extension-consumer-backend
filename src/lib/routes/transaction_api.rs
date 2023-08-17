use crate::repos::db::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};

use super::{nomination_api, account_guardians_api};


pub fn routes<S>(app_state: &AppState) -> Router<S> {
    // Router::new()
    //     .route(
    //         "/",
    //         get(get_accounts).post(create_account).put(update_account),
    //     )        
    //     .with_state(app_state.to_owned())
    unimplemented!()
}