use entity::users::{Entity as UserEntity, Model as UserModel};
use errors::errors::Errors;
use sea_orm::{ConnectionTrait, EntityTrait};
use uuid::Uuid;

/// 사용자 ID로 사용자 존재 여부를 조회한다.
///
/// # 역할
/// ID 단건 조회를 수행하고 `Option<UserModel>`을 반환한다.
///
/// # Errors
/// - 조회 실패 시 DB/저장소 에러를 반환한다.
pub async fn repository_find_user_by_id<C>(conn: &C, id: Uuid) -> Result<Option<UserModel>, Errors>
where
    C: ConnectionTrait,
{
    let user = UserEntity::find_by_id(id).one(conn).await?;

    Ok(user)
}
