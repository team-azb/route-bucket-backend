use std::convert::{TryFrom, TryInto};
use std::iter::FromIterator;

use geo::algorithm::haversine_distance::HaversineDistance;
use getset::Getters;
use itertools::Itertools;
use polyline::{decode_polyline, encode_coordinates};
use serde::{Deserialize, Serialize};

use route_bucket_utils::{ApplicationError, ApplicationResult};

use crate::model::types::{Distance, Elevation, Latitude, Longitude, Polyline};

/// Value Object for Coordinates
#[derive(Clone, Debug, PartialEq, Getters, Deserialize, Serialize)]
#[get = "pub"]
pub struct Coordinate {
    pub(super) latitude: Latitude,
    pub(super) longitude: Longitude,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) elevation: Option<Elevation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) distance_from_start: Option<Distance>,
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

impl From<Coordinate> for Polyline {
    fn from(value: Coordinate) -> Self {
        vec![value].into()
    }
}

impl TryFrom<Polyline> for Coordinate {
    type Error = ApplicationError;

    fn try_from(value: Polyline) -> Result<Self, Self::Error> {
        let mut coords = Vec::try_from(value)?;
        if !coords.is_empty() {
            Ok(coords.swap_remove(0))
        } else {
            Err(ApplicationError::DomainError(
                "Cannot convert an empty Polyline into a Coordinate!".into(),
            ))
        }
    }
}

impl From<Option<Coordinate>> for Polyline {
    fn from(value: Option<Coordinate>) -> Self {
        value.map(Polyline::from).unwrap_or(Polyline::new())
    }
}

impl TryFrom<Polyline> for Option<Coordinate> {
    type Error = ApplicationError;

    fn try_from(value: Polyline) -> Result<Self, Self::Error> {
        let mut coords = Vec::try_from(value)?;
        Ok((!coords.is_empty()).then(|| coords.swap_remove(0)))
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

#[cfg(test)]
pub(crate) mod tests {
    use rstest::{fixture, rstest};

    use super::*;

    #[fixture]
    fn tokyo() -> Coordinate {
        Coordinate::tokyo(false, None)
    }

    #[fixture]
    fn tokyo_verbose() -> Coordinate {
        Coordinate::tokyo(true, Some(0.))
    }

    #[fixture]
    fn yokohama() -> Coordinate {
        Coordinate::yokohama(false, None)
    }

    #[rstest]
    #[case::lower_bound(-90.0, -180.0)]
    #[case::akashi(35.0, 135.0)]
    #[case::upper_bound(90.0, 180.0)]
    #[should_panic(expected = "ValueObjectError returned.")]
    #[case::lat_too_small(-90.1, -180.0)]
    #[should_panic(expected = "ValueObjectError returned.")]
    #[case::lon_too_small(-90.0, -180.1)]
    #[should_panic(expected = "ValueObjectError returned.")]
    #[case::lat_too_big(90.1, 180.0)]
    #[should_panic(expected = "ValueObjectError returned.")]
    #[case::lon_too_big(90.0, 180.1)]
    fn init_validation(#[case] lat: f64, #[case] lon: f64) {
        let result = Coordinate::new(lat, lon);
        match result {
            Ok(coord) => {
                assert_eq!(coord, init_coord(lat, lon, None, None))
            }
            Err(ApplicationError::ValueObjectError(_)) => {
                panic!("ValueObjectError returned.")
            }
            Err(err) => {
                panic!("Unexpected error {:?} returned!", err)
            }
        }
    }

    #[rstest]
    fn can_set_elevation(#[from(tokyo)] mut empty_coord: Coordinate) {
        empty_coord.set_elevation(Some(Elevation::zero())).unwrap();
        assert_eq!(empty_coord.elevation, Some(Elevation::zero()))
    }

    #[rstest]
    fn cannot_set_elevation_twice(#[from(tokyo_verbose)] mut coord_with_elev: Coordinate) {
        let result = coord_with_elev.set_elevation(Some(Elevation::zero()));
        assert!(matches!(result, Err(ApplicationError::DomainError(_))))
    }

    #[rstest]
    #[case::coord_without_distance(tokyo())]
    #[case::coord_with_distance(tokyo_verbose())]
    fn can_set_distance(#[case] mut coord: Coordinate) {
        coord.set_distance_from_start(Distance::zero());
        assert_eq!(coord.distance_from_start, Some(Distance::zero()))
    }

    #[rstest]
    fn calc_correct_haversine_distance(tokyo: Coordinate, yokohama: Coordinate) {
        let distance = tokyo.haversine_distance(&yokohama);
        assert_eq!(distance.value(), 26936.42633640023)
    }

    #[rstest]
    #[case::empty(Coordinate::empty_coords(), Coordinate::empty_polyline())]
    #[case::yokohama_to_tokyo(
        Coordinate::yokohama_to_tokyo_coords(false, None),
        Coordinate::yokohama_to_tokyo_polyline()
    )]
    fn convert_coords_into_polyline(#[case] coords: Vec<Coordinate>, #[case] polyline: Polyline) {
        assert_eq!(Polyline::from(coords), polyline)
    }

