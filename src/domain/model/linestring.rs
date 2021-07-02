use std::convert::{TryFrom, TryInto};
use std::iter::FromIterator;
use std::slice::{Iter, IterMut};

use geo::algorithm::haversine_distance::HaversineDistance;
use getset::Getters;
use polyline::{decode_polyline, encode_coordinates};
use serde::{Deserialize, Serialize};

use crate::domain::model::types::{Distance, Elevation, Latitude, Longitude, Polyline};
use crate::utils::error::{ApplicationError, ApplicationResult};

/// Value Object for Coordinates
#[derive(Clone, Debug, PartialEq, Getters, Deserialize, Serialize)]
#[get = "pub"]
pub struct Coordinate {
    latitude: Latitude,
    longitude: Longitude,
    #[serde(skip_serializing_if = "Option::is_none")]
    elevation: Option<Elevation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    distance_from_start: Option<Distance>,
}

impl Coordinate {
    pub fn new(lat: f64, lon: f64) -> ApplicationResult<Coordinate> {
        let coord = Coordinate {
            latitude: Latitude::try_from(lat)?,
            longitude: Longitude::try_from(lon)?,
            elevation: None,
            distance_from_start: None,
        };
        Ok(coord)
    }

    pub fn set_elevation(&mut self, elevation: Option<Elevation>) -> ApplicationResult<()> {
        (self.elevation == None)
            .then(|| {
                self.elevation = elevation;
            })
            .ok_or(ApplicationError::DomainError(
                "Elevation already set for Coordinate.".into(),
            ))
    }

    pub fn set_distance_from_start(&mut self, distance: Distance) -> () {
        self.distance_from_start.insert(distance);
    }
}

impl From<Coordinate> for geo::Coordinate<f64> {
    fn from(coord: Coordinate) -> geo::Coordinate<f64> {
        geo::Coordinate {
            x: coord.longitude.value(),
            y: coord.latitude.value(),
        }
    }
}

impl From<Coordinate> for (f64, f64) {
    fn from(coord: Coordinate) -> (f64, f64) {
        (coord.latitude.value(), coord.longitude.value())
    }
}

impl From<Vec<Coordinate>> for Polyline {
    fn from(value: Vec<Coordinate>) -> Self {
        let line_str = geo::LineString::from_iter(value.into_iter());
        // 範囲チェックはCoordinateで行っているので、unwrapで大丈夫
        encode_coordinates(line_str, 5).map(Polyline::from).unwrap()
    }
}

impl TryFrom<Polyline> for Vec<Coordinate> {
    type Error = ApplicationError;

    fn try_from(value: Polyline) -> Result<Self, Self::Error> {
        let line_str = decode_polyline(&String::from(value), 5).map_err(|err| {
            ApplicationError::DomainError(format!("failed to encode polyline: {}", err))
        })?;
        Ok(Vec::from_iter(line_str.into_iter()))
    }
}

impl HaversineDistance<Distance> for Coordinate {
    fn haversine_distance(&self, rhs: &Self) -> Distance {
        geo::Point::new(self.longitude.value(), self.latitude.value())
            .haversine_distance(&geo::Point::new(
                rhs.longitude.value(),
                rhs.latitude.value(),
            ))
            .try_into()
            .unwrap()
    }
}
