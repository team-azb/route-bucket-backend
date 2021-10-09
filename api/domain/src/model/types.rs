use std::convert::TryFrom;

use derive_more::{Add, AddAssign, Display, From, Into, Sub, Sum};
use nanoid::nanoid;
use num_traits::{Bounded, FromPrimitive};
use serde::{Deserialize, Serialize};

use route_bucket_utils::{ApplicationError, ApplicationResult};

#[derive(Display, Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NanoId<const LEN: usize>(String);

impl<const LEN: usize> NanoId<LEN> {
    pub fn new() -> Self {
        Self(nanoid!(LEN))
    }
    pub fn from_string(id: String) -> Self {
        Self(id)
    }
}

// TODO: Make this an derive macro (ex: #[derive(NanoId)])
pub type RouteId = NanoId<11>;
pub type SegmentId = NanoId<21>;
pub type OperationId = NanoId<21>;

#[derive(Default, Display, From, Into, Debug, Clone, Serialize, Deserialize, PartialEq)]
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

    pub fn max_value() -> Self {
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

impl<const MAX_ABS: u32> NumericValueObject<f64, MAX_ABS> {
    pub fn min(a: Self, b: Self) -> Self {
        if a.0 > b.0 {
            b
        } else {
            a
        }
    }

    pub fn max(a: Self, b: Self) -> Self {
        if a.0 < b.0 {
            b
        } else {
            a
        }
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
