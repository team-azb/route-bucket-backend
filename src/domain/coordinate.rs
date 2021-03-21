use bigdecimal::BigDecimal;
use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::domain::types::{Latitude, Longitude};
use crate::lib::error::ApplicationResult;

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
