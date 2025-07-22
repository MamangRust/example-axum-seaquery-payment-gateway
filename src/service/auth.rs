use crate::{
    abstract_trait::{AuthServiceTrait, DynHashing, DynJwtService, DynUserRepository},
    domain::{
        request::{CreateUserRequest, LoginRequest, RegisterRequest},
        response::{ApiResponse, ErrorResponse, user::UserResponse},
    },
    utils::{AppError, random_vcc},
};
use async_trait::async_trait;

pub struct AuthService {
    repository: DynUserRepository,
    hashing: DynHashing,
    jwt_config: DynJwtService,
}

impl AuthService {
    pub fn new(
        repository: DynUserRepository,
        hashing: DynHashing,
        jwt_config: DynJwtService,
    ) -> Self {
        Self {
            repository,
            hashing,
            jwt_config,
        }
    }
}

#[async_trait]
impl AuthServiceTrait for AuthService {
    async fn register_user(
        &self,
        input: &RegisterRequest,
    ) -> Result<ApiResponse<UserResponse>, ErrorResponse> {
        let exists = self.repository.find_by_email_exists(&input.email).await?;

        if exists {
            return Err(ErrorResponse::from(AppError::EmailAlreadyExists));
        }

        let hashed_password = self
            .hashing
            .hash_password(&input.password)
            .await
            .map_err(|e| ErrorResponse::from(AppError::HashingError(e)))?;

        let noc_transfer = random_vcc().ok();

        let request = CreateUserRequest {
            firstname: input.firstname.clone(),
            lastname: input.lastname.clone(),
            email: input.email.clone(),
            password: hashed_password,
            confirm_password: input.confirm_password.clone(),
            noc_transfer: noc_transfer.to_owned(),
        };

        let create_user = self.repository.create_user(&request).await?;

        Ok(ApiResponse {
            status: "success".to_string(),
            message: "User registered successfully".to_string(),
            data: UserResponse::from(create_user),
        })
    }

    async fn login_user(&self, input: &LoginRequest) -> Result<ApiResponse<String>, ErrorResponse> {
        let user = self
            .repository
            .find_by_email(&input.email)
            .await?
            .ok_or_else(|| ErrorResponse::from(AppError::NotFound("User not found".to_string())))?;

        if self
            .hashing
            .compare_password(&user.password, &input.password)
            .await
            .is_err()
        {
            return Err(ErrorResponse::from(AppError::InvalidCredentials));
        }

        let token = self
            .jwt_config
            .generate_token(user.user_id as i64)
            .map_err(ErrorResponse::from)?;

        Ok(ApiResponse {
            status: "success".to_string(),
            message: "Login successful".to_string(),
            data: token,
        })
    }
}
