use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use commune::{account::error::AccountErrorCode, util::secret::Secret, Error};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use commune::account::service::VerifyCodeDto;

use crate::{router::api::ApiError, services::SharedServices};

#[instrument(skip(services, payload))]
pub async fn handler(
    Extension(services): Extension<SharedServices>,
    Json(payload): Json<AccountVerifyCodeEmailPayload>,
) -> Response {
    let dto = VerifyCodeDto::from(payload);

    match services
        .commune
        .account
        .is_email_available(&dto.email)
        .await
    {
        Ok(available) => {
            if !available {
                let email_taken_error = AccountErrorCode::EmailTaken(dto.email);
                let error = Error::User(email_taken_error);

                return ApiError::from(error).into_response();
            }
        }
        Err(err) => {
            tracing::warn!(?err, ?dto, "Failed to verify email availability");
            return ApiError::from(err).into_response();
        }
    }

    match services.commune.account.verify_code(dto).await {
        Ok(valid) => {
            let mut response = Json(VerifyCodeEmailResponse { valid }).into_response();

            *response.status_mut() = StatusCode::OK;
            response
        }
        Err(err) => {
            tracing::warn!(?err, "Failed to register user");
            ApiError::from(err).into_response()
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct AccountVerifyCodeEmailPayload {
    pub email: String,
    pub session: Uuid,
    pub code: Secret,
}

impl From<AccountVerifyCodeEmailPayload> for VerifyCodeDto {
    fn from(payload: AccountVerifyCodeEmailPayload) -> Self {
        Self {
            email: payload.email,
            session: payload.session,
            code: payload.code,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct VerifyCodeEmailResponse {
    pub valid: bool,
}
