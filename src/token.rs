use std::env;
use uuid::Uuid;
use hmac::{Hmac, NewMac};
use sha2::Sha256;
use jwt::{SignWithKey, Error};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Claim {
    id: Uuid
}

pub fn generate_token(id: Uuid) -> Result<String, Error> {
    let token_secret: String = env::var("TOKEN_SECRET").expect("TOKEN_SECRET is not set");
    let token_key: Hmac<Sha256> = Hmac::new_varkey(&token_secret.into_bytes()[..]).unwrap();
    let claim = Claim { id };
    claim.sign_with_key(&token_key)
}
