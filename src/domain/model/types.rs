use derive_more::{Add, AddAssign, Display, Sub};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::utils::error::{ApplicationError, ApplicationResult};
use num_traits::FromPrimitive;
use std::convert::TryFrom;

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

#[derive(Display, Debug, Clone, Serialize, Deserialize)]
pub struct Polyline(String);

impl Polyline {
    pub fn new() -> Self {
        Self(String::new())
    }
}

impl From<String> for Polyline {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<Polyline> for String {
    fn from(value: Polyline) -> Self {
        value.0
    }
}

pub type Latitude = NumericValueObject<f64, 90>;
pub type Longitude = NumericValueObject<f64, 180>;
pub type Elevation = NumericValueObject<i32, { i32::MAX as u32 }>;

/// Value Object for BigDecimal type
#[derive(
    Add, AddAssign, Sub, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct NumericValueObject<T, const MAX_ABS: u32>(T);

impl<T: Copy + FromPrimitive, const MAX_ABS: u32> NumericValueObject<T, MAX_ABS> {
    pub fn value(&self) -> T {
        self.0
    }

    pub fn max() -> Self {
        Self(T::from_u32(MAX_ABS).unwrap())
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

impl<const MAX_ABS: u32> TryFrom<i32> for NumericValueObject<i32, MAX_ABS> {
    type Error = ApplicationError;

    fn try_from(val: i32) -> ApplicationResult<Self> {
        if val.abs() <= MAX_ABS as i32 {
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
