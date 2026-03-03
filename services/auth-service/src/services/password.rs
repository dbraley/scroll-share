use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_produces_argon2id_hash() {
        let hash = hash_password("testpassword").unwrap();
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn verify_correct_password() {
        let hash = hash_password("mypassword").unwrap();
        assert!(verify_password("mypassword", &hash).unwrap());
    }

    #[test]
    fn verify_wrong_password() {
        let hash = hash_password("mypassword").unwrap();
        assert!(!verify_password("wrongpassword", &hash).unwrap());
    }

    #[test]
    fn different_passwords_produce_different_hashes() {
        let hash1 = hash_password("password1").unwrap();
        let hash2 = hash_password("password2").unwrap();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn same_password_produces_different_hashes_due_to_salt() {
        let hash1 = hash_password("samepassword").unwrap();
        let hash2 = hash_password("samepassword").unwrap();
        assert_ne!(hash1, hash2);
        // But both should verify
        assert!(verify_password("samepassword", &hash1).unwrap());
        assert!(verify_password("samepassword", &hash2).unwrap());
    }
}
