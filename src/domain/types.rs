use bigdecimal::BigDecimal;
use derive_more::Display;
use nanoid::nanoid;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

use crate::utils::error::{ApplicationError, ApplicationResult};
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

pub type Latitude = BigDecimalValueObject<90>;
pub type Longitude = BigDecimalValueObject<180>;

/// Value Object for BigDecimal type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BigDecimalValueObject<const MAX_ABS: u64>(
    #[serde(
        serialize_with = "serialize_big_decimal",
        deserialize_with = "deserialize_big_decimal"
    )]
    BigDecimal,
);

impl<const MAX_ABS: u64> BigDecimalValueObject<MAX_ABS> {
    pub fn value(&self) -> BigDecimal {
        self.0.clone()
    }
}

impl<const MAX_ABS: u64> TryFrom<BigDecimal> for BigDecimalValueObject<MAX_ABS> {
    type Error = ApplicationError;

    fn try_from(val: BigDecimal) -> ApplicationResult<Self> {
        if val.abs() <= BigDecimal::from(MAX_ABS) {
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
