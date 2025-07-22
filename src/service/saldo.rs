use async_trait::async_trait;
use tracing::{error, info};

use crate::{
    abstract_trait::{DynSaldoRepository, DynUserRepository, SaldoServiceTrait},
    domain::{
        request::{CreateSaldoRequest, FindAllSaldoRequest, UpdateSaldoRequest},
        response::{
            ApiResponse, ApiResponsePagination, ErrorResponse, pagination::Pagination,
            saldo::SaldoResponse,
        },
    },
    utils::AppError,
};

pub struct SaldoService {
    user_repository: DynUserRepository,
    saldo_repository: DynSaldoRepository,
}

impl SaldoService {
    pub fn new(user_repository: DynUserRepository, saldo_repository: DynSaldoRepository) -> Self {
        Self {
            user_repository,
            saldo_repository,
        }
    }
}

#[async_trait]
impl SaldoServiceTrait for SaldoService {
    async fn get_saldos(
        &self,
        req: &FindAllSaldoRequest,
    ) -> Result<ApiResponsePagination<Vec<SaldoResponse>>, ErrorResponse> {
        let page = if req.page > 0 { req.page } else { 1 };
        let page_size = if req.page_size > 0 { req.page_size } else { 10 };
        let search = if req.search.is_empty() {
            None
        } else {
            Some(req.search.clone())
        };

        let (saldos, total_items) = self
            .saldo_repository
            .find_all(page, page_size, search)
            .await?;

        info!("Found {} saldos", saldos.len());

        let total_pages = (total_items as f64 / page_size as f64).ceil() as i32;

        let saldo_responses: Vec<SaldoResponse> =
            saldos.into_iter().map(SaldoResponse::from).collect();

        Ok(ApiResponsePagination {
            status: "success".to_string(),
            message: "Saldos retrieved successfully".to_string(),
            data: saldo_responses,
            pagination: Pagination {
                page,
                page_size,
                total_items,
                total_pages,
            },
        })
    }

    async fn get_saldo(
        &self,
        id: i32,
    ) -> Result<ApiResponse<Option<SaldoResponse>>, ErrorResponse> {
        let saldo = self.saldo_repository.find_by_id(id).await?;

        if let Some(saldo) = saldo {
            Ok(ApiResponse {
                status: "success".to_string(),
                message: "Saldo retrieved successfully".to_string(),
                data: Some(SaldoResponse::from(saldo)),
            })
        } else {
            Err(ErrorResponse::from(AppError::NotFound(format!(
                "Saldo with id {id} not found",
            ))))
        }
    }

    async fn get_saldo_users(
        &self,
        id: i32,
    ) -> Result<ApiResponse<Option<Vec<SaldoResponse>>>, ErrorResponse> {
        let _user = self.user_repository.find_by_id(id).await.map_err(|_| {
            ErrorResponse::from(AppError::NotFound(format!("User with id {id} not found")))
        })?;

        let saldo = self.saldo_repository.find_by_users_id(id).await?;

        let saldo_responses = if saldo.is_empty() {
            None
        } else {
            Some(saldo.into_iter().map(SaldoResponse::from).collect())
        };

        if saldo_responses.is_none() {
            let response = ApiResponse {
                status: "success".to_string(),
                data: None,
                message: format!("No saldo found for user with id {id}"),
            };

            return Ok(response);
        }

        let response = ApiResponse {
            status: "success".to_string(),
            data: saldo_responses,
            message: "Success".to_string(),
        };

        Ok(response)
    }

    async fn get_saldo_user(
        &self,
        id: i32,
    ) -> Result<ApiResponse<Option<SaldoResponse>>, ErrorResponse> {
        let _user = self.user_repository.find_by_id(id).await.map_err(|_| {
            ErrorResponse::from(AppError::NotFound(format!("User with id {id} not found")))
        })?;

        let saldo: Option<SaldoResponse> = self
            .saldo_repository
            .find_by_user_id(id)
            .await?
            .map(SaldoResponse::from);

        match saldo {
            Some(saldo) => {
                info!("Saldo found for user_id: {id}");

                Ok(ApiResponse {
                    status: "success".to_string(),
                    message: "Saldo retrieved successfully".to_string(),
                    data: Some(saldo),
                })
            }
            None => {
                error!("No saldo found for user_id: {id}");
                Err(ErrorResponse::from(AppError::NotFound(format!(
                    "No saldo found for user_id: {id}",
                ))))
            }
        }
    }

    async fn create_saldo(
        &self,
        input: &CreateSaldoRequest,
    ) -> Result<ApiResponse<SaldoResponse>, ErrorResponse> {
        let _user = self
            .user_repository
            .find_by_id(input.user_id)
            .await
            .map_err(|_| {
                ErrorResponse::from(AppError::NotFound(format!(
                    "User with id {} not found",
                    input.user_id
                )))
            })?;

        info!("Saldo created successfully for user_id: {}", input.user_id);

        let saldo = self.saldo_repository.create(input).await?;

        Ok(ApiResponse {
            status: "success".to_string(),
            message: "Saldo created successfully".to_string(),
            data: SaldoResponse::from(saldo),
        })
    }

    async fn update_saldo(
        &self,
        input: &UpdateSaldoRequest,
    ) -> Result<ApiResponse<Option<SaldoResponse>>, ErrorResponse> {
        let _user = self
            .user_repository
            .find_by_id(input.user_id)
            .await
            .map_err(|_| {
                ErrorResponse::from(AppError::NotFound(format!(
                    "User with id {} not found",
                    input.user_id
                )))
            })?;

        let existing_saldo = self.saldo_repository.find_by_id(input.saldo_id).await?;

        match existing_saldo {
            Some(_) => {
                let updated_saldo = self.saldo_repository.update(input).await?;

                info!("Saldo updated successfully for id: {}", input.saldo_id);

                Ok(ApiResponse {
                    status: "success".to_string(),
                    message: "Saldo updated successfully".to_string(),
                    data: Some(SaldoResponse::from(updated_saldo)),
                })
            }
            None => {
                error!("Saldo with id {} not found", input.saldo_id);
                Err(ErrorResponse::from(AppError::NotFound(format!(
                    "Saldo with id {} not found",
                    input.saldo_id
                ))))
            }
        }
    }

    async fn delete_saldo(&self, id: i32) -> Result<ApiResponse<()>, ErrorResponse> {
        let user = self.user_repository.find_by_id(id).await.map_err(|_| {
            ErrorResponse::from(AppError::NotFound(format!("User with id {id} not found")))
        })?;

        let existing_saldo = self
            .saldo_repository
            .find_by_user_id(user.unwrap().user_id)
            .await?;

        match existing_saldo {
            Some(_) => {
                self.saldo_repository
                    .delete(existing_saldo.unwrap().saldo_id)
                    .await?;

                info!("Saldo deleted successfully for id: {id}");

                Ok(ApiResponse {
                    status: "success".to_string(),
                    message: "Saldo deleted successfully".to_string(),
                    data: (),
                })
            }
            None => {
                error!("Saldo with id {id} not found");
                Err(ErrorResponse::from(AppError::NotFound(format!(
                    "Saldo with id {id} not found",
                ))))
            }
        }
    }
}
