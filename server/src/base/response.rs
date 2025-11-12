use axum::{ Json, http::StatusCode };
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub struct ResponseModel<R = Value> {
  success: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  error: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  result: Option<R>,
}

pub fn success<R>(status: StatusCode) -> (StatusCode, Json<ResponseModel<R>>) {
  (
    status,
    Json(ResponseModel {
      success: true,
      error: None,
      result: None,
    }),
  )
}

pub fn result<R>(status: StatusCode, result: R) -> (StatusCode, Json<ResponseModel<R>>) {
  (
    status,
    Json(ResponseModel {
      success: true,
      error: None,
      result: Some(result),
    }),
  )
}

pub fn internal_error<R>() -> (StatusCode, Json<ResponseModel<R>>) {
  (
    StatusCode::INTERNAL_SERVER_ERROR,
    Json(ResponseModel {
      success: false,
      error: Some("Something went wrong while processing your request.".to_string()),
      result: None,
    }),
  )
}

pub fn error<R>(status: StatusCode, details: &str) -> (StatusCode, Json<ResponseModel<R>>) {
  (
    status,
    Json(ResponseModel {
      success: false,
      error: Some(details.to_string()),
      result: None,
    }),
  )
}
