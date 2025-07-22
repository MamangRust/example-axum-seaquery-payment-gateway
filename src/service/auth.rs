use crate::{
    abstract_trait::{AuthServiceTrait, DynHashing, DynJwtService, DynUserRepository},
    domain::{
        request::{CreateUserRequest, LoginRequest, RegisterRequest},
        response::{ApiResponse, ErrorResponse, user::UserResponse},
    },
    utils::{AppError, random_vcc},
};
use async_trait::async_trait;
use tracing::{error, info};

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
        info!("Attempting to register user with email: {}", input.email);
        let exists = self.repository.find_by_email_exists(&input.email).await?;

        if exists {
            error!("Email already exists: {}", input.email);
            return Err(ErrorResponse::from(AppError::EmailAlreadyExists));
        }

        let hashed_password = self
            .hashing
            .hash_password(&input.password)
            .await
            .map_err(|e| {
                error!("Error hashing password: {}", e);
                ErrorResponse::from(AppError::HashingError(e))
            })?;

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

        info!("User registered successfully with email: {}", input.email);

        Ok(ApiResponse {
            status: "success".to_string(),
            message: "User registered successfully".to_string(),
            data: UserResponse::from(create_user),
        })
    }

    async fn login_user(&self, input: &LoginRequest) -> Result<ApiResponse<String>, ErrorResponse> {
        info!("Attempting to log in user with email: {}", input.email);

        let user = match self.repository.find_by_email(&input.email).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                return Err(ErrorResponse::from(AppError::NotFound(
                    "User not found".to_string(),
                )));
            }
            Err(err) => {
                return Err(ErrorResponse::from(err));
            }
        };

        info!("User found: {} - {}", input.email, user.user_id);

        if self
            .hashing
            .compare_password(&user.password, &input.password)
            .await
            .is_err()
        {
            error!("Invalid credentials for user: {}", input.email);
            return Err(ErrorResponse::from(AppError::InvalidCredentials));
        }

        let token = self
            .jwt_config
            .generate_token(user.user_id as i64)
            .map_err(|e| {
                error!(
                    "Error generating token for user: {}, error: {e}",
                    input.email,
                );
                ErrorResponse::from(e)
            })?;

        info!("User logged in successfully: {}", input.email);

        Ok(ApiResponse {
            status: "success".to_string(),
            message: "Login successful".to_string(),
            data: token,
        })
    }
}
