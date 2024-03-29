use matrix::client::{login::*, uiaa::UserIdentifier};

use crate::{commune, error::Result, util::secret::Secret};

pub async fn service(username: impl Into<String>, password: &Secret) -> Result<Response> {
    let req = Request::new(
        LoginType::Password {
            password: password.inner(),
        },
        Some(UserIdentifier::User {
            user: username.into(),
        }),
        "commune".to_owned(),
        Some(true),
    );

    commune()
        .send_matrix_request(req, None)
        .await
        .map_err(Into::into)
}
