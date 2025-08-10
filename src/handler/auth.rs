use axum::{
    Extension, Json,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::{Value, json};
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    abstract_trait::{DynAuthService, DynUserService},
    domain::{
        request::{LoginRequest, RegisterRequest},
        response::{ApiResponse, user::UserResponse},
    },
    middleware::{jwt, validation::SimpleValidatedJson},
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Login successful", body = ApiResponse<UserResponse>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Auth"
)]
pub async fn register_user_handler(
    Extension(service): Extension<DynAuthService>,
    SimpleValidatedJson(body): SimpleValidatedJson<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    match service.register_user(&body).await {
        Ok(response) => Ok((StatusCode::OK, Json(json!(response)))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!(e)))),
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Auth"
)]
pub async fn login_user_handler(
    Extension(service): Extension<DynAuthService>,
    SimpleValidatedJson(body): SimpleValidatedJson<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    match service.login_user(&body).await {
        Ok(response) => Ok((StatusCode::OK, Json(json!(response)))),
        Err(e) => Err((StatusCode::UNAUTHORIZED, Json(json!(e)))),
    }
}

#[utoipa::path(
    get,
    path = "/api/auth/me",
    responses(
        (status = 200, description = "Get Me user", body = ApiResponse<UserResponse>)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Auth",
)]
pub async fn get_me_handler(
    Extension(service): Extension<DynUserService>,
    Extension(user_id): Extension<i32>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match service.get_user(user_id).await {
        Ok(response) => Ok((StatusCode::OK, Json(json!(response)))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": e.message
            })),
        )),
    }
}

pub fn auth_routes(app_state: Arc<AppState>) -> OpenApiRouter {
    let public_routes = OpenApiRouter::new()
        .route("/api/auth/register", post(register_user_handler))
        .route("/api/auth/login", post(login_user_handler))
        .layer(Extension(app_state.di_container.auth_service.clone()));

    let private_routes = OpenApiRouter::new()
        .route("/api/auth/me", get(get_me_handler))
        .route_layer(middleware::from_fn(jwt::auth))
        .layer(Extension(app_state.di_container.user_service.clone()))
        .layer(Extension(app_state.jwt_service.clone()));

    public_routes.merge(private_routes)
}
