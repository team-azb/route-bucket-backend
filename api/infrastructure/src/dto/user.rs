use std::{convert::TryFrom, str::FromStr};

use chrono::NaiveDate;
use getset::Getters;
use route_bucket_domain::model::{
    types::Url,
    user::{Gender, User, UserId},
};
use route_bucket_utils::{ApplicationError, ApplicationResult};

/// ルートのdto構造体
#[derive(sqlx::FromRow, Getters)]
#[get = "pub"]
pub(crate) struct UserDto {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) gender: String,
    pub(crate) birthdate: Option<NaiveDate>,
    pub(crate) icon_url: Option<String>,
}

impl UserDto {
    pub fn into_model(self) -> ApplicationResult<User> {
        let Self {
            id,
            name,
            gender,
            birthdate,
            icon_url,
        } = self;
        Ok(User::new(
            UserId::from(id),
            name,
            Gender::from_str(&gender).map_err(|e| {
                ApplicationError::DataBaseError(format!("Failed to parse user.gender ({:?})", e))
            })?,
            birthdate,
            icon_url.map(Url::try_from).transpose()?,
        ))
    }

    pub fn from_model(user: &User) -> ApplicationResult<Self> {
        let (id, name, gender, birthdate, icon_url) = user.clone().into();
        Ok(Self {
            id: id.into(),
            name,
            gender: gender.to_string(),
            birthdate,
            icon_url: icon_url.map(String::from),
        })
    }
}
