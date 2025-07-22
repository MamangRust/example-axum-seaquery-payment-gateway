use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::{Value, json};
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;

use crate::{
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
    State(data): State<Arc<AppState>>,
    SimpleValidatedJson(body): SimpleValidatedJson<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    match data.di_container.auth_service.register_user(&body).await {
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
    State(data): State<Arc<AppState>>,
    SimpleValidatedJson(body): SimpleValidatedJson<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    match data.di_container.auth_service.login_user(&body).await {
        Ok(response) => Ok((StatusCode::OK, Json(json!(response)))),
        Err(e) => Err((StatusCode::UNAUTHORIZED, Json(json!(e)))),
    }
}

#[utoipa::path(
    get,
    path = "/api/users/me",
    responses(
        (status = 200, description = "Get Me user", body = ApiResponse<UserResponse>)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Auth",
)]
pub async fn get_me_handler(
    State(data): State<Arc<AppState>>,
    Extension(user_id): Extension<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match data
        .di_container
        .user_service
        .get_user(user_id as i32)
        .await
    {
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
        .route("/api/auth/login", post(login_user_handler));

    let private_routes = OpenApiRouter::new()
        .route("/api/auth/me", get(get_me_handler))
        .route_layer(middleware::from_fn_with_state(app_state.clone(), jwt::auth));

    public_routes.merge(private_routes).with_state(app_state)
}
