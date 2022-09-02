use png::OutputInfo;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::{BufReader, Read}};
use zip::ZipArchive;
use bincode::deserialize;


#[derive(Debug, Deserialize)]
struct AtlasRecord {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    name: String
}

impl AtlasRecord {
    fn texture(&self) -> Texture {
        Texture::new(self.x, self.y, self.width, self.height)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Texture{
    nw: [f32; 2],
    ne: [f32; 2],
    se: [f32; 2],
    sw: [f32; 2],
    flipped: bool
}

impl Texture {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Texture{
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

    pub fn nw(&self) -> [f32; 2] {
        if self.flipped {self.ne} else {self.nw}
    }

    pub fn ne(&self) -> [f32; 2] {
        if self.flipped {self.nw} else {self.ne}
    }

    pub fn sw(&self) -> [f32; 2] {
        if self.flipped {self.se} else {self.sw}
    }

    pub fn se(&self) -> [f32; 2] {
        if self.flipped {self.sw} else {self.se}
    }
}


pub struct Atlas {
    records: HashMap<String, Texture>,
    path: String
}

impl Atlas {
    pub fn new(path: &str) -> Atlas {
        let reader = BufReader::new(File::open(path).unwrap());
        let mut zip = ZipArchive::new(reader).unwrap();

        let mut metadata = zip.by_name("atlas.data").unwrap();
        let mut buf = Vec::new();
        metadata.read_to_end(&mut buf).unwrap();

        let records: Vec<AtlasRecord> = deserialize(&buf).unwrap();
        let mut record_map = HashMap::new();

        for r in records {
            record_map.insert(r.name.clone(), r.texture());
        }

        Atlas {
            path: path.to_string(),
            records: record_map
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

    pub fn get(&self, key: &str) -> Texture {
        self.records.get(key).unwrap().clone()
    }
}
