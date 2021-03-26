use bigdecimal::BigDecimal;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use crate::domain::types::{Latitude, Longitude};
use crate::utils::error::{ApplicationError, ApplicationResult};

/// Value Object for Coordinates
#[derive(Clone, Debug, PartialEq, Getters, Deserialize, Serialize)]
#[get = "pub"]
pub struct Coordinate {
    latitude: Latitude,
    longitude: Longitude,
}

impl Coordinate {
    pub fn new(lat: BigDecimal, lon: BigDecimal) -> ApplicationResult<Coordinate> {
        let coord = Coordinate {
            latitude: Latitude::from(lat)?,
            longitude: Longitude::from(lon)?,
        };
        Ok(coord)
    }
}

impl TryFrom<(f64, f64)> for Coordinate {
    type Error = ApplicationError;

    fn try_from(tuple: (f64, f64)) -> ApplicationResult<Coordinate> {
        let coord = Coordinate::new(BigDecimal::from(tuple.0), BigDecimal::from(tuple.1))?;
        Ok(coord)
    }
}
