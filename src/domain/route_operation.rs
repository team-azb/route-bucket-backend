use crate::domain::coordinate::Coordinate;
use crate::domain::route::Route;
use crate::utils::error::{ApplicationError, ApplicationResult};

pub enum RouteOperation {
    Add { pos: usize, coord: Coordinate },
    Remove { pos: usize, coord: Coordinate },
    Clear { org_list: Vec<Coordinate> },
    // reverse operation for Clear
    InitWithList { list: Vec<Coordinate> },
}

impl RouteOperation {
    pub fn apply(&self, route: &mut Route) -> ApplicationResult<()> {
        match self {
            Self::Add { pos, coord } => Ok(route.insert_point(*pos, coord.clone())?),
            Self::Remove { pos, coord } => {
                let ref removed = route.remove_point(*pos)?;
                (coord == removed)
                    .then(|| ())
                    .ok_or(ApplicationError::DomainError("Contradiction on remove"))
            }
            Self::Clear { org_list } => {
                let ref removed_list = route.clear_points();
                (org_list == removed_list)
                    .then(|| ())
                    .ok_or(ApplicationError::DomainError("Contradiction on clear"))
            }
            Self::InitWithList { list } => Ok(route.init_points(list.clone())?),
        }
    }

    pub fn reverse(&self) -> RouteOperation {
        match self {
            Self::Add { pos, coord } => Self::Remove {
                pos: *pos,
                coord: coord.clone(),
            },
            Self::Remove { pos, coord } => Self::Add {
                pos: *pos,
                coord: coord.clone(),
            },
            Self::Clear { org_list } => Self::InitWithList {
                list: org_list.clone(),
            },
            Self::InitWithList { list } => Self::Clear {
                org_list: list.clone(),
            },
        }
    }
}
