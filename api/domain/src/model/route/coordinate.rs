use std::convert::{TryFrom, TryInto};
use std::iter::FromIterator;

use geo::algorithm::haversine_distance::HaversineDistance;
use getset::Getters;
use itertools::Itertools;
use num_traits::FromPrimitive;
use polyline::{decode_polyline, encode_coordinates};
use serde::{Deserialize, Serialize};

use route_bucket_utils::{ApplicationError, ApplicationResult};

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

#[cfg(test)]
pub(crate) mod tests {
    use rstest::rstest;

    use super::*;

    fn init_coord(lat: f64, lon: f64, ele: Option<i32>, dist: Option<f64>) -> Coordinate {
        Coordinate {
            latitude: lat.try_into().unwrap(),
            longitude: lon.try_into().unwrap(),
            elevation: ele.map(Elevation::try_from).transpose().unwrap(),
            distance_from_start: dist.map(Distance::try_from).transpose().unwrap(),
        }
    }

    fn coords_to_tuples(coords: Vec<Coordinate>) -> Vec<(f64, f64)> {
        coords.into_iter().map(<(f64, f64)>::from).collect()
    }

    pub trait CoordinateFixtures {
        fn tokyo() -> Coordinate {
            init_coord(35.68048, 139.76906, None, None)
        }

        fn yokohama() -> Coordinate {
            init_coord(35.46798, 139.62607, None, None)
        }

        fn chiba() -> Coordinate {
            init_coord(35.61311, 140.11135, None, None)
        }

        fn empty_coords() -> Vec<Coordinate> {
            vec![]
        }

        fn yokohama_to_tokyo_coords(set_ele: bool, set_dist: bool) -> Vec<Coordinate> {
            vec![
                init_coord(
                    35.46798,
                    139.62607,
                    set_ele.then(|| 1),
                    set_dist.then(|| 0.),
                ),
                init_coord(
                    35.52735,
                    139.73837,
                    set_ele.then(|| 1),
                    set_dist.then(|| 12121.713407643354),
                ),
                init_coord(
                    35.62426,
                    139.74990,
                    set_ele.then(|| 0),
                    set_dist.then(|| 22947.965176717134),
                ),
                init_coord(
                    35.68048,
                    139.76906,
                    set_ele.then(|| 4),
                    set_dist.then(|| 29434.629256467866),
                ),
            ]
        }

        fn tokyo_to_chiba_coords(set_ele: bool, set_dist: bool) -> Vec<Coordinate> {
            vec![
                init_coord(
                    35.68048,
                    139.76906,
                    set_ele.then(|| 4),
                    set_dist.then(|| 29434.629256467866),
                ),
                init_coord(
                    35.69341,
                    139.98265,
                    set_ele.then(|| 1),
                    set_dist.then(|| 48778.399581350575),
                ),
                init_coord(
                    35.61311,
                    140.11135,
                    set_ele.then(|| 11),
                    set_dist.then(|| 63439.42063598467),
                ),
            ]
        }

        fn yokohama_to_chiba_coords(set_ele: bool, set_dist: bool) -> Vec<Coordinate> {
            vec![
                init_coord(
                    35.46798,
                    139.62607,
                    set_ele.then(|| 1),
                    set_dist.then(|| 0.),
                ),
                init_coord(
                    35.51566,
                    139.79108,
                    set_ele.then(|| 3),
                    set_dist.then(|| 15852.041258204548),
                ),
                init_coord(
                    35.40698,
                    139.95472,
                    set_ele.then(|| 6),
                    set_dist.then(|| 34975.16367611116),
                ),
                init_coord(
                    35.61311,
                    140.11135,
                    set_ele.then(|| 11),
                    set_dist.then(|| 61926.0425172123),
                ),
            ]
        }

        fn yokohama_to_chiba_waypoints() -> Vec<Coordinate> {
            vec![Self::yokohama(), Self::tokyo(), Self::chiba()]
        }

        fn empty_polyline() -> Polyline {
            Polyline::from(String::from(""))
        }

        fn yokohama_to_tokyo_polyline() -> Polyline {
            Polyline::from(String::from("{inwE}uesYarJ{|Tu|QagAk~IwvB"))
        }

        fn yokohama_to_chiba_polyline() -> Polyline {
            Polyline::from(String::from("{inwE}uesY_iHif_@ffTw}^igg@}q]"))
        }

        fn tokyo_to_chiba_polyline() -> Polyline {
            Polyline::from(String::from("_zwxEssatYyoA}uh@ztNkcX"))
        }
    }

    impl CoordinateFixtures for Coordinate {}

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
    #[case::coord_without_elevation(Coordinate::tokyo())]
    #[should_panic(expected = "DomainError returned.")]
    #[case::coord_with_elevation(Coordinate::yokohama_to_tokyo_coords(true, false)[0].clone())]
    fn can_set_elevation_only_once(#[case] mut coord: Coordinate) {
        let result = coord.set_elevation(Some(Elevation::zero()));
        match result {
            Ok(()) => assert_eq!(coord.elevation, Some(Elevation::zero())),
            Err(ApplicationError::DomainError(_)) => {
                panic!("DomainError returned.")
            }
            Err(err) => {
                panic!("Unexpected error {:?} returned!", err)
            }
        }
    }

    #[rstest]
    #[case::coord_without_distance_from_start(Coordinate::tokyo())]
    #[case::coord_with_distance_from_start(Coordinate::yokohama_to_tokyo_coords(false, true)[0].clone())]
    fn can_set_distance(#[case] mut coord: Coordinate) {
        coord.set_distance_from_start(Distance::zero());
        assert_eq!(coord.distance_from_start, Some(Distance::zero()))
    }

    #[rstest]
    #[case::tokyo_to_yokohama(Coordinate::tokyo(), Coordinate::yokohama(), 26_936)]
    #[case::yokohama_to_tokyo(Coordinate::yokohama(), Coordinate::tokyo(), 26_936)]
    fn calc_correct_haversine_distance(
        #[case] from: Coordinate,
        #[case] to: Coordinate,
        #[case] expected_distance: i32, // meters
    ) {
        let distance = from.haversine_distance(&to);
        assert_eq!(distance.value().round() as i32, expected_distance)
    }

    #[rstest]
    #[case::empty(Coordinate::empty_coords(), Coordinate::empty_polyline())]
    #[case::yokohama_to_tokyo(
        Coordinate::yokohama_to_tokyo_coords(false, false),
        Coordinate::yokohama_to_tokyo_polyline()
    )]
    fn convert_coords_into_polyline(#[case] coords: Vec<Coordinate>, #[case] polyline: Polyline) {
        assert_eq!(Polyline::from(coords), polyline)
    }

    #[rstest]
    #[case::empty(Coordinate::empty_polyline(), Coordinate::empty_coords())]
    #[case::yokohama_to_tokyo(
        Coordinate::yokohama_to_tokyo_polyline(),
        Coordinate::yokohama_to_tokyo_coords(false, false)
    )]
    fn convert_polyline_into_coords(#[case] polyline: Polyline, #[case] coords: Vec<Coordinate>) {
        assert_eq!(
            coords_to_tuples(Vec::try_from(polyline).unwrap()),
            coords_to_tuples(coords)
        )
    }
}
