use bigdecimal::BigDecimal;
use nanoid::nanoid;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

use crate::utils::error::{ApplicationError, ApplicationResult};

// TODO: Value Object用のderive macroを作る
// ↓みたいな一要素のタプル構造体たちにfrom, valueをデフォルトで実装したい
// ただのgenericsでSelf(val)やself.0.clone()をしようとすると怒られるので、
// derive macro + traitでやるしかなさそう

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Latitude(
    #[serde(
        serialize_with = "serialize_big_decimal",
        deserialize_with = "deserialize_big_decimal"
    )]
    BigDecimal,
);

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Longitude(
    #[serde(
        serialize_with = "serialize_big_decimal",
        deserialize_with = "deserialize_big_decimal"
    )]
    BigDecimal,
);

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

fn serialize_big_decimal<S>(target: &BigDecimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&target.to_string())
}

fn deserialize_big_decimal<'de, D>(deserializer: D) -> Result<BigDecimal, D::Error>
where
    D: Deserializer<'de>,
{
    let bd_string = String::deserialize(deserializer)?;
    // TODO: ここのunwrapどうにかする
    Ok(BigDecimal::from_str(&bd_string).unwrap())
}
