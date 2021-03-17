use crate::lib::error::{ApplicationError, ApplicationResult};
use bigdecimal::BigDecimal;
use nanoid::nanoid;

// TODO: Value Object用のderive macroを作る
// ↓みたいな一要素のタプル構造体たちにfrom, valueをデフォルトで実装したい
// ただのgenericsでSelf(val)やself.0.clone()をしようとすると怒られるので、
// derive macro + traitでやるしかなさそう

#[derive(Debug, Clone)]
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

// TODO: rustc 1.51.0でconst genericsが実装される
// これを使うと、Latitude, Longitudeそれぞれのfromやvalueはいらなくなる
// (今MAX以外はimplの中身完全一致してる)
// ↑のderive ValueObjectをベースにしたtraitなイメージ (RangedValueObjectか、ValueObjectのオプションか)
// オプションならconst genericsいらなそう
#[derive(Clone, Debug, PartialEq)]
pub struct Latitude(BigDecimal);

impl Latitude {
    pub fn from(val: BigDecimal) -> ApplicationResult<Self> {
        if val.abs() <= BigDecimal::from(90.0) {
            Ok(Self(val))
        } else {
            Err(ApplicationError::ValueObjectError(
                "Absolute value of Latitude must be <= 90.0",
            ))
        }
    }

    pub fn value(&self) -> BigDecimal {
        self.0.clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Longitude(BigDecimal);

impl Longitude {
    pub fn from(val: BigDecimal) -> ApplicationResult<Self> {
        if val.abs() <= BigDecimal::from(180.0) {
            Ok(Self(val))
        } else {
            Err(ApplicationError::ValueObjectError(
                "Absolute value of Latitude must be <= 180.0",
            ))
        }
    }

    pub fn value(&self) -> BigDecimal {
        self.0.clone()
    }
}
