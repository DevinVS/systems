use png::OutputInfo;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::{BufReader, Read}};
use zip::ZipArchive;
use bincode::deserialize;

#[derive(Debug, Deserialize)]
struct AtlasRecord {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    name: String
}

#[derive(Debug, Deserialize)]
struct AtlasData {
    records: Vec<AtlasRecord>,
    width: u32,
    height: u32,
}

impl AtlasRecord {
    fn texture(&self) -> Texture {
        Texture::new(self.x, self.y, self.width, self.height)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Texture{
    nw: [u32; 2],
    ne: [u32; 2],
    se: [u32; 2],
    sw: [u32; 2],
    flipped: bool
}

impl Texture {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Texture{
        Texture {
            nw: [x, y],
            ne: [x + width, y],
            se: [x + width, y + height],
            sw: [x, y + height],
            flipped: false
        }
    }

    pub fn set_flipped(&mut self, flipped: bool) {
        self.flipped = flipped;
    }

    pub fn nw(&self) -> [u32; 2] {
        if self.flipped { self.ne } else { self.nw }
    }

    pub fn ne(&self) -> [u32; 2] {
        if self.flipped { self.nw } else { self.ne }
    }

    pub fn sw(&self) -> [u32; 2] {
        if self.flipped { self.se } else { self.sw }
    }

    pub fn se(&self) -> [u32; 2] {
        if self.flipped { self.sw } else { self.se }
    }

    pub fn x(&self) -> u32 { self.nw[0] }
    pub fn y(&self) -> u32 { self.nw[1] }
    pub fn width(&self) -> u32 { self.ne[0] - self.nw[0] }
    pub fn height(&self) -> u32 { self.se[1] - self.ne[1] }
}


pub struct Atlas {
    records: HashMap<String, Texture>,
    path: String,
    pub width: u32,
    pub height: u32
}

impl Atlas {
    pub fn new(path: &str) -> Atlas {
        let reader = BufReader::new(File::open(path).unwrap());
        let mut zip = ZipArchive::new(reader).unwrap();

        let mut metadata = zip.by_name("atlas.data").unwrap();

        let mut buf = Vec::new();
        metadata.read_to_end(&mut buf).unwrap();

        let data: AtlasData = deserialize(&buf).unwrap();

        let mut record_map = HashMap::new();

        for r in data.records {
            record_map.insert(r.name.clone(), r.texture());
        }

        Atlas {
            path: path.to_string(),
            records: record_map,
            width: data.width,
            height: data.height
        }
    }

    pub fn image_data(&self) -> (OutputInfo, Vec<u8>) {
        let reader = BufReader::new(File::open(self.path.clone()).unwrap());
        let mut zip = ZipArchive::new(reader).unwrap();

        let mut file = zip.by_name("atlas.png").unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();

        let decoder = png::Decoder::new(std::io::Cursor::new(buf));
        let mut reader = decoder.read_info().unwrap();

        let mut image_data = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut image_data).unwrap();

        (info, image_data)
    }

    pub fn get(&self, key: &str) -> Option<Texture> {
        self.records.get(key).map(|t| t.clone())
    }
}
