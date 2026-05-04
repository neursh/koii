use axum::{ extract::Request, middleware::Next, response::IntoResponse };

pub async fn log_requests(request: Request, next: Next) -> impl IntoResponse {
    tracing::info!("{} {}", request.method(), request.uri());

    next.run(request).await
}
