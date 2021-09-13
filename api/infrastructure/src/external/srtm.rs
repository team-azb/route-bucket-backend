use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::ops::Range;
use std::path::{Path, PathBuf};

use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use num_traits::FromPrimitive;

use route_bucket_domain::external::ElevationApi;
use route_bucket_domain::model::{Coordinate, Elevation, Latitude, Longitude};
use route_bucket_utils::{ApplicationError, ApplicationResult};

#[derive(num_derive::FromPrimitive)]
enum SrtmByteOrder {
    LittleEndian = 0x4949,
    BigEndian = 0x4D4D,
}

trait GetSrtmByteOrder {
    fn get_srtm_byte_order() -> SrtmByteOrder;
}

impl GetSrtmByteOrder for BigEndian {
    fn get_srtm_byte_order() -> SrtmByteOrder {
        SrtmByteOrder::BigEndian
    }
}
impl GetSrtmByteOrder for LittleEndian {
    fn get_srtm_byte_order() -> SrtmByteOrder {
        SrtmByteOrder::LittleEndian
    }
}

#[derive(Clone, Debug, num_derive::FromPrimitive, Eq, PartialEq, Hash)]
enum IfdTag {
    ImageWidth = 0x0100,
    ImageHeight = 0x0101,
    StripOffsets = 0x0111,
    // RowsPerStrip = 0x0116,
    // StripByteCounts = 0x0117,
    ModelPixelScale = 0x830E,
    ModelTiepoint = 0x8482,
    NoDataValue = 0xA481,
}

#[derive(Debug)]
struct IfdEntry {
    datatype: u16,
    count: u32,
    data: u32,
}

impl IfdEntry {
    pub fn read<Endian: ByteOrder>(f: &mut File) -> std::io::Result<Self> {
        let datatype = f.read_u16::<Endian>()?;
        let count = f.read_u32::<Endian>()?;
        let data = f.read_u32::<Endian>()?;
        Ok(Self {
            datatype,
            count,
            data,
        })
    }
}

/// struct to process srtm 30x30 GeoTIFF files from https://srtm.csi.cgiar.org/
struct SrtmFile {
    path: PathBuf,
    byte_order: SrtmByteOrder,
    lat_range: Range<Latitude>,
    lon_range: Range<Longitude>,
    pixel_scale: Coordinate,
    strip_offsets: Vec<u32>,
    no_data_value: Elevation,
}

impl SrtmFile {
    pub fn open(path: &Path) -> ApplicationResult<Self> {
        let mut path_buf = PathBuf::new();
        path_buf.push(path);

        let mut file =
            File::open(path).map_err(Self::cvt_err(format!("Failed to open {:?}", path)))?;

        let (byte_order, ifd_offset) = Self::read_header(&mut file).map_err(Self::cvt_err(
            format!("Failed to read header from {:?}", path),
        ))?;

        let ifd_entries = match byte_order {
            SrtmByteOrder::LittleEndian => Self::read_ifd::<LittleEndian>(&mut file, ifd_offset),
            SrtmByteOrder::BigEndian => Self::read_ifd::<BigEndian>(&mut file, ifd_offset),
        }
        .map_err(Self::cvt_err(format!(
            "Failed to read IFD entries from {:?}",
            path
        )))?;

        match byte_order {
            SrtmByteOrder::LittleEndian => {
                Self::from_ifd::<LittleEndian>(&ifd_entries, path_buf, file)
            }
            SrtmByteOrder::BigEndian => Self::from_ifd::<BigEndian>(&ifd_entries, path_buf, file),
        }
    }

