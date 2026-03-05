use base64::Engine;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::RngCore;
use rsa::pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey};
use rsa::pkcs8::DecodePrivateKey;
use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub username: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug, Serialize)]
pub struct JwkKey {
    pub kty: String,
    pub alg: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub kid: String,
    pub n: String,
    pub e: String,
}

#[derive(Debug, Serialize)]
pub struct Jwks {
    pub keys: Vec<JwkKey>,
}

#[derive(Clone)]
pub struct TokenService {
    encoding_key: EncodingKey,
    kid: String,
    jwks_json: serde_json::Value,
}

impl TokenService {
    pub fn new(pem: Option<&str>) -> Self {
        let private_key = match pem {
            Some(pem_str) => {
                RsaPrivateKey::from_pkcs8_pem(pem_str).expect("Failed to parse RSA private key")
            }
            None => {
                tracing::info!("No RSA_PRIVATE_KEY_PEM configured, generating dev key");
                let mut rng = rand::thread_rng();
                RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate RSA key")
            }
        };

        let public_key = private_key.to_public_key();

        // Create kid from public key hash
        let pub_der = public_key
            .to_pkcs1_der()
            .expect("Failed to encode public key");
        let kid = {
            let mut hasher = Sha256::new();
            hasher.update(pub_der.as_bytes());
            let hash = hasher.finalize();
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&hash[..8])
        };

        // Build JWKS from public key components
        let n_bytes = public_key.n().to_bytes_be();
        let e_bytes = public_key.e().to_bytes_be();

        let jwk = JwkKey {
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            use_: "sig".to_string(),
            kid: kid.clone(),
            n: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&n_bytes),
            e: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&e_bytes),
        };
        let jwks = Jwks { keys: vec![jwk] };
        let jwks_json = serde_json::to_value(&jwks).expect("Failed to serialize JWKS");

        // Create encoding key from PKCS#1 DER (required by jsonwebtoken)
        let private_der = private_key
            .to_pkcs1_der()
            .expect("Failed to encode private key");
        let encoding_key = EncodingKey::from_rsa_der(private_der.as_bytes());

        Self {
            encoding_key,
            kid,
            jwks_json,
        }
    }

    pub fn create_access_token(
        &self,
        user_id: Uuid,
        username: &str,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let claims = Claims {
            iss: "auth-service".to_string(),
            sub: user_id.to_string(),
            username: username.to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::minutes(15)).timestamp(),
        };

        let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some(self.kid.clone());

        encode(&header, &claims, &self.encoding_key)
    }

    pub fn generate_refresh_token() -> (String, String) {
        // Generate 256-bit random token
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);

        // Hash for storage (we store only the hash, never the raw token)
        let hash = Self::hash_refresh_token(&token);

        (token, hash)
    }

    pub fn hash_refresh_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn jwks(&self) -> &serde_json::Value {
        &self.jwks_json
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_valid_access_token() {
        let svc = TokenService::new(None);
        let token = svc
            .create_access_token(Uuid::new_v4(), "testuser")
            .unwrap();

        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3, "JWT should have 3 parts");
    }

    #[test]
    fn access_token_has_correct_claims() {
        let svc = TokenService::new(None);
        let user_id = Uuid::new_v4();
        let token = svc.create_access_token(user_id, "alice").unwrap();

        let parts: Vec<&str> = token.split('.').collect();
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .unwrap();
        let claims: Claims = serde_json::from_slice(&payload).unwrap();

        assert_eq!(claims.iss, "auth-service");
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.username, "alice");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn access_token_ttl_is_15_minutes() {
        let svc = TokenService::new(None);
        let token = svc
            .create_access_token(Uuid::new_v4(), "testuser")
            .unwrap();

        let parts: Vec<&str> = token.split('.').collect();
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .unwrap();
        let claims: Claims = serde_json::from_slice(&payload).unwrap();

        let ttl = claims.exp - claims.iat;
        assert_eq!(ttl, 900); // 15 * 60
    }

    #[test]
    fn refresh_token_is_unique() {
        let (t1, _) = TokenService::generate_refresh_token();
        let (t2, _) = TokenService::generate_refresh_token();
        assert_ne!(t1, t2);
    }

    #[test]
    fn refresh_token_hash_is_deterministic() {
        let (token, hash) = TokenService::generate_refresh_token();
        assert_eq!(hash, TokenService::hash_refresh_token(&token));
    }

    #[test]
    fn jwks_has_required_fields() {
        let svc = TokenService::new(None);
        let jwks = svc.jwks();

        let keys = jwks["keys"].as_array().unwrap();
        assert_eq!(keys.len(), 1);

        let key = &keys[0];
        assert_eq!(key["kty"], "RSA");
        assert_eq!(key["alg"], "RS256");
        assert_eq!(key["use"], "sig");
        assert!(key["kid"].is_string());
        assert!(key["n"].is_string());
        assert!(key["e"].is_string());
    }

    #[test]
    fn token_verifies_with_jwks_key() {
        let svc = TokenService::new(None);
        let token = svc
            .create_access_token(Uuid::new_v4(), "verify_me")
            .unwrap();

        let jwks = svc.jwks();
        let key = &jwks["keys"][0];
        let n = key["n"].as_str().unwrap();
        let e = key["e"].as_str().unwrap();

        let decoding_key = jsonwebtoken::DecodingKey::from_rsa_components(n, e).unwrap();
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_issuer(&["auth-service"]);
        validation.validate_aud = false;

        let result = jsonwebtoken::decode::<Claims>(&token, &decoding_key, &validation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().claims.username, "verify_me");
    }
}
