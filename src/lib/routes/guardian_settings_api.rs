use crate::{
    models::api::{
        api_error, api_success, AccountGuardianSettingsRequest, AccountGuardianSettingsResponse,
        ApiErrorResponse, ApiResponse, SigningStrategy,
    },
    operations::jwt::{decode_jwt, validate_jwt_claims},
    repos::{account_repo, db::AppState, guardian_account_repo, guardian_settings_repo},
};
use axum::{
    extract::State,
    routing::{get, put},
    Json, Router,
};
use axum_auth::AuthBearer;
use hyper::StatusCode;
use sea_orm::DatabaseConnection;

use super::account_guardians_api::to_account_guardians;

pub fn routes<S>(app_state: &AppState) -> Router<S> {
    Router::new()
        .route("/", get(get_guardian_settings))
        .route("/", put(update_guardian_settings))
        .with_state(app_state.to_owned())
}

async fn update_guardian_settings(
    app_state: State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(req): Json<AccountGuardianSettingsRequest>,
) -> Result<Json<ApiResponse<AccountGuardianSettingsResponse, ApiErrorResponse>>, StatusCode> {
    match try_update_guardian_settings(app_state, token, req).await {
        Ok(payload) => Ok(Json(api_success(payload))),
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn try_update_guardian_settings(
    app_state: State<AppState>,
    token: String,
    req: AccountGuardianSettingsRequest,
) -> anyhow::Result<AccountGuardianSettingsResponse> {
    let claims = decode_jwt(token).await?;
    validate_jwt_claims(claims.clone()).await?;
    let account = account_repo::find_by_id(&app_state.database, claims.sub.clone()).await?;
    match account {
        Some(acc) => {
            validate_guardian_quantity(req.signers.clone(), req.guardians.clone()).await?;
            validate_guardians_for_account(
                &app_state.database,
                req.guardians.clone(),
                acc.id.clone(),
            )
            .await?;

            guardian_settings_repo::update_settings_for_account_id(
                &app_state.database,
                acc.id.clone(),
                req.signers.clone(),
            )
            .await?;

            guardian_account_repo::update_all_guardians_for_account_to_status(
                &app_state.database,
                acc.id.clone(),
                "AVAILABLE".to_string(),
            )
            .await?;
            guardian_account_repo::update_guardians_for_account_to_status(
                &app_state.database,
                acc.id.clone(),
                req.guardians.clone(),
                "ACTIVE".to_string(),
            )
            .await?;
            let active_guardians = guardian_account_repo::find_all_active_guardians_by_account_id(
                &app_state.database,
                acc.id.clone(),
            )
            .await?;
            let active_guardian_accounts =
                to_account_guardians(&app_state.database, active_guardians).await?;

            Ok(AccountGuardianSettingsResponse {
                signers: SigningStrategy::OneOfOne,
                active_guardians: active_guardian_accounts,
                signing_strategies: SigningStrategy::all(),
            })
        }
        None => Err(anyhow::anyhow!("Account not found")),
    }
}

async fn validate_guardians_for_account(
    db: &DatabaseConnection,
    account_guardian_ids: Vec<String>,
    account_id: String,
) -> anyhow::Result<()> {
    let account_guardians = guardian_account_repo::find_all_guardians_for_account_by_ids(
        db,
        account_guardian_ids.clone(),
        account_id.clone(),
    )
    .await?;
    let all_guardians_for_account =
        guardian_account_repo::find_all_guardians_by_account_id(db, account_id.clone()).await?;
    let valid_ids = all_guardians_for_account
        .iter()
        .map(|g| g.id.clone())
        .collect::<Vec<String>>();
    if account_guardians.len() != account_guardian_ids.len() {
        return Err(anyhow::anyhow!(
            "Invalid account guardians supplied: {:?}, valid guardians are: {:?}",
            account_guardian_ids
                .iter()
                .filter(|id| !valid_ids.contains(id))
                .collect::<Vec<&String>>(),
            valid_ids
        ));
    }

    Ok(())
}

async fn validate_guardian_quantity(
    signer: SigningStrategy,
    account_guardian_ids: Vec<String>,
) -> anyhow::Result<()> {
    let required_quantity = SigningStrategy::get_signers_for(signer.clone())?;

    if account_guardian_ids.len() != required_quantity as usize {
        return Err(anyhow::anyhow!(format!(
            "Invalid number of guardians supplied. Expected {}, got {}, for signing strategy {}",
            required_quantity,
            account_guardian_ids.len(),
            signer
        )));
    }

    Ok(())
}

async fn get_guardian_settings(
    app_state: State<AppState>,
    AuthBearer(token): AuthBearer,
) -> Result<Json<ApiResponse<AccountGuardianSettingsResponse, ApiErrorResponse>>, StatusCode> {
    match try_get_guardian_settings(app_state, token).await {
        Ok(payload) => Ok(Json(api_success(payload))),
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn try_get_guardian_settings(
    app_state: State<AppState>,
    token: String,
) -> anyhow::Result<AccountGuardianSettingsResponse> {
    let claims = decode_jwt(token).await?;
    validate_jwt_claims(claims.clone()).await?;
    let account = account_repo::find_by_id(&app_state.database, claims.sub.clone()).await?;
    match account {
        Some(acc) => {
            let settings =
                guardian_settings_repo::find_for_account_id(&app_state.database, acc.id.clone())
                    .await?;
            match settings {
                Some(s) => {
                    let account_guardians =
                        guardian_account_repo::find_all_active_guardians_by_account_id(
                            &app_state.database,
                            acc.id.clone(),
                        )
                        .await?;

                    let active_guardians =
                        to_account_guardians(&app_state.database, account_guardians).await?;

                    Ok(AccountGuardianSettingsResponse {
                        signers: s.signers,
                        active_guardians,
                        signing_strategies: SigningStrategy::all(),
                    })
                }
                None => Err(anyhow::anyhow!("Settings not found")),
            }
        }
        None => Err(anyhow::anyhow!("Account not found")),
    }
}
