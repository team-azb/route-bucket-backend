use std::convert::TryFrom;
use std::io::{Cursor, Read};
use std::str::from_utf8;

use quick_xml::events::Event;
use quick_xml::{Reader, Writer};

use crate::domain::model::route::Route;
use crate::utils::error::{ApplicationError, ApplicationResult};

pub struct RouteGpx(Vec<u8>);

impl RouteGpx {
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl TryFrom<Route> for RouteGpx {
    type Error = ApplicationError;

    fn try_from(route: Route) -> ApplicationResult<Self> {
        let mut org_gpx_buf = Vec::new();
        gpx::write(&route.into(), &mut org_gpx_buf).unwrap();

        println!("{}", from_utf8(&org_gpx_buf).unwrap());

        let mut reader = Reader::from_str(from_utf8(&org_gpx_buf).unwrap());
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        let mut read_buf = Vec::new();

        let to_write_err = |err: quick_xml::Error| {
            ApplicationError::ExternalError(format!("Failed to write gpx event ({:?})", err))
        };

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
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(ApplicationError::ExternalError(format!(
                        "Produced GPX didn't contain <gpx> element!\n{:?}",
                        from_utf8(&org_gpx_buf).unwrap()
                    )))
                }
                Err(e) => {
                    return Err(ApplicationError::ExternalError(format!(
                        "Invalid gpx produced by crate gpx! {:?}",
                        e
                    )))
                }
                Ok(_) => {}
            }
        }
        reader
            .into_underlying_reader()
            .read_to_end(&mut read_buf)
            .unwrap();
        writer.write(&read_buf).map_err(to_write_err)?;

        Ok(Self(writer.into_inner().into_inner()))
    }
}
