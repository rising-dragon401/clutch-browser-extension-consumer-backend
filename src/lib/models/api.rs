use sea_orm::entity::prelude::*;
use std::{fmt, str::FromStr};

use serde::{de, Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ApiPayload<S, E> {
    Success(S),
    Error(E),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse<T, E> {
    pub status: ApiResponseStatus,
    pub payload: ApiPayload<T, E>,
}

pub fn api_error<T>(error_message: String) -> ApiResponse<T, ApiErrorResponse> {
    ApiResponse {
        status: ApiResponseStatus::Error,
        payload: ApiPayload::Error(ApiErrorResponse { error_message }),
    }
}

pub fn api_success<T>(payload: T) -> ApiResponse<T, ApiErrorResponse> {
    ApiResponse {
        status: ApiResponseStatus::Success,
        payload: ApiPayload::Success(payload),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ApiResponseStatus {
    Success,
    Error,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiErrorResponse {
    pub error_message: String,
}

// Email Verification API
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct VerificationRequest {
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct VerificationResponse {
    pub success: bool,
}

// Account API
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct AccountCreateRequest {
    pub email: String,
    pub code: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct AccountCreateResponse {
    pub jwt: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct AccountUpdateRequest {
    pub email: String,
    pub wallet_address: Option<String>,
    pub eoa_address: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct AccountUpdateResponse {
    pub updated: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Account {
    pub id: String,
    pub email: String,
    pub wallet_address: String,
    pub eoa_address: String,
    pub eoa_private_address: String,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct ListAccountsResponse {
    pub accounts: Vec<Account>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AccountParams {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub wallet_address: Option<String>,
    pub eoa_address: Option<String>,
    pub email: Option<String>,
}

fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

// Nominations API
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NominationCreateRequest {
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NominationCreateResponse {
    pub nomination_id: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Nomination {
    pub id: String,
    pub email: String,
    pub guardian_id: String,
    pub account_id: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct ListNominationsResponse {
    pub nominations: Vec<Nomination>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct NominationParams {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub status: Option<String>,
    pub nomination_id: Option<String>,
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NominationDeleteResponse {
    pub nomination_id: String,
}

// Guardian API for Guardians

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct GuardianNominationParams {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub status: Option<String>,
    pub nomination_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct GuardianAccountParams {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub account_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NominationUpdateRequest {
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NominationUpdateResponse {
    pub nomination_id: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ListGuardianAccountsResponse {
    pub accounts: Vec<GuardianAccount>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct GuardianAccount {
    pub id: String,
    pub email: String,
    pub wallet_address: String,
}

// Guardian API for Accounts

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AccountGuardianParams {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub guardian_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct AccountGuardian {
    pub id: String,
    pub email: String,
    pub wallet_address: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ListAccountGuardiansResponse {
    pub guardians: Vec<AccountGuardian>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AccountGuardianDeleteResponse {
    pub guardian_id: String,
}

// Guardian Settings API

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountGuardianSettingsResponse {
    pub signing_strategies: Vec<SigningStrategy>,
    pub signers: SigningStrategy,
    pub active_guardians: Vec<AccountGuardian>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountGuardianSettingsRequest {
    pub signers: SigningStrategy,
    pub guardians: Vec<String>,
}

#[derive(EnumIter, DeriveActiveEnum, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[sea_orm(rs_type = "String", db_type = "String(Some(1))")]
pub enum SigningStrategy {
    #[sea_orm(string_value = "OneOfOne")]
    OneOfOne,
    #[sea_orm(string_value = "OneOfTwo")]
    OneOfTwo,
    #[sea_orm(string_value = "TwoOfTwo")]
    TwoOfTwo,
    #[sea_orm(string_value = "OneOfThree")]
    OneOfThree,
    #[sea_orm(string_value = "TwoOfThree")]
    TwoOfThree,
    #[sea_orm(string_value = "ThreeOfThree")]
    ThreeOfThree,
}

impl SigningStrategy {
    pub fn get_signers_for(item: SigningStrategy) -> anyhow::Result<i64> {
        match item {
            SigningStrategy::OneOfOne => Ok(1),
            SigningStrategy::OneOfTwo => Ok(2),
            SigningStrategy::TwoOfTwo => Ok(2),
            SigningStrategy::OneOfThree => Ok(3),
            SigningStrategy::TwoOfThree => Ok(3),
            SigningStrategy::ThreeOfThree => Ok(3),
        }
    }

    pub fn all() -> Vec<SigningStrategy> {
        vec![
            SigningStrategy::OneOfOne,
            SigningStrategy::OneOfTwo,
            SigningStrategy::TwoOfTwo,
            SigningStrategy::OneOfThree,
            SigningStrategy::TwoOfThree,
            SigningStrategy::ThreeOfThree,
        ]
    }
}
