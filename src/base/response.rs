use axum::{ Json, http::{ HeaderName, StatusCode }, response::AppendHeaders };
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub struct ResponseBody<R = Value> {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<R>,
}

pub type ResponseModel<R = Value> = (
    StatusCode,
    Option<AppendHeaders<Vec<(HeaderName, String)>>>,
    Json<ResponseBody<R>>,
);

pub fn success<R>(
    status: StatusCode,
    headers: Option<AppendHeaders<Vec<(HeaderName, String)>>>
) -> ResponseModel<R> {
    (
        status,
        headers,
        Json(ResponseBody {
            success: true,
            error: None,
            result: None,
        }),
    )
}

pub fn result<R>(
    status: StatusCode,
    result: R,
    headers: Option<AppendHeaders<Vec<(HeaderName, String)>>>
) -> ResponseModel<R> {
    (
        status,
        headers,
        Json(ResponseBody {
            success: true,
            error: None,
            result: Some(result),
        }),
    )
}

pub fn internal_error<R>(
    headers: Option<AppendHeaders<Vec<(HeaderName, String)>>>
) -> ResponseModel<R> {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        headers,
        Json(ResponseBody {
            success: false,
            error: Some("Something went wrong while processing your request.".to_string()),
            result: None,
        }),
    )
}

pub fn error<R>(
    status: StatusCode,
    details: &str,
    headers: Option<AppendHeaders<Vec<(HeaderName, String)>>>
) -> ResponseModel<R> {
    (
        status,
        headers,
        Json(ResponseBody {
            success: false,
            error: Some(details.to_string()),
            result: None,
        }),
    )
}
