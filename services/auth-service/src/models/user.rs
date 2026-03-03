use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            created_at: user.created_at,
        }
    }
}

impl CreateUserRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.username.len() < 3 || self.username.len() > 50 {
            errors.push("username must be between 3 and 50 characters".to_string());
        }

        if self.display_name.is_empty() || self.display_name.len() > 100 {
            errors.push("display_name must be between 1 and 100 characters".to_string());
        }

        if self.password.len() < 8 || self.password.len() > 128 {
            errors.push("password must be between 8 and 128 characters".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_request_passes_validation() {
        let req = CreateUserRequest {
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            password: "password123".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn short_username_fails_validation() {
        let req = CreateUserRequest {
            username: "ab".to_string(),
            display_name: "Test".to_string(),
            password: "password123".to_string(),
        };
        let errors = req.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("username")));
    }

    #[test]
    fn empty_display_name_fails_validation() {
        let req = CreateUserRequest {
            username: "testuser".to_string(),
            display_name: "".to_string(),
            password: "password123".to_string(),
        };
        let errors = req.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("display_name")));
    }

    #[test]
    fn short_password_fails_validation() {
        let req = CreateUserRequest {
            username: "testuser".to_string(),
            display_name: "Test".to_string(),
            password: "short".to_string(),
        };
        let errors = req.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("password")));
    }

    #[test]
    fn multiple_validation_errors_returned() {
        let req = CreateUserRequest {
            username: "a".to_string(),
            display_name: "".to_string(),
            password: "short".to_string(),
        };
        let errors = req.validate().unwrap_err();
        assert_eq!(errors.len(), 3);
    }
}