    #[rstest]
    #[case::empty(Coordinate::empty_polyline(), Coordinate::empty_coords())]
    #[case::yokohama_to_tokyo(
        Coordinate::yokohama_to_tokyo_polyline(),
        Coordinate::yokohama_to_tokyo_coords(false, None)
    )]
    fn convert_polyline_into_coords(#[case] polyline: Polyline, #[case] coords: Vec<Coordinate>) {
        assert_eq!(Vec::try_from(polyline), Ok(coords))
    }

    fn init_coord(lat: f64, lon: f64, ele: Option<i32>, dist: Option<f64>) -> Coordinate {
        Coordinate {
            latitude: lat.try_into().unwrap(),
            longitude: lon.try_into().unwrap(),
            elevation: ele.map(Elevation::try_from).transpose().unwrap(),
            distance_from_start: dist.map(Distance::try_from).transpose().unwrap(),
        }
    }

    pub trait CoordinateFixtures {
        fn yokohama(set_ele: bool, dist: Option<f64>) -> Coordinate {
            init_coord(35.46798, 139.62607, set_ele.then(|| 1), dist)
        }

        fn tokyo(set_ele: bool, dist: Option<f64>) -> Coordinate {
            init_coord(35.68048, 139.76906, set_ele.then(|| 4), dist)
        }

        fn chiba(set_ele: bool, dist: Option<f64>) -> Coordinate {
            init_coord(35.61311, 140.11135, set_ele.then(|| 11), dist)
        }

        fn empty_coords() -> Vec<Coordinate> {
            vec![]
        }

        fn yokohama_to_tokyo_coords(set_ele: bool, dist_offset: Option<f64>) -> Vec<Coordinate> {
            vec![
                Self::yokohama(set_ele, dist_offset.clone()),
                Self::tokyo(set_ele, dist_offset.map(|d| d + 26936.42633640023)),
            ]
        }

        fn tokyo_to_chiba_coords(set_ele: bool, dist_offset: Option<f64>) -> Vec<Coordinate> {
            vec![
                Self::tokyo(set_ele, dist_offset.clone()),
                Self::chiba(set_ele, dist_offset.map(|d| d + 31823.54759611465)),
            ]
        }

        fn yokohama_to_chiba_coords(set_ele: bool, dist_offset: Option<f64>) -> Vec<Coordinate> {
            vec![
                Self::yokohama(set_ele, dist_offset.clone()),
                Self::chiba(set_ele, dist_offset.map(|d| d + 46779.709825324135)),
            ]
        }

        fn yokohama_to_chiba_via_tokyo_coords(
            set_ele: bool,
            dist_offset: Option<f64>,
        ) -> Vec<Coordinate> {
            vec![
                Self::yokohama(set_ele, dist_offset.clone()),
                Self::tokyo(set_ele, dist_offset.map(|d| d + 26936.42633640023)),
                Self::chiba(set_ele, dist_offset.map(|d| d + 58759.97393251488)),
            ]
        }

        fn empty_polyline() -> Polyline {
            Polyline::from(String::from(""))
        }

        fn yokohama_to_tokyo_polyline() -> Polyline {
            Polyline::from(String::from("{inwE}uesYcoh@u|Z"))
        }

        fn yokohama_to_chiba_polyline() -> Polyline {
            Polyline::from(String::from("{inwE}uesYaj[_x}A"))
        }

        fn tokyo_to_chiba_polyline() -> Polyline {
            Polyline::from(String::from("_zwxEssatY`dLizaA"))
        }
    }

    impl CoordinateFixtures for Coordinate {}
}
