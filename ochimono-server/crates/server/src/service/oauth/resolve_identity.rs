use super::state::{PendingSignupData, SignIn, key, store};
use crate::{
    repository::user as repository,
    service::auth::{AuthState, session::SessionService},
};
use auth_core::token::generate_secure_token;
use dto::auth::{LoginResponse, UserResponse};
use errors::errors::{Errors, ServiceResult};
use sea_orm::TransactionTrait;

pub async fn service_resolve_google_identity(
    state: &AuthState,
    browser: &str,
    subject: &str,
    email: &str,
    persistent: bool,
) -> ServiceResult<SignIn> {
    if let Some(discovered) = repository::repository_find_user_by_oauth(&state.db, subject).await? {
        let transaction = state.db.begin().await?;
        let user =
            repository::repository_get_user_by_id_for_update(&transaction, discovered.id).await?;
        if user.google_subject != subject {
            return Err(Errors::UserUnauthorized);
        }
        let issued = SessionService::create_session(state, &user, persistent).await?;
        transaction.commit().await?;
        return Ok((
            LoginResponse::SignedIn {
                user: UserResponse {
                    id: user.id,
                    handle: user.handle,
                },
            },
            Some(issued),
        ));
    }
    let token = generate_secure_token();
    store(
        state,
        key("signup", &token, browser),
        &PendingSignupData {
            subject: subject.into(),
            email: email.into(),
            remember_me: persistent,
        },
    )
    .await?;
    Ok((
        LoginResponse::PendingSignup {
            pending_token: token,
        },
        None,
    ))
}
