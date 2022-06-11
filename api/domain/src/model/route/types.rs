use std::convert::TryFrom;

use derive_more::{Add, AddAssign, Display, From, Into, Sub, Sum};
use num_traits::{Bounded, Float, FromPrimitive, Zero};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use route_bucket_utils::{ApplicationError, ApplicationResult};
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Default, Display, From, Into, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Polyline(String);

impl Polyline {
    pub fn new() -> Self {
        Self(String::new())
    }
}

// NOTE: genericsの特殊化が実装されたら、FloatValueObjectは不要かも
//     : NumericValueObject<OrderedFloat<T>>の特殊化で乗り切れる気がしている
pub type Latitude = FloatValueObject<f64, 90, -90>;
pub type Longitude = FloatValueObject<f64, 180, -180>;
pub type Elevation = NumericValueObject<i32>;
pub type Distance = FloatValueObject<f64, { i64::MAX }>;

/// Value Object for Numeric type
#[derive(
    Default,
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
pub struct NumericValueObject<T, const MAX: i64 = 0, const MIN: i64 = 0>(T);

type FloatValueObject<T, const MAX: i64 = 0, const MIN: i64 = 0> =
    NumericValueObject<OrderedFloat<T>, MAX, MIN>;

// NOTE: Tに対して実装しようとすると、下のFloatへの実装とconflictする
// traitの特殊化待ち: https://github.com/rust-lang/rust/issues/31844
impl<const MAX: i64, const MIN: i64> NumericValueObject<i32, MAX, MIN> {
    pub fn value(&self) -> i32 {
        self.0
    }
}

impl<T: Float, const MAX: i64, const MIN: i64> FloatValueObject<T, MAX, MIN> {
    pub fn value(&self) -> T {
        self.0.into_inner()
    }
}

impl<T: Zero, const MAX: i64, const MIN: i64> Zero for NumericValueObject<T, MAX, MIN> {
    fn zero() -> Self {
        Self(T::zero())
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<T, const MAX: i64, const MIN: i64> NumericValueObject<T, MAX, MIN> {
    fn has_limit() -> bool {
        MIN < MAX
    }
}

impl<T: Bounded + FromPrimitive, const MAX: i64, const MIN: i64> Bounded
    for NumericValueObject<T, MAX, MIN>
{
    fn min_value() -> Self {
        Self(
            Self::has_limit()
                .then(|| MIN)
                .and_then(T::from_i64)
                .unwrap_or_else(T::min_value),
        )
    }

    fn max_value() -> Self {
        Self(
            Self::has_limit()
                .then(|| MAX)
                .and_then(T::from_i64)
                .unwrap_or_else(T::max_value),
        )
    }
}

impl<T, const MAX: i64, const MIN: i64> Validate for NumericValueObject<T, MAX, MIN>
where
    Self: Bounded + PartialOrd,
{
    fn validate(&self) -> Result<(), ValidationErrors> {
        (Self::min_value()..=Self::max_value())
            .contains(self)
            .then(|| ())
            .ok_or_else(|| {
                let mut errs = ValidationErrors::new();
                errs.add("0", ValidationError::new("Value out of range!"));
                errs
            })
    }
}

// NOTE: TryFrom<T>を実装しようとすると、coreのimplとconflictする
// これも特殊化待ち: https://github.com/rust-lang/rust/issues/31844
impl<const MAX: i64, const MIN: i64> TryFrom<i32> for NumericValueObject<i32, MAX, MIN> {
    type Error = ApplicationError;

    fn try_from(val: i32) -> ApplicationResult<Self> {
        let res = Self(val);
        res.validate()?;
        Ok(res)
    }
}

// NOTE: TryFrom<T>を実装しようとすると、coreのimplとconflictする
impl<const MAX: i64, const MIN: i64> TryFrom<f64> for FloatValueObject<f64, MAX, MIN> {
    type Error = ApplicationError;

    fn try_from(val: f64) -> ApplicationResult<Self> {
        let res = Self(OrderedFloat(val));
        res.validate()?;
        Ok(res)
    }
}
