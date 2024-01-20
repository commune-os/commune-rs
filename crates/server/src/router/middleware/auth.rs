use axum::body::Body;
use axum::http::{header::AUTHORIZATION, Request};
use axum::middleware::Next;
use axum::response::Response;

use commune::util::secret::Secret;

use crate::router::api::ApiError;
use crate::services::SharedServices;

#[derive(Debug, Clone)]
pub struct AccessToken(Secret);

impl ToString for AccessToken {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

pub async fn auth(mut request: Request<Body>, next: Next) -> Result<Response, ApiError> {
    let access_token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(ApiError::unauthorized())?
        .to_owned();

    let services = request
        .extensions()
        .get::<SharedServices>()
        .ok_or_else(|| {
            tracing::error!("SharedServices not found in request extensions");
            ApiError::internal_server_error()
        })?;

    let access_token = Secret::new(access_token);
    let user = services
        .commune
        .account
        .whoami(&access_token)
        .await
        .map_err(|err| {
            tracing::error!("Failed to validate token: {}", err);
            ApiError::internal_server_error()
        })?;

    request.extensions_mut().insert(user);
    request.extensions_mut().insert(AccessToken(access_token));

    Ok(next.run(request).await)
}