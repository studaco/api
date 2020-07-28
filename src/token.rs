use hmac::{Hmac, NewMac};
use jwt::{Error, SignWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::env;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Claim {
    id: Uuid,
}

pub fn generate_token(id: Uuid) -> Result<String, Error> {
    let token_secret: String = env::var("TOKEN_SECRET").expect("TOKEN_SECRET is not set");
    let token_key: Hmac<Sha256> = Hmac::new_varkey(&token_secret.into_bytes()[..]).unwrap();
    let claim = Claim { id };
    claim.sign_with_key(&token_key)
}
