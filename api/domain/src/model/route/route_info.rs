use chrono::{DateTime, Utc};
use derivative::Derivative;
use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::model::user::UserId;

use super::{Distance, Elevation, RouteId};

#[derive(Clone, Debug, Getters, Derivative, Deserialize, Serialize)]
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
    #[derivative(Default(value = "Utc::now()"))]
    pub(super) created_at: DateTime<Utc>,
    #[derivative(Default(value = "Utc::now()"))]
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

    pub fn with_details(
        id: RouteId,
        name: &str,
        owner_id: UserId,
        op_num: usize,
        ascent_elevation_gain: Elevation,
        descent_elevation_gain: Elevation,
        total_distance: Distance,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> RouteInfo {
        RouteInfo {
            id,
            name: name.to_string(),
            owner_id,
            op_num,
            ascent_elevation_gain,
            descent_elevation_gain,
            total_distance,
            created_at,
            updated_at,
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

    use crate::model::user::tests::UserIdFixtures;

    use super::*;

    #[fixture]
    fn route0_without_op() -> RouteInfo {
        RouteInfo::route0(0)
    }

    #[fixture]
    fn route0_op2() -> RouteInfo {
        RouteInfo::route0(2)
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
        fn route0(op_num: usize) -> RouteInfo {
            RouteInfo {
                id: RouteId::new(),
                name: "route0".into(),
                owner_id: UserId::doncic(),
                op_num,
            }
        }

        fn route1(op_num: usize) -> RouteInfo {
            RouteInfo {
                id: RouteId::new(),
                name: "route1".into(),
                owner_id: UserId::doncic(),
                op_num,
            }
        }
    }

    impl RouteInfoFixtures for RouteInfo {}
}
