//! This module contains handlers for user registration.
//!
//! reference: https://matrix-org.github.io/synapse/latest/admin_api/register_api.html

use hmac::Mac;

mod get_nonce;
mod register;

#[derive(Clone, Debug)]
pub struct Hmac {
    inner: Vec<u8>,
}

impl Hmac {
    fn new(
        shared_secret: &str,
        nonce: &str,
        username: &str,
        password: &str,
        admin: bool,
    ) -> Result<Self, hmac::digest::InvalidLength> {
        let mut mac = hmac::Hmac::<sha1::Sha1>::new_from_slice(shared_secret.as_bytes())?;
        let admin = match admin {
            true => "admin",
            false => "notadmin",
        };

        mac.update(
            &[nonce, username, password, admin]
                .map(str::as_bytes)
                .join(&0x00),
        );

        let result = mac.finalize().into_bytes();

        Ok(Self {
            inner: result.to_vec(),
        })
    }

    fn get(&self) -> String {
        hex::encode(&self.inner)
    }
}