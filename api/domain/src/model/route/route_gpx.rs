use std::convert::TryFrom;
use std::io::{Cursor, Read};
use std::str::from_utf8;

use itertools::Itertools;
use num_traits::FromPrimitive;
use quick_xml::events::Event;
use quick_xml::{Reader, Writer};

use route_bucket_utils::{ApplicationError, ApplicationResult};

use crate::model::route::{
    coordinate::Coordinate, route_info::RouteInfo, segment_list::SegmentList, Route,
};

#[cfg(test)]
use derivative::Derivative;

impl From<Coordinate> for gpx::Waypoint {
    fn from(coord: Coordinate) -> Self {
        let elevation = coord
            .elevation
            .map(|elev| elev.value())
            .map(f64::from_i32)
            .flatten();

        let mut waypoint = Self::new(<(f64, f64)>::from(coord).into());
        waypoint.elevation = elevation;

        waypoint
    }
}

impl From<RouteInfo> for gpx::Metadata {
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

impl From<SegmentList> for gpx::Track {
    fn from(seg_list: SegmentList) -> Self {
        let mut trk = Self::new();
        trk.segments.push(gpx::TrackSegment::new());
        trk.segments[0].points = seg_list
            .segments
            .into_iter()
            .map(|seg| seg.into_iter())
            .flatten()
            .map(gpx::Waypoint::from)
            .collect_vec();
        trk
    }
}

impl From<Route> for gpx::Gpx {
    fn from(route: Route) -> Self {
        gpx::Gpx {
            version: gpx::GpxVersion::Gpx11,
            metadata: Some(route.info.into()),
            tracks: vec![route.seg_list.into()],
            ..Default::default()
        }
    }
}
pub struct RouteGpx {
    name: String,
    data: Vec<u8>,
}

impl RouteGpx {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }
}

impl TryFrom<Route> for RouteGpx {
    type Error = ApplicationError;

    fn try_from(route: Route) -> ApplicationResult<Self> {
        let mut org_gpx_buf = Vec::new();
        let file_name = route.info.name.clone();
        gpx::write(&route.into(), &mut org_gpx_buf).unwrap();

        let mut reader = Reader::from_str(from_utf8(&org_gpx_buf).unwrap());
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        let mut read_buf = Vec::new();

        let to_write_err = |err: quick_xml::Error| {
            ApplicationError::ExternalError(format!("Failed to write gpx event ({:?})", err))
        };

        let mut found_gpx_element = false;

        loop {
            match reader.read_event(&mut read_buf) {
                Ok(Event::Start(mut elem)) if elem.name() == b"gpx" => {
                    elem.extend_attributes([
                        ("xsi:schemaLocation", "http://www.topografix.com/GPX/11.xsd"),
                        ("xmlns", "http://www.topografix.com/GPX/1/1"),
                        ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
                    ]);
                    writer
                        .write_event(Event::Start(elem))
                        .map_err(to_write_err)?;

                    found_gpx_element = true;
                }
                Ok(Event::Eof) => {
                    found_gpx_element
                        .then(|| ())
                        .ok_or(ApplicationError::ExternalError(format!(
                            "Produced GPX didn't contain <gpx> element!
                            {:?}",
                            from_utf8(&org_gpx_buf).unwrap()
                        )))?;
                    break;
                }
                Ok(event) => {
                    writer.write_event(event).map_err(to_write_err)?;
                }
                Err(e) => {
                    return Err(ApplicationError::ExternalError(format!(
                        "Invalid gpx produced by crate gpx! {:?}",
                        e
                    )))
                }
            }
        }

        Ok(Self {
            name: file_name,
            data: writer.into_inner().into_inner(),
        })
    }
}
