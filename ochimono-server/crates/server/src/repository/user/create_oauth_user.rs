use entity::users::{ActiveModel as UserActiveModel, Model as UserModel};
use errors::errors::Errors;
use sea_orm::{ActiveModelTrait, ConnectionTrait, DbErr, Set, SqlErr};
use uuid::Uuid;

/// Map an insert failure to a meaningful 409 when it is a unique-constraint
/// violation (a concurrent signup won the race after the in-transaction
/// re-checks), instead of leaking it as a generic 500.
fn map_create_user_db_err(err: DbErr) -> Errors {
    match err.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(detail)) if detail.contains("handle") => {
            Errors::UserHandleAlreadyExists
        }
        Some(SqlErr::UniqueConstraintViolation(_)) => Errors::OauthAccountAlreadyLinked,
        _ => Errors::DatabaseError(err.to_string()),
    }
}

/// OAuth를 통한 새 유저 생성 (비밀번호 없음)
pub async fn repository_create_oauth_user<C>(
    conn: &C,
    provider_user_id: &str,
    email: &str,
    handle: &str,
) -> Result<UserModel, Errors>
where
    C: ConnectionTrait,
{
    let new_user = UserActiveModel {
        id: Set(Uuid::now_v7()),
        google_subject: Set(provider_user_id.to_owned()),
        email: Set(email.to_owned()),
        handle: Set(handle.to_owned()),
        ..Default::default()
    };
    let user = new_user
        .insert(conn)
        .await
        .map_err(map_create_user_db_err)?;
    Ok(user)
}
