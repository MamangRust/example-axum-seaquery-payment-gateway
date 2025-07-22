use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct RegisterRequest {
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
