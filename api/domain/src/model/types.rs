use std::{convert::TryFrom, marker::PhantomData};

use derive_more::{Display, Into};
use nanoid::nanoid;
use route_bucket_utils::ApplicationError;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Default, Debug, Clone, Display, Serialize, Deserialize, PartialEq, Eq)]
#[display(fmt = "{}", id)]
#[serde(transparent)]
pub struct NanoId<T, const LEN: usize> {
    id: String,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T, const LEN: usize> NanoId<T, LEN> {
    pub fn new() -> Self {
        Self::from_string(nanoid!(LEN))
    }
    pub fn from_string(id: String) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }
}

#[derive(Clone, Debug, Validate, Display, Into, Serialize, Deserialize)]
#[display(fmt = "{}", value)]
#[serde(transparent)]
pub struct Email {
    #[validate(email)]
    value: String,
}

impl TryFrom<String> for Email {
    type Error = ApplicationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let email = Email { value };
        email.validate().map_err(|e| {
            ApplicationError::DomainError(format!("Validation failed for Email! ({:?})", e))
        })?;
        Ok(email)
    }
}

#[derive(Clone, Debug, Validate, Display, Into, Serialize, Deserialize)]
#[display(fmt = "{}", value)]
#[serde(transparent)]
pub struct Url {
    #[validate(url)]
    value: String,
}

impl TryFrom<String> for Url {
    type Error = ApplicationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let url = Url { value };
        url.validate().map_err(|e| {
            ApplicationError::DomainError(format!("Validation failed for Url! ({:?})", e))
        })?;
        Ok(url)
    }
}
