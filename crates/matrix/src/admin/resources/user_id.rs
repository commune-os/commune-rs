use std::{borrow::Cow, fmt::Display};

/// A Matrix user ID.
///
/// # Example
///
/// ```
/// @user:server.com
/// ```
///
/// The `server` value corresponds to the Synapse Server Name and can be
/// found on homeserver.yaml.
///
/// # Devnotes
///
/// Perhaps using Ruma's `UserId` would be better?
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserId(Cow<'static, str>);

impl UserId {
    pub fn new<S: AsRef<str>>(name: S, server_name: S) -> Self {
        let user_id = format!(
            "@{name}:{server_name}",
            name = name.as_ref(),
            server_name = server_name.as_ref()
        );

        Self(user_id.into())
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
