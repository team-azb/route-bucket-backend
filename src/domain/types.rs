use bigdecimal::BigDecimal;
use derive_more::Display;
use nanoid::nanoid;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

use crate::utils::error::{ApplicationError, ApplicationResult};
use num::Signed;
use std::convert::TryFrom;
use std::fmt::Display;

// TODO: Value Object用のderive macroを作る
// ↓みたいな一要素のタプル構造体たちにfrom, valueをデフォルトで実装したい
// ただのgenericsでSelf(val)やself.0.clone()をしようとすると怒られるので、
// derive macro + traitでやるしかなさそう

#[derive(Display, Debug, Clone, Serialize, Deserialize)]
pub struct RouteId(String);

impl RouteId {
    pub fn new() -> RouteId {
        RouteId(nanoid!(11))
    }
    pub fn from_string(id: String) -> Self {
        Self(id)
    }
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

pub type Latitude = NumericValueObject<f64, 90>;
pub type Longitude = NumericValueObject<f64, 180>;

/// Value Object for BigDecimal type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NumericValueObject<T, const MAX_ABS: u32>(T);

impl<T: Copy, const MAX_ABS: u32> NumericValueObject<T, MAX_ABS> {
    pub fn value(&self) -> T {
        self.0
    }
}

// NOTE: TryFrom<T>を実装しようとすると、coreのimplとconflictする
impl<const MAX_ABS: u32> TryFrom<f64> for NumericValueObject<f64, MAX_ABS> {
    type Error = ApplicationError;

    fn try_from(val: f64) -> ApplicationResult<Self> {
        if val.abs() <= MAX_ABS.into() {
            Ok(Self(val))
        } else {
            Err(ApplicationError::ValueObjectError(format!(
                // TODO: stringのconst genericsが追加されたら、
                // メッセージに具体的なエイリアス名(Latitudeとか)を入れる
                "Invalid value {} for BigDecimalValueObject<{}>",
                val,
                MAX_ABS
            )))
        }
    }
}
