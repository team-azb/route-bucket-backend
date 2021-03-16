use crate::lib::error::{ApplicationError, ApplicationResult};

/// A Value Class for Coordinates
#[derive(Clone, Debug, PartialEq)]
pub struct Coordinate {
    latitude: Latitude,
    longitude: Longitude,
}

impl Coordinate {
    pub fn create(lat :f64, lon: f64) -> ApplicationResult<Coordinate> {
        let coord = Coordinate{
            latitude: Latitude::from_f64(lat)?,
            longitude: Longitude::from_f64(lon)?,
        };
        Ok(coord)
    }
}

pub trait FromF64<T> {
    fn from_f64(val :f64) -> ApplicationResult<T>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Latitude(f64);

impl FromF64<Latitude> for Latitude {
    fn from_f64(val :f64) -> ApplicationResult<Self> {
        if val.abs() <= 90.0 {
            Ok(Latitude(val))
        } else {
            Err(ApplicationError::ValueObjectError("Absolute value of Latitude must be <= 90"))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Longitude(f64);


impl FromF64<Longitude> for Longitude {
    fn from_f64(val :f64) -> ApplicationResult<Self> {
        if val.abs() <= 180.0 {
            Ok(Longitude(val))
        } else {
            Err(ApplicationError::ValueObjectError("Absolute value of Longitude must be <= 180"))
        }
    }
}
