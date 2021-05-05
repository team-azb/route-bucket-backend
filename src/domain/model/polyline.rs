use crate::domain::model::types::{Latitude, Longitude};
use crate::utils::error::{ApplicationError, ApplicationResult};
use geo::LineString;
use getset::Getters;
use polyline::{decode_polyline, encode_coordinates};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::iter::FromIterator;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(into = "String")]
pub struct Polyline(Vec<Coordinate>);

impl Polyline {
    pub fn new() -> Polyline {
        Polyline(Vec::new())
    }

    pub fn from_vec(points: Vec<Coordinate>) -> Polyline {
        Polyline(points)
    }

    pub fn encode(&self) -> ApplicationResult<String> {
        let line_str: LineString<f64> = LineString::from_iter(self.0.clone().into_iter());
        encode_coordinates(line_str, 5)
            // TODO: encode_coordinatesのErr(String)も表示する
            .map_err(|_| ApplicationError::DomainError("failed to encode polyline".into()))
    }

    pub fn decode(poly_str: &String) -> ApplicationResult<Polyline> {
        let line_str = decode_polyline(poly_str, 5)
            // TODO: encode_coordinatesのErr(String)も表示する
            .map_err(|_| ApplicationError::DomainError("failed to encode polyline".into()))?;
        Ok(Polyline::from_vec(
            line_str
                .into_iter()
                .map(|coord| Coordinate::new(coord.y, coord.x))
                .collect::<ApplicationResult<Vec<_>>>()?,
        ))
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

    pub fn clear(&mut self) -> Polyline {
        std::mem::replace(self, Polyline::new())
    }

    // only when points is empty
    pub fn init_if_empty(&mut self, points: Polyline) -> ApplicationResult<()> {
        if !self.0.is_empty() {
            Err(ApplicationError::DomainError(
                "Failed to set points. self.points was already inited.".into(),
            ))
        } else {
            self.0 = points.0;
            Ok(())
        }
    }
}

impl Into<String> for Polyline {
    fn into(self) -> String {
        // Coordinateで範囲チェックしてるので
        // encode_coordinatesのerrには引っかからないはず
        self.encode().unwrap()
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
    pub fn new(lat: f64, lon: f64) -> ApplicationResult<Coordinate> {
        let coord = Coordinate {
            latitude: Latitude::try_from(lat)?,
            longitude: Longitude::try_from(lon)?,
        };
        Ok(coord)
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
