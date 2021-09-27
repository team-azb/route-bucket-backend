use getset::Getters;
use gpx::Metadata;
use serde::{Deserialize, Serialize};

use crate::model::RouteId;

#[cfg(test)]
use derivative::Derivative;

#[derive(Clone, Debug, Getters, Deserialize, Serialize)]
#[get = "pub"]
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(PartialEq))]
pub struct RouteInfo {
    #[cfg_attr(test, derivative(PartialEq = "ignore"))]
    pub(super) id: RouteId,
    pub(super) name: String,
    #[serde(skip_serializing)]
    pub(super) op_num: usize,
}

impl RouteInfo {
    pub fn new(id: RouteId, name: &String, op_num: usize) -> RouteInfo {
        RouteInfo {
            id,
            name: name.clone(),
            op_num,
        }
    }

    pub fn rename(&mut self, name: &String) {
        self.name = name.clone();
    }

    pub fn clear_route(&mut self) {
        self.op_num = 0;
    }
}

impl From<RouteInfo> for Metadata {
    fn from(route_info: RouteInfo) -> Self {
        Self {
            name: Some(route_info.name),
            description: None,
            // TODO: ここにRouteBucketのリンクを入れられると良さそう
            author: None,
            links: vec![],
            time: None,
            keywords: None,
            bounds: None,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use rstest::{fixture, rstest};

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
        info.rename(&"Renamed!!!".into());
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
                op_num,
            }
        }
    }

    impl RouteInfoFixtures for RouteInfo {}
}
