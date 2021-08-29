use route_bucket_domain::model::{User, UserId};

/// 座標のdto構造体
#[derive(sqlx::FromRow)]
pub(crate) struct UserDto {
    pub(crate) id: String,
}

impl From<User> for UserDto {
    fn from(model: User) -> Self {
        let id = UserId::from(model).to_string();
        Self { id }
    }
}

impl From<UserDto> for User {
    fn from(dto: UserDto) -> Self {
        UserId::from_string(dto.id).into()
    }
}
