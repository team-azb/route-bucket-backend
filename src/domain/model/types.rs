use std::convert::TryFrom;

use derive_more::{Add, AddAssign, Display, From, Into, Sub, Sum};
use nanoid::nanoid;
use num_traits::{Bounded, FromPrimitive};
use serde::{Deserialize, Serialize};

use crate::utils::error::{ApplicationError, ApplicationResult};

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

#[derive(Display, From, Into, Debug, Clone, Serialize, Deserialize)]
pub struct Polyline(String);

impl Polyline {
    pub fn new() -> Self {
        Self(String::new())
    }
}

pub type Latitude = NumericValueObject<f64, 90>;
pub type Longitude = NumericValueObject<f64, 180>;
// NOTE: genericsの特殊化が実装されたら、この0は消せる
// 参考: https://github.com/rust-lang/rust/issues/31844
pub type Elevation = NumericValueObject<i32, 1000000>;
pub type Distance = NumericValueObject<f64, 0>;

/// Value Object for BigDecimal type
#[derive(
    Add,
    AddAssign,
    Sub,
    Sum,
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
)]
pub struct NumericValueObject<T, const MAX_ABS: u32>(T);

impl<T: Copy + FromPrimitive + Bounded, const MAX_ABS: u32> NumericValueObject<T, MAX_ABS> {
    pub fn value(&self) -> T {
        self.0
    }

    pub fn max() -> Self {
        Self(if MAX_ABS == 0 {
            T::max_value()
        } else {
            T::from_u32(MAX_ABS).unwrap()
        })
    }

    pub fn zero() -> Self {
        Self(T::from_u32(0).unwrap())
    }
}

// NOTE: TryFrom<T>を実装しようとすると、coreのimplとconflictする
// これも特殊化待ち: https://github.com/rust-lang/rust/issues/31844
impl<const MAX_ABS: u32> TryFrom<f64> for NumericValueObject<f64, MAX_ABS> {
    type Error = ApplicationError;

    fn try_from(val: f64) -> ApplicationResult<Self> {
        if MAX_ABS == 0 || val.abs() <= MAX_ABS.into() {
            Ok(Self(val))
        } else {
            Err(ApplicationError::ValueObjectError(format!(
                // TODO: stringのconst genericsが追加されたら、
                // メッセージに具体的なエイリアス名(Latitudeとか)を入れる
                "Invalid value {} for NumericValueObject<f64, {}>",
                val,
                MAX_ABS
            )))
        }
    }
}

// NOTE: TryFrom<T>を実装しようとすると、coreのimplとconflictする
impl<const MAX_ABS: u32> TryFrom<i32> for NumericValueObject<i32, MAX_ABS> {
    type Error = ApplicationError;

    fn try_from(val: i32) -> ApplicationResult<Self> {
        if MAX_ABS == 0 || val.abs() <= MAX_ABS as i32 {
            Ok(Self(val))
        } else {
            Err(ApplicationError::ValueObjectError(format!(
                // TODO: stringのconst genericsが追加されたら、
                // メッセージに具体的なエイリアス名(Latitudeとか)を入れる
                "Invalid value {} for NumericValueObject<i32, {}>",
                val,
                MAX_ABS
            )))
        }
    }
}
