use getset::Getters;
use gpx::Metadata;
use serde::{Deserialize, Serialize};

use crate::model::RouteId;

#[derive(Clone, Debug, Getters, Deserialize, Serialize)]
#[get = "pub"]
pub struct RouteInfo {
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
