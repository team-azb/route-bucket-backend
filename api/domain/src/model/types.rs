use std::marker::PhantomData;

use derive_more::Display;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

#[derive(Display, Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