    pub fn get(&self, coord: &Coordinate) -> ApplicationResult<Option<Elevation>> {
        let mut file = File::open(&self.path)
            .map_err(Self::cvt_err(format!("Failed to open {:?}", self.path)))?;

        let (lon_scale, lat_scale): (f64, f64) = self.pixel_scale.clone().into();
        let (lon, lat): (f64, f64) = coord.clone().into();

        let lon_idx = ((lon - self.lon_range.start.value()) / lon_scale) as u32;
        let lat_idx = ((self.lat_range.end.value() - lat) / lat_scale) as usize;

        let offset = (self.strip_offsets[lat_idx] + lon_idx * 2) as u64;

        file.seek(SeekFrom::Start(offset))
            .map_err(Self::cvt_err(format!(
                "failed to seek to offset {}",
                offset
            )))?;

        // TODO: 毎回読まずにLRU cacheを用意する
        //     : 行ごと読んで取っておくのは有効かも
        let data: i32 = match self.byte_order {
            SrtmByteOrder::LittleEndian => file.read_i16::<LittleEndian>(),
            SrtmByteOrder::BigEndian => file.read_i16::<BigEndian>(),
        }
        .map_err(Self::cvt_err(format!(
            "Failed to read elevation of {:?} from {:?}",
            coord, self.path
        )))?
        .into();

        // TODO: then_someが実装されたら置き換える https://github.com/rust-lang/rust/issues/64260
        Ok((data != self.no_data_value.value()).then(|| data.try_into().unwrap()))
    }

    pub fn contains(&self, coord: &Coordinate) -> bool {
        self.lat_range.contains(coord.latitude()) && self.lon_range.contains(coord.longitude())
    }

