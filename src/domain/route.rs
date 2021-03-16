use crate::domain::coordinate::Coordinate;
use crate::domain::types::RouteId;
use crate::lib::error::ApplicationResult;

#[derive(Debug)]
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
            points
        }
    }

    pub fn add_point(&mut self, point: Coordinate) {
        self.points.push(point);
    }

    pub fn show_points(&self) {
        println!("{:?}", self.points);
    }
}