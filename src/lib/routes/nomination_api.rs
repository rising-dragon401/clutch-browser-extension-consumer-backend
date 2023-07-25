use crate::{
    models::api::{
        api_error, api_success, ApiErrorResponse, ApiResponse, ListNominationsResponse, Nomination,
        NominationCreateRequest, NominationCreateResponse, NominationDeleteResponse,
        NominationParams,
    },
    operations::jwt::{decode_jwt, validate_jwt_claims},
    repos::{account_repo, db::AppState, guardian_repo, nomination_repo},
};
use axum::{
    extract::{Path, Query, State},
    routing::{delete, get},
    Json, Router,
};
use axum_auth::AuthBearer;
use email_address::EmailAddress;
use hyper::StatusCode;
use uuid::Uuid;

pub fn routes<S>(app_state: &AppState) -> Router<S> {
    Router::new()
        .route("/", get(get_nominations).post(create_nomination))
        .route("/:nomination_id", delete(delete_nomination))
        .with_state(app_state.to_owned())
}

async fn delete_nomination(
    app_state: State<AppState>,
    AuthBearer(token): AuthBearer,
    Path(nomination_id): Path<String>,
) -> Result<Json<ApiResponse<NominationDeleteResponse, ApiErrorResponse>>, StatusCode> {
    match try_delete_nomination(app_state, token, nomination_id).await {
        Ok(payload) => Ok(Json(api_success(payload))),
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn try_delete_nomination(
    app_state: State<AppState>,
    token: String,
    nomination_id: String,
) -> anyhow::Result<NominationDeleteResponse> {
    let claims = decode_jwt(token).await?;
    validate_jwt_claims(claims.clone()).await?;
    let account = account_repo::find_by_id(&app_state.database, claims.sub.clone()).await?;
    match account {
        Some(acc) => {
            let nomination = nomination_repo::find_by_account_and_id(
                &app_state.database,
                acc.id.clone(),
                nomination_id.clone(),
            )
            .await?;
            match nomination {
                Some(nom) => {
                    if nom.status == *"PENDING" {
                        nomination_repo::delete_by_account_and_id(
                            &app_state.database,
                            acc.id.clone(),
                            nomination_id.clone(),
                        )
                        .await
                        .map(|_| NominationDeleteResponse { nomination_id })
                    } else {
                        Err(anyhow::anyhow!(
                            "Nomination can't be deleted with state: {}, must be in state PENDING",
                            nom.status
                        ))
                    }
                }
                None => Err(anyhow::anyhow!("Nomination not found")),
            }
        }
        None => Err(anyhow::anyhow!("Account not found")),
    }
}

async fn get_nominations(
    app_state: State<AppState>,
    AuthBearer(token): AuthBearer,
    Query(params): Query<NominationParams>,
) -> Result<Json<ApiResponse<ListNominationsResponse, ApiErrorResponse>>, StatusCode> {
    match try_get_nominations(app_state, token, &params).await {
        Ok(payload) => Ok(Json(api_success(payload))),
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn try_get_nominations(
    app_state: State<AppState>,
    token: String,
    params: &NominationParams,
) -> anyhow::Result<ListNominationsResponse> {
    let claims = decode_jwt(token).await?;
    validate_jwt_claims(claims.clone()).await?;
    let account = account_repo::find_by_id(&app_state.database, claims.sub.clone()).await?;
    match account {
        Some(acc) => match &params.nomination_id {
            Some(nomination_id) => find_all_by_id(&app_state, acc.id, nomination_id.clone()).await,
            None => match &params.status {
                Some(status) => find_all_by_status(&app_state, acc.id, status.clone()).await,
                None => match &params.email {
                    Some(email) => find_all_by_email(&app_state, acc.id, email.clone()).await,
                    None => find_all_nominations(&app_state, acc.id).await,
                },
            },
        },
        None => Err(anyhow::anyhow!("Account not found")),
    }
}

async fn find_all_by_id(
    app_state: &State<AppState>,
    account_id: String,
    nomination_id: String,
) -> anyhow::Result<ListNominationsResponse> {
    nomination_repo::find_all_by_account_and_id(&app_state.database, account_id, nomination_id)
        .await
        .map(|r| {
            let nominations = r
                .iter()
                .map(|nom| Nomination {
                    id: nom.id.to_string(),
                    email: nom.email.clone(),
                    guardian_id: nom.guardian_id.clone(),
                    account_id: nom.account_id.clone(),
                    status: nom.status.clone(),
                })
                .collect();
            ListNominationsResponse { nominations }
        })
}

async fn find_all_by_status(
    app_state: &State<AppState>,
    account_id: String,
    status: String,
) -> anyhow::Result<ListNominationsResponse> {
    nomination_repo::find_all_by_account_and_status(
        &app_state.database,
        account_id,
        status.to_uppercase(),
    )
    .await
    .map(|r| {
        let nominations = r
            .iter()
            .map(|nom| Nomination {
                id: nom.id.to_string(),
                email: nom.email.clone(),
                guardian_id: nom.guardian_id.clone(),
                account_id: nom.account_id.clone(),
                status: nom.status.clone(),
            })
            .collect();
        ListNominationsResponse { nominations }
    })
}

async fn find_all_by_email(
    app_state: &State<AppState>,
    account_id: String,
    email: String,
) -> anyhow::Result<ListNominationsResponse> {
    nomination_repo::find_all_by_account_and_email(&app_state.database, account_id, email)
        .await
        .map(|r| {
            let nominations = r
                .iter()
                .map(|nom| Nomination {
                    id: nom.id.to_string(),
                    email: nom.email.clone(),
                    guardian_id: nom.guardian_id.clone(),
                    account_id: nom.account_id.clone(),
                    status: nom.status.clone(),
                })
                .collect();
            ListNominationsResponse { nominations }
        })
}

async fn find_all_nominations(
    app_state: &State<AppState>,
    account_id: String,
) -> anyhow::Result<ListNominationsResponse> {
    nomination_repo::find_all_by_account(&app_state.database, account_id)
        .await
        .map(|r| {
            let nominations = r
                .iter()
                .map(|nom| Nomination {
                    id: nom.id.to_string(),
                    email: nom.email.clone(),
                    guardian_id: nom.guardian_id.clone(),
                    account_id: nom.account_id.clone(),
                    status: nom.status.clone(),
                })
                .collect();
            ListNominationsResponse { nominations }
        })
}

async fn create_nomination(
    app_state: State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(req): Json<NominationCreateRequest>,
) -> Result<Json<ApiResponse<NominationCreateResponse, ApiErrorResponse>>, StatusCode> {
    match try_create_nomination(&app_state, token, &req).await {
        Ok(payload) => Ok(Json(api_success(payload))),
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn try_create_nomination(
    app_state: &State<AppState>,
    token: String,
    req: &NominationCreateRequest,
) -> anyhow::Result<NominationCreateResponse> {
    if EmailAddress::is_valid(&req.email) {
        let claims = decode_jwt(token).await?;
        validate_jwt_claims(claims.clone()).await?;
        let account = account_repo::find_by_id(&app_state.database, claims.sub.clone()).await?;
        let nomination_id = Uuid::new_v4();
        match account {
            Some(acc) => {
                let maybe_guardian =
                    guardian_repo::find_by_email(&app_state.database, req.email.clone()).await?;
                match maybe_guardian {
                    Some(guardian) => {
                        nomination_repo::create(
                            &app_state.database,
                            nomination_id,
                            req.email.clone(),
                            acc.id,
                            guardian.id,
                            "PENDING".to_string(),
                        )
                        .await?;
                        Ok(NominationCreateResponse {
                            nomination_id: nomination_id.to_string(),
                        })
                    }
                    None => {
                        let maybe_account = account_repo::find_by_email(
                            &app_state.database,
                            req.email.clone().as_str(),
                        )
                        .await?;
                        match maybe_account {
                            Some(user_account) => {
                                let guardian_id = Uuid::new_v4();
                                guardian_repo::create(
                                    &app_state.database,
                                    guardian_id,
                                    req.email.clone(),
                                    Some(user_account.id),
                                    None,
                                )
                                .await?;
                                nomination_repo::create(
                                    &app_state.database,
                                    nomination_id,
                                    req.email.clone(),
                                    acc.id.clone(),
                                    guardian_id.to_string(),
                                    "PENDING".to_string(),
                                )
                                .await?;
                                Ok(NominationCreateResponse {
                                    nomination_id: nomination_id.to_string(),
                                })
                            }
                            None => {
                                let guardian_id = Uuid::new_v4();
                                guardian_repo::create(
                                    &app_state.database,
                                    guardian_id,
                                    req.email.clone(),
                                    None,
                                    None,
                                )
                                .await?;
                                nomination_repo::create(
                                    &app_state.database,
                                    nomination_id,
                                    req.email.clone(),
                                    acc.id,
                                    guardian_id.to_string(),
                                    "PENDING".to_string(),
                                )
                                .await?;
                                Ok(NominationCreateResponse {
                                    nomination_id: nomination_id.to_string(),
                                })
                            }
                        }
                    }
                }
            }
            None => Err(anyhow::anyhow!(
                "Error account not found for account: {}",
                claims.sub
            )),
        }
    } else {
        Err(anyhow::anyhow!("Invalid email format {}", req.email))
    }
}
