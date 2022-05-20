use derive_more::From;
use serde::Serialize;

use super::Coordinate;

#[derive(Clone, Debug, Serialize, From)]
#[cfg_attr(any(test, feature = "fixtures"), derive(PartialEq))]
pub struct BoundingBox {
    min_coord: Coordinate,
    max_coord: Coordinate,
}

#[cfg(any(test, feature = "fixtures"))]
pub(crate) mod tests {
    use std::convert::TryInto;

    use crate::model::route::coordinate::tests::CoordinateFixtures;

    use super::*;

    pub trait BoundingBoxFixture {
        fn yokohama() -> BoundingBox {
            BoundingBox {
                min_coord: Coordinate::yokohama(false, None),
                max_coord: Coordinate::yokohama(false, None),
            }
        }

        fn yokohama_to_chiba_via_tokyo() -> BoundingBox {
            BoundingBox {
                min_coord: Coordinate {
                    latitude: 35.46798.try_into().unwrap(),
                    longitude: 139.62607.try_into().unwrap(),
                    elevation: None,
                    distance_from_start: None,
                },
                max_coord: Coordinate {
                    latitude: 35.68048.try_into().unwrap(),
                    longitude: 140.11135.try_into().unwrap(),
                    elevation: None,
                    distance_from_start: None,
                },
            }
        }
    }

    impl BoundingBoxFixture for BoundingBox {}
}
