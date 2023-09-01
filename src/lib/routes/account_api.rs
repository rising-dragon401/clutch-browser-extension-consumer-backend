use super::{account_guardians_api, nomination_api, sign_message};
use crate::{
    config::settings::Settings,
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
    utils::convert_to_hex,
};
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use axum_auth::AuthBearer;
use chrono::Utc;
use clutch_wallet_lib::utils::wallet_lib::{WalletInstance, WalletLib};
use email_address::EmailAddress;
use ethers::{
    abi::Token,
    prelude::*,
    providers::Provider,
    types::{Address, U256},
};
use hyper::StatusCode;
use rand::thread_rng;
use sea_orm::DatabaseConnection;
use std::str::FromStr;
use uuid::Uuid;

pub fn routes<S>(app_state: &AppState) -> Router<S> {
    Router::new()
        .route(
            "/",
            get(get_accounts).post(create_account).put(update_account),
        )
        .route("/:email", get(get_account_by_email))
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

async fn get_account_by_email(
    app_state: State<AppState>,
    Path(email): Path<String>,
) -> Result<Json<ApiResponse<Account, ApiErrorResponse>>, StatusCode> {
    if EmailAddress::is_valid(&email) {
        let account = account_repo::find_by_email(&app_state.database, &email)
            .await
            .unwrap();
        match account {
            Some(acc) => Ok(Json(api_success(Account {
                id: acc.id,
                email: acc.email,
                wallet_address: acc.wallet_address,
                eoa_address: acc.eoa_address,
                eoa_private_address: acc.eoa_private_address,
                updated_at: acc.updated_at,
            }))),
            None => Ok(Json(api_error(format!("No such email {}", email)))),
        }
    } else {
        Ok(Json(api_error(format!("Invalid email format {}", email))))
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
                    eoa_private_address: account.eoa_private_address.clone(),
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
                    eoa_private_address: account.eoa_private_address.clone(),
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
                    eoa_private_address: account.eoa_private_address.clone(),
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
                    eoa_private_address: account.eoa_private_address.clone(),
                    updated_at: account.updated_at,
                })
                .collect();
            Ok(Json(api_success(ListAccountsResponse { accounts })))
        }
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

#[utoipa::path(
  post,
  path="/accounts/",
  responses(
    (status = 200, description = "Account is created successfully", body=AccountCreateResponse)
  )
)]
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
                validate_code(&app_state.database, req.email.clone(), req.code.clone()).await?;
                let app_state_data = app_state.0.clone();
                let account_id = Uuid::new_v4();
                let (contract_wallet, eoa_public, eoa_private) = create_wallet_addr(
                    app_state_data.wallet_lib,
                    &app_state.settings,
                    &req.paymaster_tokens,
                )
                .await?;
                store_account(
                    app_state,
                    req,
                    account_id,
                    convert_to_hex(contract_wallet),
                    convert_to_hex(eoa_public),
                    eoa_private,
                )
                .await?;
                let jwt = generate_jwt(account_id.to_string()).await?;
                Ok(AccountCreateResponse {
                    jwt,
                    contract_wallet_addr: convert_to_hex(contract_wallet),
                })
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

async fn create_wallet_addr(
    wallet_lib: WalletLib,
    settings: &Settings,
    paymaster_tokens: &Option<Vec<String>>,
) -> Result<(H160, H160, String), anyhow::Error> {
    let mut wallet_lib = wallet_lib;
    let wallet_signer = LocalWallet::new(&mut thread_rng()).with_chain_id(settings.chain_id());
    let zero_hash: H256 = [0u8; 32].into();

    let mut user_op = wallet_lib
        .create_unsigned_deploy_wallet_user_op(0, wallet_signer.address(), zero_hash, "0x", None)
        .await
        .map_err(|err| anyhow::anyhow!("Err, {}", err))?;
    let gas_price = "100"; // gwei
    user_op.max_fee_per_gas = ethers::utils::parse_units(gas_price, "gwei")
        .unwrap()
        .into();
    user_op.max_priority_fee_per_gas = ethers::utils::parse_units(gas_price, "gwei")
        .unwrap()
        .into();

    let _ = wallet_lib
        .estimate_user_operation_gas(&mut user_op, None)
        .await
        .unwrap();

    if let Some(paymaster_tokens) = paymaster_tokens {
        let to = paymaster_tokens
            .iter()
            .map(|addr| Address::from_str(addr).unwrap())
            .collect::<Vec<Address>>();
        let approve_data = WalletInstance::approve(
            Address::from_str(&settings.contracts.paymaster().clone()).unwrap(),
            ethers::utils::parse_ether(100000).unwrap(),
        )
        .unwrap();
        let approve_call_data = to
            .iter()
            .map(|_| approve_data.clone())
            .collect::<Vec<Bytes>>();
        let call_data = WalletInstance::execute_batch(to, approve_call_data).unwrap();
        user_op.call_data = call_data;
        user_op.call_gas_limit = U256::from(50000 * (paymaster_tokens.len() + 1));
    }
    let pre_fund_ret = wallet_lib
        .pre_fund(user_op.clone())
        .await
        .map_err(|err| anyhow::anyhow!("Err, {}", err))?;

    let default_wallet = settings
        .wallet_private_key()
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(settings.chain_id());

    let key_as_bytes = wallet_signer.signer().to_bytes();
    let private_key = hex::encode(key_as_bytes);

    let http = Provider::<Http>::try_from(&settings.rpc())?;
    let provider = SignerMiddleware::new(http.clone(), default_wallet.clone());
    let tx = TransactionRequest::new()
        .to(user_op.clone().sender)
        .value(pre_fund_ret.missfund);

    let _ = provider.send_transaction(tx, None).await?.await?;

    let dt = Utc::now();
    let valid_after = dt.timestamp() as u64;
    let valid_until = dt.timestamp() as u64 + 3600;

    let (packed_user_op_hash, validation_data) = wallet_lib
        .pack_user_op_hash(user_op.clone(), Some(valid_after), Some(valid_until))
        .await
        .map_err(|e| anyhow::anyhow!("Err{}", e))?;

    // let key_as_bytes = wallet.signer().to_bytes();
    // let private_key = hex::encode(key_as_bytes);
    let signature = sign_message(packed_user_op_hash, wallet_signer.clone()).await?;
    let packed_signature_ret = wallet_lib
        .pack_user_op_signature(signature, validation_data, None)
        .await
        .map_err(|e| anyhow::anyhow!("Err{}", e))?;

    user_op.signature = ethers::types::Bytes::from(packed_signature_ret);
    let _: bool = wallet_lib
        .send_user_operation(user_op.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Err{}", e))?;

    Ok((user_op.sender, wallet_signer.address(), private_key))
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
