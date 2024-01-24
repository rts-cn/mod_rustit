use hmac::{Hmac, Mac};
use jwt::{RegisteredClaims, SignWithKey, VerifyWithKey};
use sha2::Sha256;

pub fn new_token(secret_key: String, uuid: String) -> Result<String, &'static str> {
    let claims = RegisteredClaims {
        subject: Some(uuid.to_string()),
        ..Default::default()
    };
    let key: Hmac<Sha256> =
        Hmac::new_from_slice(secret_key.as_bytes()).map_err(|_e| "Invalid key")?;
    let signed_token = claims.sign_with_key(&key).map_err(|_e| "Sign failed")?;
    Ok(signed_token)
}

pub fn verify(secret_key: &str, token: &str) -> Result<String, &'static str> {
    let key: Hmac<Sha256> =
        Hmac::new_from_slice(secret_key.as_bytes()).map_err(|_e| "Invalid key")?;
    let claims: RegisteredClaims =
        VerifyWithKey::verify_with_key(token, &key).map_err(|_e| "Parse failed")?;
    claims.subject.ok_or("Missing subject")
}
