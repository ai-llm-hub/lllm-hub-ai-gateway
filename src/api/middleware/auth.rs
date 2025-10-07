use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::domain::entities::project::Project;
use crate::domain::repositories::project_repository::ProjectRepository;
use crate::shared::error::AppError;

/// Authentication middleware for Project API keys
pub async fn authenticate(
    State(repo): State<Arc<dyn ProjectRepository>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::AuthenticationError("Missing Authorization header".to_string()))?;

    // Extract Bearer token
    let api_key = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            AppError::AuthenticationError("Invalid Authorization header format".to_string())
        })?
        .trim();

    if api_key.is_empty() {
        return Err(AppError::AuthenticationError(
            "Empty API key".to_string(),
        ));
    }

    // Validate API key format (should start with pk_)
    if !api_key.starts_with("pk_") {
        return Err(AppError::AuthenticationError(
            "Invalid API key format".to_string(),
        ));
    }

    // Fetch project from database
    let project = repo.find_by_api_key(api_key).await?;

    // Check if project is active
    if !project.is_active() {
        return Err(AppError::AuthorizationError(
            "Project is not active".to_string(),
        ));
    }

    // Store project in request extensions for handlers to use
    req.extensions_mut().insert(project);

    Ok(next.run(req).await)
}

/// Extract project from request extensions
pub fn extract_project(req: &Request) -> Result<&Project, AppError> {
    req.extensions()
        .get::<Project>()
        .ok_or_else(|| AppError::InternalError("Project not found in request".to_string()))
}