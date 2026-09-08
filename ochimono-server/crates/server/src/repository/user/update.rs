use entity::users::{ActiveModel, Model};
use errors::errors::Errors;
use sea_orm::{ActiveModelTrait, ConnectionTrait, IntoActiveModel, Set};

/// Advance the durable security generation while holding the user's row lock.
pub async fn repository_advance_security_generation<C>(conn: &C, user: Model) -> Result<(), Errors>
where
    C: ConnectionTrait,
{
    let generation = user
        .security_generation
        .checked_add(1)
        .ok_or_else(|| Errors::SysInternalError("Security generation exhausted".into()))?;
    let mut user: ActiveModel = user.into_active_model();
    user.security_generation = Set(generation);
    user.update(conn).await?;
    Ok(())
}
