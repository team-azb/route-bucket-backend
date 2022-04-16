use std::convert::TryFrom;

use derive_more::{Add, AddAssign, Display, From, Into, Sub, Sum};
use num_traits::{Bounded, Float, FromPrimitive, Signed, Zero};
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

/// Value Object for BigDecimal type
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

impl<T: Copy, const MAX: i64, const MIN: i64> NumericValueObject<T, MAX, MIN> {
    pub fn value(&self) -> T {
        self.0
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

impl<T: Bounded + Signed + FromPrimitive, const MAX: i64, const MIN: i64> Bounded
    for NumericValueObject<T, MAX, MIN>
{
    fn min_value() -> Self {
        Self(if MAX > MIN {
            T::from_i64(MIN).unwrap()
        } else {
            T::min_value()
        })
    }

    fn max_value() -> Self {
        Self(if MAX > MIN {
            T::from_i64(MAX).unwrap()
        } else {
            T::max_value()
        })
    }
}

pub trait ValidatableInner: PartialOrd + Bounded + Signed + FromPrimitive {}
impl<T: PartialOrd + Bounded + Signed + FromPrimitive> ValidatableInner for T {}

impl<T: ValidatableInner, const MAX: i64, const MIN: i64> Validate
    for NumericValueObject<T, MAX, MIN>
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

pub trait InnerFloat: ValidatableInner + Float {}
impl<T: ValidatableInner + Float> InnerFloat for T {}

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
    PartialOrd,
    Serialize,
    Deserialize,
    Validate,
)]
pub struct FloatValueObject<T: InnerFloat, const MAX: i64 = 0, const MIN: i64 = 0> {
    #[validate]
    value: NumericValueObject<OrderedFloat<T>, MAX, MIN>,
}

impl<T: InnerFloat, const MAX: i64, const MIN: i64> FloatValueObject<T, MAX, MIN> {
    pub fn value(&self) -> T {
        self.value.value().into_inner()
    }
}

impl<T: InnerFloat + Zero, const MAX: i64, const MIN: i64> Zero for FloatValueObject<T, MAX, MIN> {
    fn zero() -> Self {
        Self {
            value: NumericValueObject::<OrderedFloat<T>, MAX, MIN>::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        self.value.is_zero()
    }
}

impl<T: InnerFloat, const MAX: i64, const MIN: i64> Bounded for FloatValueObject<T, MAX, MIN> {
    fn min_value() -> Self {
        Self {
            value: NumericValueObject::<OrderedFloat<T>, MAX, MIN>::min_value(),
        }
    }

    fn max_value() -> Self {
        Self {
            value: NumericValueObject::<OrderedFloat<T>, MAX, MIN>::max_value(),
        }
    }
}

// NOTE: TryFrom<T>を実装しようとすると、coreのimplとconflictする
impl<const MAX: i64, const MIN: i64> TryFrom<f64> for FloatValueObject<f64, MAX, MIN> {
    type Error = ApplicationError;

    fn try_from(val: f64) -> ApplicationResult<Self> {
        let res = Self {
            value: NumericValueObject::<_, MAX, MIN>(OrderedFloat(val)),
        };
        res.validate()?;
        Ok(res)
    }
}

// NOTE: derive(Eq, Ord)ではダメで、なぜか手で実装する必要があった
impl<T: InnerFloat, const MAX: i64, const MIN: i64> Eq for FloatValueObject<T, MAX, MIN> where
    FloatValueObject<T, MAX, MIN>: PartialEq
{
}
impl<T: InnerFloat, const MAX: i64, const MIN: i64> Ord for FloatValueObject<T, MAX, MIN>
where
    FloatValueObject<T, MAX, MIN>: Eq,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}
