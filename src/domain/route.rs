use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::domain::coordinate::Coordinate;
use crate::domain::types::RouteId;
use crate::lib::error::ApplicationResult;

#[derive(Debug, Getters, Deserialize, Serialize)]
#[get = "pub"]
pub struct Route {
    id: RouteId,
    name: String,
    points: Vec<Coordinate>,
}

impl Route {
    pub fn new(id: RouteId, name: String, points: Vec<Coordinate>) -> Route {
        Route {
            id,
            name: name.to_string(),
            points,
        }
    }

    pub fn add_point(&mut self, point: Coordinate) {
        self.points.push(point);
    }
}

pub trait RouteRepository {
    fn find(&self, id: &RouteId) -> ApplicationResult<Route>;

    fn register(&self, route: &Route) -> ApplicationResult<()>;

    // TODO: こいつrepositoryではなくてusecase説ある
    fn create(&self, name: &String) -> ApplicationResult<RouteId> {
        let route = Route {
            id: RouteId::new(),
            name: name.clone(),
            points: Vec::new(),
        };
        self.register(&route)?;
        Ok(route.id)
    }
}
