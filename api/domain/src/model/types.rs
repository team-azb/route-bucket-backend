use std::convert::TryFrom;
use std::marker::PhantomData;

use derive_more::{Add, AddAssign, Display, From, Into, Sub, Sum};
use nanoid::nanoid;
use num_traits::{Bounded, FromPrimitive};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use route_bucket_utils::{ApplicationError, ApplicationResult};

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

#[derive(Default, Display, From, Into, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Polyline(String);

impl Polyline {
    pub fn new() -> Self {
        Self(String::new())
    }
}

pub type Latitude = NumericValueObject<OrderedFloat<f64>, 90>;
pub type Longitude = NumericValueObject<OrderedFloat<f64>, 180>;
// NOTE: genericsの特殊化が実装されたら、この0は消せる
// 参考: https://github.com/rust-lang/rust/issues/31844
pub type Elevation = NumericValueObject<i32, 1000000>;
pub type Distance = NumericValueObject<OrderedFloat<f64>, 0>;

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

impl<const MAX_ABS: u32> NumericValueObject<i32, MAX_ABS> {
    pub fn value(&self) -> i32 {
        self.0
    }
}

impl<const MAX_ABS: u32> NumericValueObject<OrderedFloat<f64>, MAX_ABS> {
    pub fn value(&self) -> f64 {
        self.0.into_inner()
    }
}

impl<T: Copy + FromPrimitive + Bounded, const MAX_ABS: u32> NumericValueObject<T, MAX_ABS> {
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

// NOTE: TryFrom<T>を実装しようとすると、coreのimplとconflictする
// これも特殊化待ち: https://github.com/rust-lang/rust/issues/31844
impl<const MAX_ABS: u32> TryFrom<f64> for NumericValueObject<OrderedFloat<f64>, MAX_ABS> {
    type Error = ApplicationError;

    fn try_from(val: f64) -> ApplicationResult<Self> {
        if (MAX_ABS == 0 || val.abs() <= MAX_ABS.into()) && val.is_finite() {
            Ok(Self(OrderedFloat(val)))
        } else {
            Err(ApplicationError::ValueObjectError(format!(
                // TODO: stringのconst genericsが追加されたら、
                // メッセージに具体的なエイリアス名(Latitudeとか)を入れる
                "Invalid value {} for NumericValueObject<OrderedFloat<f64>, {}>",
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
