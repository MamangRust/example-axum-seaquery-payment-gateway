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
        info!(
            "📝 [Auth] Attempting to register user with email: {}",
            input.email
        );

        let exists = self
            .repository
            .find_by_email_exists(&input.email)
            .await
            .map_err(|e| {
                error!(
                    "❌ [Auth] Database error while checking email existence: {}",
                    e
                );
                ErrorResponse::from(e)
            })?;

        if exists {
            error!(
                "🛑 [Auth] Registration failed: Email already exists: {}",
                input.email
            );
            return Err(ErrorResponse::from(AppError::EmailAlreadyExists));
        }

        let hashed_password = self
            .hashing
            .hash_password(&input.password)
            .await
            .map_err(|e| {
                error!("🔐 [Auth] Failed to hash password: {}", e);
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

        let create_user = self.repository.create_user(&request).await.map_err(|e| {
            error!("❌ [Auth] Failed to create user in database: {}", e);
            ErrorResponse::from(e)
        })?;

        info!(
            "✅ [Auth] User registered successfully: ID={}, Email={}",
            create_user.user_id, input.email
        );

        Ok(ApiResponse {
            status: "success".to_string(),
            message: "User registered successfully".to_string(),
            data: UserResponse::from(create_user),
        })
    }

    async fn login_user(&self, input: &LoginRequest) -> Result<ApiResponse<String>, ErrorResponse> {
        info!("🔐 [Auth] Login attempt for user: {}", input.email);

        let user = match self.repository.find_by_email(&input.email).await {
            Ok(Some(user)) => {
                info!(
                    "👤 [Auth] User found: ID={}, Email={}",
                    user.user_id, input.email
                );
                user
            }
            Ok(None) => {
                error!("❌ [Auth] Login failed: User not found: {}", input.email);
                return Err(ErrorResponse::from(AppError::NotFound(
                    "User not found".to_string(),
                )));
            }
            Err(err) => {
                error!(
                    "❌ [Auth] Database error during login for {}: {}",
                    input.email, err
                );
                return Err(ErrorResponse::from(err));
            }
        };

        if self
            .hashing
            .compare_password(&user.password, &input.password)
            .await
            .is_err()
        {
            error!("⛔ [Auth] Invalid credentials for user: {}", input.email);
            return Err(ErrorResponse::from(AppError::InvalidCredentials));
        }

        let token = self
            .jwt_config
            .generate_token(user.user_id as i64)
            .map_err(|e| {
                error!(
                    "❌ [Auth] Failed to generate JWT token for user {}: {}",
                    input.email, e
                );
                ErrorResponse::from(e)
            })?;

        info!("✅ [Auth] Login successful for user: {}", input.email);

        Ok(ApiResponse {
            status: "success".to_string(),
            message: "Login successful".to_string(),
            data: token,
        })
    }
}