    fn read_header(f: &mut File) -> std::io::Result<(SrtmByteOrder, u32)> {
        f.seek(SeekFrom::Start(0u64))?;

        let byte_order = SrtmByteOrder::from_u16(f.read_u16::<LittleEndian>()?).ok_or(
            std::io::Error::new(std::io::ErrorKind::Other, "invalid SRTM byte_order"),
        )?;

        let version = f.read_u16::<LittleEndian>()?;

        (0x2A == version).then(|| ()).ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("invalid SRTM version {:X}", version),
        ))?;

        let ifd_offset = match byte_order {
            SrtmByteOrder::LittleEndian => f.read_u32::<LittleEndian>(),
            SrtmByteOrder::BigEndian => f.read_u32::<BigEndian>(),
        }?;

        Ok((byte_order, ifd_offset))
    }

    fn read_ifd<Endian: ByteOrder>(
        f: &mut File,
        offset: u32,
    ) -> std::io::Result<HashMap<IfdTag, IfdEntry>> {
        f.seek(SeekFrom::Start(offset as u64))?;

        let entry_count = f.read_u16::<Endian>()?;
        let mut entries = HashMap::new();

        for _ in 0..entry_count {
            let tag_opt = IfdTag::from_u16(f.read_u16::<Endian>()?);
            let entry = IfdEntry::read::<Endian>(f)?;
            if let Some(tag) = tag_opt {
                entries.insert(tag, entry);
            }
        }

        let next_offset = f.read_u32::<Endian>()?;
        if next_offset != 0 {
            Self::read_ifd::<Endian>(f, next_offset)?
                .into_iter()
                .for_each(|(k, v)| {
                    entries.insert(k, v);
                });
        }

        Ok(entries)
    }

    fn from_ifd<Endian: ByteOrder + GetSrtmByteOrder>(
        entries: &HashMap<IfdTag, IfdEntry>,
        path: PathBuf,
        mut file: File,
    ) -> ApplicationResult<Self> {
        let read_tag_data = |tag: IfdTag| -> ApplicationResult<&IfdEntry> {
            entries
                .get(&tag)
                .ok_or(ApplicationError::ExternalError(format!(
                    "Failed to find tag {:?}",
                    tag
                )))
        };
        let width = read_tag_data(IfdTag::ImageWidth)?.data;
        let height = read_tag_data(IfdTag::ImageHeight)?.data;

        let strip_offsets_entry = read_tag_data(IfdTag::StripOffsets)?;
        let strip_offsets = Self::read_strip_offsets::<Endian>(strip_offsets_entry, &mut file)
            .map_err(Self::cvt_err("Failed to read strip offsets".into()))?;

        let pixel_scale_entry = read_tag_data(IfdTag::ModelPixelScale)?;
        let [pixel_scale_lon, pixel_scale_lat] =
            Self::read_pixel_scale::<Endian>(pixel_scale_entry, &mut file)
                .map_err(Self::cvt_err("Failed to read strip offsets".into()))?;

        let tiepoint_offset_entry = read_tag_data(IfdTag::ModelTiepoint)?;
        let [left_lon, up_lat] = Self::read_tiepoint::<Endian>(tiepoint_offset_entry, &mut file)
            .map_err(Self::cvt_err("Failed to read strip offsets".into()))?;
        let bottom_lat = up_lat - pixel_scale_lat * height as f64;
        let right_lon = left_lon + pixel_scale_lon * width as f64;

        let no_data_entry = read_tag_data(IfdTag::NoDataValue)?;
        let no_data_value = Self::read_no_data_value(no_data_entry, &mut file)
            .map_err(Self::cvt_err("Failed to read NO_DATA value".into()))?
            .try_into()?;

        Ok(Self {
            path,
            byte_order: Endian::get_srtm_byte_order(),
            lat_range: Latitude::try_from(bottom_lat)?..Latitude::try_from(up_lat)?,
            lon_range: Longitude::try_from(left_lon)?..Longitude::try_from(right_lon)?,
            pixel_scale: Coordinate::new(pixel_scale_lat, pixel_scale_lon)?,
            strip_offsets,
            no_data_value,
        })
    }

    fn read_strip_offsets<Endian: ByteOrder>(
        entry: &IfdEntry,
        file: &mut File,
    ) -> std::io::Result<Vec<u32>> {
        let mut strip_offsets = vec![0u32; entry.count as usize];
        file.seek(SeekFrom::Start(entry.data as u64))?;
        file.read_u32_into::<Endian>(&mut strip_offsets[..])?;
        Ok(strip_offsets)
    }

    fn read_pixel_scale<Endian: ByteOrder>(
        entry: &IfdEntry,
        file: &mut File,
    ) -> std::io::Result<[f64; 2]> {
        let mut pixel_scale = [0f64; 2];
        file.seek(SeekFrom::Start(entry.data as u64))?;
        file.read_f64_into::<Endian>(&mut pixel_scale)?;
        Ok(pixel_scale)
    }

    fn read_tiepoint<Endian: ByteOrder>(
        entry: &IfdEntry,
        file: &mut File,
    ) -> std::io::Result<[f64; 2]> {
        let mut tiepoint = [0f64; 6];
        file.seek(SeekFrom::Start(entry.data as u64))?;
        file.read_f64_into::<Endian>(&mut tiepoint)?;
        Ok(tiepoint[3..5].try_into().unwrap())
    }

    fn read_no_data_value(entry: &IfdEntry, file: &mut File) -> std::io::Result<i32> {
        let mut buf = vec![0u8; (entry.count - 1) as usize];
        file.seek(SeekFrom::Start(entry.data as u64))?;
        file.read(&mut buf[..])?;
        Ok(std::str::from_utf8(&buf).unwrap().parse::<i32>().unwrap())
    }

    /// Returns a closure that converts io::Error to ApplicationError
    fn cvt_err(msg: String) -> Box<dyn Fn(std::io::Error) -> ApplicationError> {
        Box::new(move |err| ApplicationError::ExternalError(format!("{} ({})", msg, err)))
    }
}

/// struct to search coordinate from multiple SrtmFiles
pub struct SrtmReader {
    files: Vec<SrtmFile>,
}

impl SrtmReader {
    pub fn new() -> ApplicationResult<Self> {
        // TODO: 複数ファイル対応
        Ok(Self {
            files: vec![SrtmFile::open(Path::new("resources/srtm_data/srtm.tif"))?],
        })
    }
}

impl ElevationApi for SrtmReader {
    fn get_elevation(&self, coord: &Coordinate) -> ApplicationResult<Option<Elevation>> {
        for area in self.files.iter() {
            if area.contains(coord) {
                return area.get(coord);
            }
        }
        return Ok(None);
    }
}
