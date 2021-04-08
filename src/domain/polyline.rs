use crate::domain::types::{Latitude, Longitude};
use crate::utils::error::{ApplicationError, ApplicationResult};
use bigdecimal::BigDecimal;
use geo::{LineString, Point};
use getset::Getters;
use polyline::{decode_polyline, encode_coordinates};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::iter::FromIterator;
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

    pub fn insert_point(&mut self, pos: usize, point: Coordinate) -> ApplicationResult<()> {
        if pos > self.len() {
            // TODO: ここの説明の改善
            Err(ApplicationError::DomainError(
                "Failed to insert point.".into(),
            ))
        } else {
            Ok(self.insert(pos, point))
        }
    }

    pub fn remove_point(&mut self, pos: usize) -> ApplicationResult<Coordinate> {
        if pos > self.len() {
            Err(ApplicationError::DomainError(
                "Failed to remove point.".into(),
            ))
        } else {
            Ok(self.remove(pos))
        }
    }

    pub fn clear_points(&mut self) -> Polyline {
        std::mem::replace(self, Polyline::new())
    }

    // only when points is empty
    pub fn init_points(&mut self, points: Polyline) -> ApplicationResult<()> {
        if !self.is_empty() {
            Err(ApplicationError::DomainError(
                "Failed to set points. self.points was already inited.".into(),
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
