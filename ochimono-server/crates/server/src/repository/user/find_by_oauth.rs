use entity::users::{Column as UserColumn, Entity as UserEntity, Model as UserModel};
use errors::errors::Errors;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

/// OAuth provider의 사용자 ID로 유저 조회
pub async fn repository_find_user_by_oauth<C>(
    conn: &C,
    provider_user_id: &str,
) -> Result<Option<UserModel>, Errors>
where
    C: ConnectionTrait,
{
    let user = UserEntity::find()
        .filter(UserColumn::GoogleSubject.eq(provider_user_id))
        .one(conn)
        .await?;
    Ok(user)
}
