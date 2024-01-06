use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::admin::resources::user_id::UserId;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub device_id: String,
    pub is_guest: bool,
    pub user_id: UserId,
}

impl Session {
    /// Gets information about the owner of a given access token.
    ///
    /// Note that, as with the rest of the Client-Server API, Application
    /// Services may masquerade as users within their namespace by giving a
    /// user_id query parameter. In this situation, the server should verify
    /// that the given user_id is registered by the appservice, and return it
    /// in the response body.
    ///
    /// Refer: https://playground.matrix.org/#get-/_matrix/client/v3/account/whoami
    #[instrument(skip(client, access_token))]
    pub async fn get(
        client: &crate::http::Client,
        access_token: impl Into<String>,
    ) -> Result<Self> {
        // Clones the client in order to temporally set a token for the `GET`
        // request
        let mut tmp = (*client).clone();

        tmp.set_token(access_token)?;

        let resp = tmp.get("/_matrix/client/v3/account/whoami").await?;

        Ok(resp.json().await?)
    }
}
