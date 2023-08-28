use super::{account_guardians_api, nomination_api};
use crate::{
    models::api::{
        api_error, api_success, ApiErrorResponse, ApiResponse, PrefundRequest, PrefundResponse,
        SendTransactionRequest, SendTransactionResponse,
    },
    repos::{account_repo, db::AppState},
    routes::sign_message,
};
use axum::{
    extract::{Path, Query, State},
    routing::post,
    Json, Router,
};

use chrono::Utc;
use clutch_wallet_lib::utils::wallet_lib::{self, abi_entry_point, Transaction};
use clutch_wallet_lib::utils::{bundler::UserOperationTransport, wallet_lib::WalletLib};

use ethers::{
    abi::{Address, Token},
    prelude::encode_function_data,
    signers::{LocalWallet, Signer},
    types::{Bytes, U256},
    utils,
};
use hyper::StatusCode;
use std::str::FromStr;

pub fn routes<S>(app_state: &AppState) -> Router<S> {
    Router::new()
        .route("/", post(send_transaction))
        .route("/prefund", post(prefund))
        .with_state(app_state.to_owned())
}

#[utoipa::path(
    post,
    path = "/transaction/",
    responses(
        (status = 200, description = "Send funds successfully", body = SendTransactionResponse),
    )
)]
async fn send_transaction(
    app_state: State<AppState>,
    Json(req): Json<SendTransactionRequest>,
) -> Result<Json<ApiResponse<SendTransactionResponse, ApiErrorResponse>>, StatusCode> {
    match try_send_transaction(&app_state, &req).await {
        Ok(payload) => Ok(Json(api_success(payload))),
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn try_send_transaction(
    app_state: &State<AppState>,
    req: &SendTransactionRequest,
) -> anyhow::Result<SendTransactionResponse> {
    let gas_price = "100";
    let app_state = app_state.0.clone();
    let account = account_repo::find_by_wallet_address(&app_state.database, req.from.clone())
        .await?
        .unwrap();
    let private_key = account.eoa_private_address;
    let wallet_signer = private_key
        .as_str()
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(app_state.settings.chain_id());

    let mut wallet_lib = app_state.wallet_lib;
    let abi_entrypoint = abi_entry_point();
    let call_data = encode_function_data(
        abi_entrypoint.function("depositTo")?,
        Token::Address(Address::from_str(&req.to).unwrap()),
    )
    .unwrap();

    let tx: Transaction = Transaction {
        to: Address::from_str(&req.to).unwrap(),
        value: Some(U256::from(utils::parse_ether(&req.value)?)),
        data: Some(call_data),
        gas_limit: None,
    };

    let dt = Utc::now();
    let valid_after = dt.timestamp() as u64;
    let valid_until = dt.timestamp() as u64 + 3600;

    let mut user_op_tx = wallet_lib
        .from_transaction(
            ethers::utils::parse_units(gas_price, "gwei")
                .unwrap()
                .into(),
            ethers::utils::parse_units(gas_price, "gwei")
                .unwrap()
                .into(),
            Address::from_str(&req.from).unwrap(),
            vec![tx],
            None,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Err{}", e))?;
    let _ = wallet_lib
        .estimate_user_operation_gas(&mut user_op_tx, None)
        .await
        .map_err(|e| anyhow::anyhow!("Err{}", e))?;
    // println!("user_op_tx {:?}", user_op_tx);
    let (packed_user_op_hash, validation_data) = wallet_lib
        .pack_user_op_hash(user_op_tx.clone(), Some(valid_after), Some(valid_until))
        .await
        .map_err(|e| anyhow::anyhow!("Err{}", e))?;

    let signature = sign_message(packed_user_op_hash, wallet_signer).await?;
    let packed_signature_ret = wallet_lib
        .pack_user_op_signature(signature, validation_data, None)
        .await
        .map_err(|e| anyhow::anyhow!("Err{}", e))?;

    user_op_tx.signature = ethers::types::Bytes::from(packed_signature_ret);
    let _ = wallet_lib
        .send_user_operation(user_op_tx.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Err{}", e))?;
    Ok(SendTransactionResponse {
        status: "Success".to_string(),
    })
}

async fn prefund(
    app_state: State<AppState>,
    Json(req): Json<PrefundRequest>,
) -> Result<Json<ApiResponse<PrefundResponse, ApiErrorResponse>>, StatusCode> {
    match try_prefud(&app_state, &req).await {
        Ok(payload) => Ok(Json(api_success(payload))),
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn try_prefud(
    app_state: &State<AppState>,
    req: &PrefundRequest,
) -> anyhow::Result<PrefundResponse> {
    let mut tx: Transaction = Default::default();
    let app_state = app_state.0.clone();
    let mut wallet_lib = app_state.wallet_lib;

    if req.send_type == "send_eth" {
        tx = Transaction {
            to: Address::from_str(&req.to).unwrap(),
            data: Some(Bytes::from(b"")),
            value: Some(ethers::utils::parse_ether(&req.value.clone().unwrap()).unwrap()),
            gas_limit: None,
        };
    } else if req.send_type == "send_erc20" {
        let call_data = WalletLib::transfer_erc20_calldata(
            Address::from_str(&req.to).unwrap(),
            ethers::utils::parse_ether(&req.value.clone().unwrap()).unwrap(),
        )
        .unwrap();
        tx = Transaction {
            to: Address::from_str(&req.to).unwrap(),
            data: Some(call_data),
            value: None,
            gas_limit: None,
        };
    };

    let max_fee_per_gas = U256::from_str(&app_state.settings.default_max_fee()).unwrap();
    let max_priority_fee_per_gas =
        U256::from_str(&app_state.settings.default_max_priority_fee()).unwrap();
    let mut user_op = wallet_lib
        .from_transaction(
            max_fee_per_gas,
            max_priority_fee_per_gas,
            Address::from_str(&req.from).unwrap(),
            vec![tx],
            None,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Err {}", e))
        .unwrap();

    let _ = wallet_lib
        .estimate_user_operation_gas(&mut user_op, None)
        .await
        .map_err(|e| anyhow::anyhow!("Err{}", e))?;

    let prefund = wallet_lib
        .pre_fund(user_op)
        .await
        .map_err(|e| anyhow::anyhow!("Err {}", e))
        .unwrap();
    Ok(PrefundResponse {
        deposit: prefund.deposit.to_string(),
        prefund: prefund.prefund.to_string(),
        missfund: prefund.missfund.to_string(),
    })
}
