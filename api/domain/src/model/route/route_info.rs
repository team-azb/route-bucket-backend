use chrono::{DateTime, Utc};
use derivative::Derivative;
use derive_more::From;
use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::model::user::UserId;

use super::{Distance, Elevation, RouteId};

#[derive(Clone, Debug, From, Getters, Derivative, Deserialize, Serialize)]
#[get = "pub"]
#[derivative(Default)]
#[cfg_attr(any(test, feature = "fixtures"), derivative(PartialEq))]
pub struct RouteInfo {
    #[derivative(Default(value = "RouteId::new()"))]
    #[cfg_attr(any(test, feature = "fixtures"), derivative(PartialEq = "ignore"))]
    pub(super) id: RouteId,
    pub(super) name: String,
    #[derivative(Default(value = "UserId::from(\"\".to_string())"))]
    pub(super) owner_id: UserId,
    #[serde(skip_serializing)]
    pub(super) op_num: usize,
    pub(super) ascent_elevation_gain: Elevation,
    pub(super) descent_elevation_gain: Elevation,
    pub(super) total_distance: Distance,
    #[derivative(Default(value = "chrono::MIN_DATETIME"))]
    pub(super) created_at: DateTime<Utc>,
    #[derivative(Default(value = "chrono::MIN_DATETIME"))]
    pub(super) updated_at: DateTime<Utc>,
}

impl RouteInfo {
    pub fn new(name: &str, owner_id: UserId) -> RouteInfo {
        RouteInfo {
            name: name.to_string(),
            owner_id,
            ..Default::default()
        }
    }

    pub fn rename(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn clear_route(&mut self) {
        self.op_num = 0;
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub(crate) mod tests {
    use rstest::{fixture, rstest};
    use std::convert::TryFrom;

    use crate::model::user::tests::UserIdFixtures;

    use super::*;

    #[fixture]
    fn route0_without_op() -> RouteInfo {
        RouteInfo::empty_route0(0)
    }

    #[fixture]
    fn route0_op2() -> RouteInfo {
        RouteInfo::empty_route0(2)
    }

    #[rstest]
    fn can_rename(#[from(route0_without_op)] mut info: RouteInfo) {
        info.rename("Renamed!!!");
        assert_eq!(info.name.to_string(), String::from("Renamed!!!"))
    }

    #[rstest]
    fn can_clear(#[from(route0_op2)] mut info: RouteInfo) {
        info.clear_route();
        assert_eq!(info.op_num, 0)
    }

    pub trait RouteInfoFixtures {
        fn empty_route0(op_num: usize) -> RouteInfo {
            RouteInfo {
                id: RouteId::new(),
                name: "route0".into(),
                owner_id: UserId::doncic(),
                op_num,
                ..Default::default()
            }
        }

        fn filled_route0(
            asc_gain: i32,
            desc_gain: i32,
            total_dist: f64,
            op_num: usize,
        ) -> RouteInfo {
            RouteInfo {
                ascent_elevation_gain: Elevation::try_from(asc_gain).unwrap(),
                descent_elevation_gain: Elevation::try_from(desc_gain).unwrap(),
                total_distance: Distance::try_from(total_dist).unwrap(),
                ..Self::empty_route0(op_num)
            }
        }

        fn empty_route1(op_num: usize) -> RouteInfo {
            RouteInfo {
                name: "route1".into(),
                ..Self::empty_route0(op_num)
            }
        }

        fn yokohama_to_chiba() -> RouteInfo {
            RouteInfo::filled_route0(10, 0, 46779.709825324135, 2)
        }

        fn yokohama_to_chiba_via_tokyo() -> RouteInfo {
            RouteInfo::filled_route0(10, 0, 58759.973932514884, 3)
        }

        fn yokohama_to_tokyo() -> RouteInfo {
            RouteInfo::filled_route0(3, 0, 26936.42633640023, 3)
        }
    }

    impl RouteInfoFixtures for RouteInfo {}
}
