use std::convert::TryFrom;
use std::iter::FromIterator;
use std::slice::{Iter, IterMut};

use getset::Getters;
use polyline::{decode_polyline, encode_coordinates};
use serde::{Deserialize, Serialize};

use crate::domain::model::types::{Elevation, Latitude, Longitude, Polyline};
use crate::utils::error::{ApplicationError, ApplicationResult};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct LineString(Vec<Coordinate>);

impl LineString {
    pub fn new() -> LineString {
        LineString(Vec::new())
    }

    pub fn get(&self, i: usize) -> ApplicationResult<&Coordinate> {
        if i < self.0.len() {
            Ok(&self.0[i])
        } else {
            Err(ApplicationError::DomainError(
                "Index out of range in get.".into(),
            ))
        }
    }

    pub fn replace(&mut self, i: usize, val: Coordinate) -> ApplicationResult<Coordinate> {
        if i < self.0.len() {
            Ok(std::mem::replace(&mut self.0[i], val))
        } else {
            Err(ApplicationError::DomainError(
                "Index out of range in set.".into(),
            ))
        }
    }

    pub fn insert(&mut self, pos: usize, point: Coordinate) -> ApplicationResult<()> {
        if pos > self.0.len() {
            // TODO: ここの説明の改善
            Err(ApplicationError::DomainError(
                "Failed to insert point.".into(),
            ))
        } else {
            Ok(self.0.insert(pos, point))
        }
    }

    pub fn remove(&mut self, pos: usize) -> ApplicationResult<Coordinate> {
        if pos > self.0.len() {
            Err(ApplicationError::DomainError(
                "Failed to remove point.".into(),
            ))
        } else {
            Ok(self.0.remove(pos))
        }
    }

    pub fn clear(&mut self) -> LineString {
        std::mem::replace(self, LineString::new())
    }

    // only when points is empty
    pub fn init_if_empty(&mut self, points: LineString) -> ApplicationResult<()> {
        if !self.0.is_empty() {
            Err(ApplicationError::DomainError(
                "Failed to set points. self.points was already inited.".into(),
            ))
        } else {
            self.0 = points.0;
            Ok(())
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> Iter<Coordinate> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Coordinate> {
        self.0.iter_mut()
    }
}

impl From<LineString> for geo::LineString<f64> {
    fn from(value: LineString) -> Self {
        geo::LineString::from_iter(value.0.into_iter())
    }
}

impl TryFrom<geo::LineString<f64>> for LineString {
    type Error = ApplicationError;

    fn try_from(value: geo::LineString<f64>) -> Result<Self, Self::Error> {
        value
            .into_iter()
            .map(|coord| Coordinate::new(coord.y, coord.x))
            .collect::<ApplicationResult<Vec<_>>>()
            .map(LineString::from)
    }
}

impl From<Vec<Coordinate>> for LineString {
    fn from(points: Vec<Coordinate>) -> Self {
        LineString(points)
    }
}

impl From<LineString> for Polyline {
    fn from(value: LineString) -> Self {
        let line_str = geo::LineString::from(value);
        // 範囲チェックはCoordinateで行っているので、unwrapで大丈夫
        encode_coordinates(line_str, 5).map(Polyline::from).unwrap()
    }
}

impl TryFrom<Polyline> for LineString {
    type Error = ApplicationError;

    fn try_from(value: Polyline) -> Result<Self, Self::Error> {
        let line_str = decode_polyline(&String::from(value), 5).map_err(|err| {
            ApplicationError::DomainError(format!("failed to encode polyline: {}", err))
        })?;
        LineString::try_from(line_str)
    }
}

/// Value Object for Coordinates
#[derive(Clone, Debug, PartialEq, Getters, Deserialize, Serialize)]
#[get = "pub"]
pub struct Coordinate {
    latitude: Latitude,
    longitude: Longitude,
    #[serde(skip_serializing_if = "Option::is_none")]
    elevation: Option<Elevation>,
}

impl Coordinate {
    pub fn new(lat: f64, lon: f64) -> ApplicationResult<Coordinate> {
        let coord = Coordinate {
            latitude: Latitude::try_from(lat)?,
            longitude: Longitude::try_from(lon)?,
            elevation: None,
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
