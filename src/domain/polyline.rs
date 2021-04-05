use bigdecimal::BigDecimal;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use crate::domain::types::{Latitude, Longitude};
use crate::utils::error::{ApplicationError, ApplicationResult};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Polyline(Vec<Coordinate>);

impl Polyline {
    pub fn new() -> Polyline {
        Polyline(Vec::new())
    }

    pub fn from_vec(points: Vec<Coordinate>) -> Polyline {
        Polyline(points)
    }

    pub fn insert_point(&mut self, pos: usize, point: Coordinate) -> ApplicationResult<()> {
        if pos > self.len() {
            // TODO: ここの説明の改善
            Err(ApplicationError::DomainError("Failed to insert point."))
        } else {
            Ok(self.insert(pos, point))
        }
    }

    pub fn remove_point(&mut self, pos: usize) -> ApplicationResult<Coordinate> {
        if pos > self.len() {
            Err(ApplicationError::DomainError("Failed to remove point."))
        } else {
            Ok(self.remove(pos))
        }
    }

    pub fn clear_points(&mut self) -> Polyline {
        std::mem::replace(self, Polyline::new())
    }

    // only when points is empty
    pub fn init_points(&mut self, points: Polyline) -> ApplicationResult<()> {
        if self.is_empty() {
            Err(ApplicationError::DomainError(
                "Failed to set points. self.points was already inited.",
            ))
        } else {
            self.0 = points.0;
            Ok(())
        }
    }
}

// Vecのメソッド(sizeや[i])をそのまま呼べるように
impl Deref for Polyline {
    type Target = Vec<Coordinate>;

    fn deref(&self) -> &Vec<Coordinate> {
        &self.0
    }
}
impl DerefMut for Polyline {
    fn deref_mut(&mut self) -> &mut Vec<Coordinate> {
        &mut self.0
    }
}

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
