use super::state::{PendingSignupData, SignIn, consume, key};
use crate::{
    repository::user as repository,
    service::auth::{AuthState, session::SessionService},
};
use dto::auth::{LoginResponse, UserResponse};
use errors::errors::{Errors, ServiceResult};
use sea_orm::TransactionTrait;

pub fn valid_handle(handle: &str) -> bool {
    (3..=24).contains(&handle.len())
        && handle
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
}

pub async fn service_complete_signup(
    state: &AuthState,
    browser: &str,
    token: &str,
    handle: &str,
) -> ServiceResult<SignIn> {
    if !valid_handle(handle) {
        return Err(Errors::BadRequestError("Invalid handle".into()));
    }
    let proof: PendingSignupData = consume(state, key("signup", token, browser)).await?;
    let transaction = state.db.begin().await?;
    // A provider subject is the identity key. An email match never links another account.
    let user = repository::repository_create_oauth_user(
        &transaction,
        &proof.subject,
        &proof.email,
        handle,
    )
    .await?;
    let issued = SessionService::create_session(state, &user, proof.remember_me).await?;
    transaction.commit().await?;
    Ok((
        LoginResponse::SignedIn {
            user: UserResponse {
                id: user.id,
                handle: user.handle,
            },
        },
        Some(issued),
    ))
}
