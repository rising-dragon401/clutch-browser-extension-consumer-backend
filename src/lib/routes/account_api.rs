use super::{account_guardians_api, nomination_api};
use crate::{
    models::api::{
        api_error, api_success, Account, AccountCreateRequest, AccountCreateResponse,
        AccountParams, AccountUpdateRequest, AccountUpdateResponse, ApiErrorResponse, ApiResponse,
        ListAccountsResponse,
    },
    operations::{
        jwt::{decode_jwt, generate_jwt, validate_jwt_claims},
        time::get_unix_timestamp_ms,
    },
    repos::{account_repo, db::AppState, verification_repo},
};
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use axum_auth::AuthBearer;
use email_address::EmailAddress;
use ethers::{
    abi::{self, Token},
    prelude::*,
    providers::Provider,
    types::{Address, U256},
};
use hyper::StatusCode;
use sea_orm::DatabaseConnection;
use std::str::FromStr;
use uuid::Uuid;

pub fn routes<S>(app_state: &AppState) -> Router<S> {
    Router::new()
        .route(
            "/",
            get(get_accounts).post(create_account).put(update_account),
        )
        .nest("/nominations", nomination_api::routes(app_state))
        .nest("/guardians", account_guardians_api::routes(app_state))
        .with_state(app_state.to_owned())
}

