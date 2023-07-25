use crate::{
    models::api::{
        api_error, api_success, AccountGuardian, AccountGuardianDeleteResponse,
        AccountGuardianParams, ApiErrorResponse, ApiResponse, ListAccountGuardiansResponse,
    },
    operations::jwt::{decode_jwt, validate_jwt_claims},
    repos::{
        account_repo,
        db::AppState,
        guardian_account_repo::{self, Model},
        guardian_repo,
    },
};
use axum::{
    extract::{Path, Query, State},
    routing::{delete, get},
    Json, Router,
};
use axum_auth::AuthBearer;
use hyper::StatusCode;
use sea_orm::DatabaseConnection;

use super::guardian_settings_api;

pub fn routes<S>(app_state: &AppState) -> Router<S> {
    Router::new()
        .route("/", get(get_guardians))
        .route("/:guardian_id", delete(remove_account_guardian))
        .nest("/settings", guardian_settings_api::routes(app_state))
        .with_state(app_state.to_owned())
}

async fn remove_account_guardian(
    app_state: State<AppState>,
    AuthBearer(token): AuthBearer,
    Path(guardian_id): Path<String>,
) -> Result<Json<ApiResponse<AccountGuardianDeleteResponse, ApiErrorResponse>>, StatusCode> {
    match try_delete_guardian(app_state, token, guardian_id).await {
        Ok(payload) => Ok(Json(api_success(payload))),
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn try_delete_guardian(
    app_state: State<AppState>,
    token: String,
    guardian_id: String,
) -> anyhow::Result<AccountGuardianDeleteResponse> {
    let claims = decode_jwt(token).await?;
    validate_jwt_claims(claims.clone()).await?;
    let account = account_repo::find_by_id(&app_state.database, claims.sub.clone()).await?;
    match account {
        Some(acc) => {
            guardian_repo::find_by_id(&app_state.database, guardian_id.clone()).await?;
            let account_guardian =
                guardian_account_repo::find_guardian_by_guardian_id_and_account_id(
                    &app_state.database,
                    guardian_id.clone(),
                    acc.id,
                )
                .await?;
            match account_guardian {
                Some(ag) => {
                    if ag.status == "AVAILABLE" {
                        guardian_account_repo::delete_by_id(&app_state.database, ag.id).await?;
                        Ok(AccountGuardianDeleteResponse {
                            guardian_id: guardian_id.clone(),
                        })
                    } else {
                        Err(anyhow::anyhow!("Guardian must not be ACTIVE"))
                    }
                }
                None => Err(anyhow::anyhow!("Guardian not found for account")),
            }
        }
        None => Err(anyhow::anyhow!("Account not found")),
    }
}

async fn get_guardians(
    app_state: State<AppState>,
    AuthBearer(token): AuthBearer,
    Query(params): Query<AccountGuardianParams>,
) -> Result<Json<ApiResponse<ListAccountGuardiansResponse, ApiErrorResponse>>, StatusCode> {
    match try_get_guardians(app_state, token, &params).await {
        Ok(payload) => Ok(Json(api_success(payload))),
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn try_get_guardians(
    app_state: State<AppState>,
    token: String,
    params: &AccountGuardianParams,
) -> anyhow::Result<ListAccountGuardiansResponse> {
    let claims = decode_jwt(token).await?;
    validate_jwt_claims(claims.clone()).await?;
    let account = account_repo::find_by_id(&app_state.database, claims.sub.clone()).await?;
    match account {
        Some(acc) => match &params.guardian_id {
            Some(guardian_id) => {
                find_all_account_guardians_by_guardian_id(
                    &app_state,
                    acc.id,
                    guardian_id.to_owned(),
                )
                .await
            }
            None => match &params.status {
                Some(status) => {
                    find_all_account_guardians_by_status(&app_state, acc.id, status.to_owned())
                        .await
                }
                None => find_all_account_guardians(&app_state, acc.id).await,
            },
        },

        None => Err(anyhow::anyhow!("Account not found")),
    }
}

async fn find_all_account_guardians_by_guardian_id(
    app_state: &AppState,
    account_id: String,
    guardian_id: String,
) -> anyhow::Result<ListAccountGuardiansResponse> {
    let account_guardians = guardian_account_repo::find_all_accounts_by_guardian_id_and_account_id(
        &app_state.database,
        guardian_id,
        account_id,
    )
    .await?;
    let guardians = to_account_guardians(&app_state.database, account_guardians).await?;
    Ok(ListAccountGuardiansResponse { guardians })
}

async fn find_all_account_guardians_by_status(
    app_state: &AppState,
    account_id: String,
    status: String,
) -> anyhow::Result<ListAccountGuardiansResponse> {
    let account_guardians = guardian_account_repo::find_all_guardians_by_account_id_and_status(
        &app_state.database,
        account_id,
        status.to_uppercase(),
    )
    .await?;
    let guardians = to_account_guardians(&app_state.database, account_guardians).await?;
    Ok(ListAccountGuardiansResponse { guardians })
}

async fn find_all_account_guardians(
    app_state: &AppState,
    account_id: String,
) -> anyhow::Result<ListAccountGuardiansResponse> {
    let account_guardians =
        guardian_account_repo::find_all_guardians_by_account_id(&app_state.database, account_id)
            .await?;
    let guardians = to_account_guardians(&app_state.database, account_guardians).await?;
    Ok(ListAccountGuardiansResponse { guardians })
}

pub async fn to_account_guardians(
    db: &DatabaseConnection,
    account_guardians: Vec<Model>,
) -> anyhow::Result<Vec<AccountGuardian>> {
    let guardian_ids = account_guardians
        .iter()
        .map(|ag| ag.guardian_id.to_string())
        .collect::<Vec<String>>();
    let guardian_list = guardian_repo::find_all_by_ids(db, guardian_ids).await?;
    Ok(account_guardians
        .iter()
        .map(|ag| AccountGuardian {
            id: ag.id.to_string(),
            email: guardian_list
                .iter()
                .find(|g| g.id == ag.guardian_id)
                .map(|g| g.email.clone())
                .unwrap_or_else(|| "".to_string()),
            wallet_address: guardian_list
                .iter()
                .find(|g| g.id == ag.guardian_id)
                .map(|g| g.email.clone())
                .unwrap_or_else(|| "".to_string()),
            status: ag.status.clone(),
        })
        .collect())
}
