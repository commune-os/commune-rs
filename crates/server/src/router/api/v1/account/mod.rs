pub mod create;
pub mod login;
pub mod verify_code;

use axum::routing::post;
use axum::Router;

use crate::services::SharedServices;

pub struct Account;

impl Account {
    pub fn routes() -> Router<SharedServices> {
        let verify = Router::new().route("/code", post(verify_code::handler));

        Router::new()
            .route("/", post(create::handler))
            .route("/login", post(login::handler))
            .nest("/verify", verify)
    }
}
