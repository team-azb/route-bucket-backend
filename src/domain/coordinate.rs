use crate::lib::error::{ApplicationError, ApplicationResult};

use crate::domain::types::{Latitude, Longitude};
use bigdecimal::BigDecimal;
use getset::Getters;

/// Value Object for Coordinates
#[derive(Clone, Debug, PartialEq, Getters)]
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
