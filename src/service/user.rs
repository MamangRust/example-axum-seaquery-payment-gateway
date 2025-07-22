use async_trait::async_trait;
use tracing::{error, info};

use crate::{
    abstract_trait::{DynHashing, DynUserRepository, UserServiceTrait},
    domain::{
        request::{CreateUserRequest, FindAllUserRequest, RegisterRequest, UpdateUserRequest},
        response::{
            ApiResponse, ApiResponsePagination, ErrorResponse, pagination::Pagination,
            user::UserResponse,
        },
    },
    utils::{AppError, random_vcc},
};

pub struct UserService {
    repository: DynUserRepository,
    hashing: DynHashing,
}

impl UserService {
    pub fn new(repository: DynUserRepository, hashing: DynHashing) -> Self {
        Self {
            repository,
            hashing,
        }
    }
}

#[async_trait]
impl UserServiceTrait for UserService {
    async fn get_users(
        &self,
        req: &FindAllUserRequest,
    ) -> Result<ApiResponsePagination<Vec<UserResponse>>, ErrorResponse> {
        let page = if req.page > 0 { req.page } else { 1 };
        let page_size = if req.page_size > 0 { req.page_size } else { 10 };
        let search = if req.search.is_empty() {
            None
        } else {
            Some(req.search.clone())
        };

        let (users, total_items) = self.repository.find_all(page, page_size, search).await?;

        info!("Found {} users", users.len());

        let total_pages = (total_items as f64 / page_size as f64).ceil() as i32;

        let user_responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();

        Ok(ApiResponsePagination {
            status: "success".to_string(),
            message: "Users retrieved successfully".to_string(),
            data: user_responses,
            pagination: Pagination {
                page,
                page_size,
                total_items,
                total_pages,
            },
        })
    }

    async fn get_user(&self, id: i32) -> Result<ApiResponse<Option<UserResponse>>, ErrorResponse> {
        let user = self.repository.find_by_id(id).await?;

        if let Some(user) = user {
            Ok(ApiResponse {
                status: "success".to_string(),
                message: "User retrieved successfully".to_string(),
                data: Some(UserResponse::from(user)),
            })
        } else {
            Err(ErrorResponse::from(AppError::NotFound(format!(
                "User with id {id} not found",
            ))))
        }
    }

    async fn create_user(
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
            .map_err(|e| ErrorResponse::from(AppError::HashingError(e)))?;

        let noc_transfer = random_vcc().map(Some).unwrap_or(None);

        let request = &CreateUserRequest {
            firstname: input.firstname.clone(),
            lastname: input.lastname.clone(),
            email: input.email.clone(),
            password: hashed_password,
            confirm_password: input.confirm_password.clone(),
            noc_transfer: noc_transfer.to_owned(),
        };

        info!("Creating user with email: {}", input.email);
        let create_user = self.repository.create_user(request).await?;

        info!("User Create successfully with email: {}", input.email);

        Ok(ApiResponse {
            status: "success".to_string(),
            message: "User Create successfully".to_string(),
            data: UserResponse::from(create_user),
        })
    }

    async fn update_user(
        &self,
        input: &UpdateUserRequest,
    ) -> Result<Option<ApiResponse<UserResponse>>, ErrorResponse> {
        let user = self.repository.update_user(input).await?;

        Ok(Some(ApiResponse {
            status: "success".to_string(),
            message: "User updated successfully".to_string(),
            data: UserResponse::from(user),
        }))
    }

    async fn delete_user(&self, id: i32) -> Result<ApiResponse<()>, ErrorResponse> {
        self.repository.delete_user(id).await?;

        Ok(ApiResponse {
            status: "success".to_string(),
            message: "User deleted successfully".to_string(),
            data: (),
        })
    }
}
