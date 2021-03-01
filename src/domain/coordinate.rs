use lombok::Getters;
use std::os::raw::c_uint;

/// A Value Class for Coordinates
#[derive(Clone, Debug, Getters, AllArgsConstructor, PartialEq, Eq)]
pub struct Coordinate {
    latitude: Latitude,
    longitude: Longitude,
}

#[derive(Clone, Debug, Getters, PartialEq, Eq)]
pub struct Latitude(f64);

impl FromF64 for Latitude {
    fn from_f64(val :f64) -> Result<Self, todo!()> {
        // TODO: エラー処理を書く
        if val.abs() <= 90.0 {
            Ok(Latitude(val))
        } else {
            todo!()
        }
    }
}

#[derive(Clone, Debug, Getters, PartialEq, Eq)]
pub struct Longitude(f64);


impl FromF64 for Longitude {
    fn from_f64(val :f64) -> Result<Self, todo!()> {
        // TODO: エラー処理を書く
        if val.abs() <= 180.0 {
            Ok(Longitude(val))
        } else {
            todo!()
        }
    }
}
