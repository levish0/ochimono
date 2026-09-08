use crate::errors::Errors;
use crate::protocol::system::*;
use axum::http::StatusCode;
use tracing::{error, warn};

/// 시스템 관련 에러 로깅 처리
pub fn log_error(err: &Errors) {
    match err {
        // 시스템 심각도 에러 - error! 레벨
        Errors::SysInternalError(_) | Errors::DatabaseError(_) | Errors::TransactionError(_) => {
            error!(error=?err,"System error occurred");
        }
        Errors::NotFound(_) => {
            warn!(error=?err,"Resource not found");
        }
        Errors::AuthUnavailable => {
            warn!("Authentication unavailable");
        }
        _ => {}
    }
}
/// Returns: (StatusCode, error_code, details)
pub fn map_response(error: &Errors) -> Option<(StatusCode, &'static str, Option<String>)> {
    match error {
        Errors::SysInternalError(msg) => Some((
            StatusCode::INTERNAL_SERVER_ERROR,
            SYS_INTERNAL_ERROR,
            Some(msg.clone()),
        )),
        Errors::DatabaseError(msg) => Some((
            StatusCode::INTERNAL_SERVER_ERROR,
            SYS_DATABASE_ERROR,
            Some(msg.clone()),
        )),
        Errors::TransactionError(msg) => Some((
            StatusCode::INTERNAL_SERVER_ERROR,
            SYS_TRANSACTION_ERROR,
            Some(msg.clone()),
        )),
        Errors::NotFound(msg) => Some((StatusCode::NOT_FOUND, SYS_NOT_FOUND, Some(msg.clone()))),
        Errors::AuthUnavailable => {
            Some((StatusCode::SERVICE_UNAVAILABLE, "auth:unavailable", None))
        }
        _ => None, // 다른 도메인의 에러는 None 반환
    }
}
