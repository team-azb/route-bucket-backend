use geo::algorithm::haversine_distance::HaversineDistance;
use getset::Getters;
use itertools::Itertools;
use num_traits::FromPrimitive;
use polyline::{decode_polyline, encode_coordinates};
use route_bucket_utils::{ApplicationError, ApplicationResult};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::iter::FromIterator;

use crate::model::types::{Distance, Elevation, Latitude, Longitude, Polyline};

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

impl TryFrom<geo::Coordinate<f64>> for Coordinate {
    type Error = ApplicationError;

    fn try_from(geo_coord: geo::Coordinate<f64>) -> ApplicationResult<Coordinate> {
        Ok(Coordinate {
            latitude: Latitude::try_from(geo_coord.y)?,
            longitude: Longitude::try_from(geo_coord.x)?,
            elevation: None,
            distance_from_start: None,
        })
    }
}

impl From<Coordinate> for gpx::Waypoint {
    fn from(coord: Coordinate) -> Self {
        let elevation = coord
            .elevation
            .map(|elev| elev.value())
            .map(f64::from_i32)
            .flatten();

        let mut waypoint = Self::new(<(f64, f64)>::from(coord).into());
        waypoint.elevation = elevation;

        waypoint
    }
}

impl From<Coordinate> for (f64, f64) {
    fn from(coord: Coordinate) -> (f64, f64) {
        (coord.longitude.value(), coord.latitude.value())
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
        line_str.into_iter().map(Coordinate::try_from).try_collect()
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