async fn update_account(
    app_state: State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(req): Json<AccountUpdateRequest>,
) -> Result<Json<ApiResponse<AccountUpdateResponse, ApiErrorResponse>>, StatusCode> {
    match try_update_account(&app_state, token, &req).await {
        Ok(payload) => Ok(Json(api_success(payload))),
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn get_accounts(
    app_state: State<AppState>,
    Query(params): Query<AccountParams>,
) -> Result<Json<ApiResponse<ListAccountsResponse, ApiErrorResponse>>, StatusCode> {
    match params.wallet_address {
        Some(wallet_address) => find_accounts_by_wallet_address(&app_state, wallet_address).await,
        None => match params.eoa_address {
            Some(eoa_address) => find_accounts_by_eoa_address(&app_state, eoa_address).await,
            None => match params.email {
                Some(email) => find_accounts_by_email_address(&app_state, email).await,
                None => find_all_accounts(&app_state).await,
            },
        },
    }
}

async fn find_accounts_by_email_address(
    app_state: &State<AppState>,
    email: String,
) -> Result<Json<ApiResponse<ListAccountsResponse, ApiErrorResponse>>, StatusCode> {
    match account_repo::find_all_by_email_address(&app_state.database, email).await {
        Ok(payload) => {
            let accounts = payload
                .iter()
                .map(|account| Account {
                    id: account.id.to_string(),
                    email: account.email.clone(),
                    wallet_address: account.wallet_address.clone(),
                    eoa_address: account.eoa_address.clone(),
                    updated_at: account.updated_at,
                })
                .collect();
            Ok(Json(api_success(ListAccountsResponse { accounts })))
        }
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn find_accounts_by_eoa_address(
    app_state: &State<AppState>,
    eoa_address: String,
) -> Result<Json<ApiResponse<ListAccountsResponse, ApiErrorResponse>>, StatusCode> {
    match account_repo::find_all_by_eoa_address(&app_state.database, eoa_address).await {
        Ok(payload) => {
            let accounts = payload
                .iter()
                .map(|account| Account {
                    id: account.id.to_string(),
                    email: account.email.clone(),
                    wallet_address: account.wallet_address.clone(),
                    eoa_address: account.eoa_address.clone(),
                    updated_at: account.updated_at,
                })
                .collect();
            Ok(Json(api_success(ListAccountsResponse { accounts })))
        }
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn find_accounts_by_wallet_address(
    app_state: &State<AppState>,
    wallet_address: String,
) -> Result<Json<ApiResponse<ListAccountsResponse, ApiErrorResponse>>, StatusCode> {
    match account_repo::find_all_by_wallet_address(&app_state.database, wallet_address).await {
        Ok(payload) => {
            let accounts = payload
                .iter()
                .map(|account| Account {
                    id: account.id.to_string(),
                    email: account.email.clone(),
                    wallet_address: account.wallet_address.clone(),
                    eoa_address: account.eoa_address.clone(),
                    updated_at: account.updated_at,
                })
                .collect();
            Ok(Json(api_success(ListAccountsResponse { accounts })))
        }
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn find_all_accounts(
    app_state: &State<AppState>,
) -> Result<Json<ApiResponse<ListAccountsResponse, ApiErrorResponse>>, StatusCode> {
    match account_repo::find_all(&app_state.database).await {
        Ok(payload) => {
            let accounts = payload
                .iter()
                .map(|account| Account {
                    id: account.id.to_string(),
                    email: account.email.clone(),
                    wallet_address: account.wallet_address.clone(),
                    eoa_address: account.eoa_address.clone(),
                    updated_at: account.updated_at,
                })
                .collect();
            Ok(Json(api_success(ListAccountsResponse { accounts })))
        }
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn create_account(
    app_state: State<AppState>,
    Json(req): Json<AccountCreateRequest>,
) -> Result<Json<ApiResponse<AccountCreateResponse, ApiErrorResponse>>, StatusCode> {
    match try_create_account(&app_state, &req).await {
        Ok(payload) => Ok(Json(api_success(payload))),
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn try_create_account(
    app_state: &State<AppState>,
    req: &AccountCreateRequest,
) -> anyhow::Result<AccountCreateResponse> {
    if EmailAddress::is_valid(&req.email) {
        let account = account_repo::find_by_email(&app_state.database, &req.email).await?;
        match account {
            Some(_) => Err(anyhow::anyhow!(
                "Error account already exists for email: {}",
                req.email
            )),
            None => {
                // validate_code(&app_state.database, req.email.clone(), req.code.clone()).await?;
                // let app_state_data = app_state.0.clone();

                let account_id = Uuid::new_v4();
                store_account(
                    app_state,
                    req,
                    account_id,
                    "abcdef".to_string(),
                    "defa".to_string(),
                    "aaaaaa".to_string(),
                )
                .await?;
                let jwt = generate_jwt(account_id.to_string()).await?;
                Ok(AccountCreateResponse { jwt })
            }
        }
    } else {
        Err(anyhow::anyhow!("Invalid email format {}", req.email))
    }
}

async fn try_update_account(
    app_state: &State<AppState>,
    token: String,
    req: &AccountUpdateRequest,
) -> anyhow::Result<AccountUpdateResponse> {
    if EmailAddress::is_valid(&req.email) {
        let claims = decode_jwt(token).await?;
        validate_jwt_claims(claims.clone()).await?;

        let account = account_repo::find_by_id(&app_state.database, claims.sub).await?;
        match account {
            Some(_) => {
                account_repo::update(
                    &app_state.database,
                    req.email.clone(),
                    req.wallet_address.clone(),
                    req.eoa_address.clone(),
                )
                .await?;
                let updated =
                    req.wallet_address.clone().is_some() || req.eoa_address.clone().is_some();
                Ok(AccountUpdateResponse { updated })
            }
            None => Err(anyhow::anyhow!(
                "Error no account found for email: {}",
                req.email
            )),
        }
    } else {
        Err(anyhow::anyhow!("Invalid email format {}", req.email))
    }
}

async fn store_account(
    app_state: &State<AppState>,
    req: &AccountCreateRequest,
    id: Uuid,
    wallet: String,
    eoa: String,
    eoa_private: String,
) -> Result<(), anyhow::Error> {
    let updated_at = get_unix_timestamp_ms();
    account_repo::create(
        &app_state.database,
        id,
        req.email.clone(),
        wallet,
        eoa,
        eoa_private,
        updated_at,
    )
    .await
    .map_err(|e| {
        anyhow::anyhow!(format!(
            "Error creating account for email: {} with error: {}",
            req.email, e
        ))
    })?;
    Ok(())
}

pub async fn validate_code(
    db: &DatabaseConnection,
    email: String,
    code: String,
) -> anyhow::Result<()> {
    let expiry = verification_repo::find_by_email_and_code(db, &email, &code)
        .await?
        .map(|v| v.expires_at)
        .ok_or_else(|| {
            anyhow::anyhow!(format!(
                "Verification not found with email {} and code {}",
                email, code
            ))
        })?;
    if get_unix_timestamp_ms() > expiry {
        Err(anyhow::anyhow!(format!(
            "Verification code expired for email {} and code {}",
            email, code
        )))
    } else {
        Ok(())
    }
}
