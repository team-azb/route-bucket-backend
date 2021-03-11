use crate::domain::coordinate::Coordinate;

#[derive(Debug)]
pub struct Route {
    name: String,
    points: Vec<Coordinate>,
}

impl Route {
    pub fn new(name: &str) -> Route {
        Route {
            name: name.to_string(),
            points: Vec::new()
        }
    }

    pub fn add_point(&mut self, point: Coordinate) {
        self.points.push(point);
    }

    pub fn show_points(&self) {
        println!("{:?}", self.points);
    }
}