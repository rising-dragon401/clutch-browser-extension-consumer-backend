use crate::{
    models::api::{
        api_error, api_success, ApiErrorResponse, ApiResponse, GuardianAccount,
        GuardianAccountParams, GuardianNominationParams, ListGuardianAccountsResponse,
        ListNominationsResponse, Nomination, NominationUpdateRequest, NominationUpdateResponse,
    },
    operations::jwt::{decode_jwt, validate_jwt_claims},
    repos::{account_repo, db::AppState, guardian_account_repo, guardian_repo, nomination_repo},
};
use axum::{
    extract::{Path, Query, State},
    routing::{get, put},
    Json, Router,
};
use axum_auth::AuthBearer;
use hyper::StatusCode;
use uuid::Uuid;

pub fn routes<S>(app_state: &AppState) -> Router<S> {
    Router::new()
        .route("/nominations", get(get_nominations))
        .route("/accounts", get(get_accounts))
        .route("/nomination/:nomination_id", put(update_status))
        .with_state(app_state.to_owned())
}

async fn update_status(
    app_state: State<AppState>,
    AuthBearer(token): AuthBearer,
    Path(nomination_id): Path<String>,
    Json(req): Json<NominationUpdateRequest>,
) -> Result<Json<ApiResponse<NominationUpdateResponse, ApiErrorResponse>>, StatusCode> {
    match try_update_status(app_state, token, nomination_id, req.status).await {
        Ok(payload) => Ok(Json(api_success(payload))),
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn try_update_status(
    app_state: State<AppState>,
    token: String,
    nomination_id: String,
    status: String,
) -> anyhow::Result<NominationUpdateResponse> {
    let claims = decode_jwt(token).await?;
    validate_jwt_claims(claims.clone()).await?;
    let account = account_repo::find_by_id(&app_state.database, claims.sub.clone()).await?;
    match account {
        Some(acc) => {
            let guardian = guardian_repo::find_by_account_id(&app_state.database, acc.id).await?;
            match guardian {
                Some(g) => {
                    let nominations = nomination_repo::find_all_by_guardian_and_nomination_id(
                        &app_state.database,
                        g.id.clone(),
                        nomination_id.clone(),
                    )
                    .await?;
                    match nominations.len() {
                        0 => Err(anyhow::anyhow!("Nomination not found")),
                        _ => {
                            let nomination = nominations.get(0).unwrap();
                            validate_nomination_status(status.clone(), nomination.status.clone())
                                .await?;

                            guardian_account_repo::create(
                                &app_state.database,
                                Uuid::new_v4(),
                                g.id.clone(),
                                nomination.account_id.clone(),
                                "AVAILABLE".to_string(),
                            )
                            .await?;

                            nomination_repo::update_status_by_guardian_id(
                                &app_state.database,
                                nomination.id.clone(),
                                g.id,
                                status.clone().to_uppercase(),
                            )
                            .await?;
                            Ok(NominationUpdateResponse {
                                nomination_id: nomination.id.clone(),
                                status: status.clone().to_uppercase(),
                            })
                        }
                    }
                }
                None => Err(anyhow::anyhow!("Guardian not found")),
            }
        }
        None => Err(anyhow::anyhow!("Account not found")),
    }
}

async fn validate_nomination_status(
    requested_status: String,
    current_status: String,
) -> anyhow::Result<()> {
    match requested_status.to_uppercase().as_str() {
        "PENDING" => Err(anyhow::anyhow!(
            "Invalid status must be ACCEPTED or REJECTED"
        )),
        "ACCEPTED" => {
            if current_status == "REJECTED" {
                Err(anyhow::anyhow!("Nomination already rejected"))
            } else {
                Ok(())
            }
        }
        "REJECTED" => {
            if current_status == "ACCEPTED" {
                Err(anyhow::anyhow!("Nomination already accepted"))
            } else {
                Ok(())
            }
        }
        _ => Err(anyhow::anyhow!(
            "Invalid status must be ACCEPTED or REJECTED"
        )),
    }
}

async fn get_accounts(
    app_state: State<AppState>,
    AuthBearer(token): AuthBearer,
    Query(params): Query<GuardianAccountParams>,
) -> Result<Json<ApiResponse<ListGuardianAccountsResponse, ApiErrorResponse>>, StatusCode> {
    match try_get_accounts(app_state, token, &params).await {
        Ok(payload) => Ok(Json(api_success(payload))),
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn try_get_accounts(
    app_state: State<AppState>,
    token: String,
    params: &GuardianAccountParams,
) -> anyhow::Result<ListGuardianAccountsResponse> {
    let claims = decode_jwt(token).await?;
    validate_jwt_claims(claims.clone()).await?;
    let account = account_repo::find_by_id(&app_state.database, claims.sub.clone()).await?;
    match account {
        Some(acc) => match &params.account_id {
            Some(account_id) => {
                find_all_guardian_accounts_by_account_id(&app_state, acc.id, account_id.to_owned())
                    .await
            }
            None => find_all_guardian_accounts(&app_state, acc.id).await,
        },

        None => Err(anyhow::anyhow!("Account not found")),
    }
}

async fn find_all_guardian_accounts_by_account_id(
    app_state: &State<AppState>,
    guardian_account_id: String,
    account_id: String,
) -> anyhow::Result<ListGuardianAccountsResponse> {
    let guardian =
        guardian_repo::find_by_account_id(&app_state.database, guardian_account_id).await?;
    match guardian {
        Some(g) => {
            let guardian_accounts =
                guardian_account_repo::find_all_accounts_by_guardian_id_and_account_id(
                    &app_state.database,
                    g.id,
                    account_id,
                )
                .await?;

            let account_ids = guardian_accounts
                .iter()
                .map(|ga| ga.account_id.clone())
                .collect::<Vec<String>>();

            account_repo::find_all_by_account_ids(&app_state.database, account_ids)
                .await
                .map(|r| {
                    let accounts = r
                        .iter()
                        .map(|acc| GuardianAccount {
                            id: acc.id.to_string(),
                            email: acc.email.clone(),
                            wallet_address: acc.wallet_address.clone(),
                        })
                        .collect();
                    ListGuardianAccountsResponse { accounts }
                })
        }
        None => Ok(ListGuardianAccountsResponse { accounts: vec![] }),
    }
}

async fn find_all_guardian_accounts(
    app_state: &State<AppState>,
    guardian_account_id: String,
) -> anyhow::Result<ListGuardianAccountsResponse> {
    let guardian =
        guardian_repo::find_by_account_id(&app_state.database, guardian_account_id).await?;
    match guardian {
        Some(g) => {
            let guardian_accounts =
                guardian_account_repo::find_all_accounts_by_guardian_id(&app_state.database, g.id)
                    .await?;

            let account_ids = guardian_accounts
                .iter()
                .map(|ga| ga.account_id.clone())
                .collect::<Vec<String>>();

            account_repo::find_all_by_account_ids(&app_state.database, account_ids)
                .await
                .map(|r| {
                    let accounts = r
                        .iter()
                        .map(|acc| GuardianAccount {
                            id: acc.id.to_string(),
                            email: acc.email.clone(),
                            wallet_address: acc.wallet_address.clone(),
                        })
                        .collect();
                    ListGuardianAccountsResponse { accounts }
                })
        }
        None => Ok(ListGuardianAccountsResponse { accounts: vec![] }),
    }
}

async fn get_nominations(
    app_state: State<AppState>,
    AuthBearer(token): AuthBearer,
    Query(params): Query<GuardianNominationParams>,
) -> Result<Json<ApiResponse<ListNominationsResponse, ApiErrorResponse>>, StatusCode> {
    match try_get_nominations(app_state, token, &params).await {
        Ok(payload) => Ok(Json(api_success(payload))),
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn try_get_nominations(
    app_state: State<AppState>,
    token: String,
    params: &GuardianNominationParams,
) -> anyhow::Result<ListNominationsResponse> {
    let claims = decode_jwt(token).await?;
    validate_jwt_claims(claims.clone()).await?;
    let account = account_repo::find_by_id(&app_state.database, claims.sub.clone()).await?;
    match account {
        Some(acc) => match &params.nomination_id {
            Some(nomination_id) => {
                find_all_nominations_by_id(&app_state, acc.id, nomination_id.clone()).await
            }
            None => match &params.status {
                Some(status) => {
                    find_all_nominations_by_status(
                        &app_state,
                        acc.id,
                        status.clone().to_uppercase(),
                    )
                    .await
                }
                None => find_all_nominations(&app_state, acc.id).await,
            },
        },
        None => Err(anyhow::anyhow!("Account not found")),
    }
}

async fn find_all_nominations_by_id(
    app_state: &State<AppState>,
    account_id: String,
    nomination_id: String,
) -> anyhow::Result<ListNominationsResponse> {
    let guardian = guardian_repo::find_by_account_id(&app_state.database, account_id).await?;
    match guardian {
        Some(g) => nomination_repo::find_all_by_guardian_and_nomination_id(
            &app_state.database,
            g.id,
            nomination_id,
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
        }),
        None => Ok(ListNominationsResponse {
            nominations: vec![],
        }),
    }
}

async fn find_all_nominations_by_status(
    app_state: &State<AppState>,
    account_id: String,
    status: String,
) -> anyhow::Result<ListNominationsResponse> {
    let guardian = guardian_repo::find_by_account_id(&app_state.database, account_id).await?;
    match guardian {
        Some(g) => {
            nomination_repo::find_all_by_guardian_and_status(&app_state.database, g.id, status)
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
        None => Ok(ListNominationsResponse {
            nominations: vec![],
        }),
    }
}

async fn find_all_nominations(
    app_state: &State<AppState>,
    account_id: String,
) -> anyhow::Result<ListNominationsResponse> {
    let guardian = guardian_repo::find_by_account_id(&app_state.database, account_id).await?;
    match guardian {
        Some(g) => nomination_repo::find_all_by_guardian(&app_state.database, g.id)
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
            }),
        None => Ok(ListNominationsResponse {
            nominations: vec![],
        }),
    }
}
