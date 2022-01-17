mod extractor;
mod exporter;
mod exporter2;

extern crate serde;
extern crate gray_matter;
extern crate sxd_document;
extern crate sxd_xpath;
extern crate regex;
extern crate lazy_static;
extern crate flate2;
extern crate glob;

use flate2::read::GzDecoder;

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use extractor::MarginNotesExtractor;
use crate::exporter2::{Exporter, OO3};
use crate::extractor::parse_xml;

struct MarginNotes {
    oo3_path: PathBuf,
    root_items: Vec<Item>
}

#[derive(Debug)]
pub struct Item {
    title: String,
    margin_note_id: Option<String>,
    margin_note_url: Option<String>,
    image_ids: Vec<String>,
    children: Vec<Item>,
    comments: Option<String>,
}

impl MarginNotes {
    fn new<P: AsRef<Path>>(oo3_path: P) -> MarginNotes {
        let vec = fs::read(oo3_path.as_ref().join("contents.xml")).unwrap();
        let gzip_xml = vec.as_slice();
        let mut raw_xml = GzDecoder::new(gzip_xml);
        let mut s = String::new();
        raw_xml.read_to_string(&mut s).unwrap();

        let extractor = MarginNotesExtractor::new();

        MarginNotes {
            oo3_path: oo3_path.as_ref().to_path_buf(),
            root_items: extractor.root_items(parse_xml(&s)),
        }
    }
}

// Group in export by Book Title (ie by source)
// Check non-uniqueness of names and compensate (maybe append parent title in ()'s)
fn main() {
    let oo3: OO3 = OO3::new("./ps1av2.oo3");

    let exporter: Exporter = Exporter::new(
        oo3,
        "/Users/joshuacoles/Library/Mobile Documents/iCloud~md~obsidian/Documents/TestBed/PS",
        "/Users/joshuacoles/Library/Mobile Documents/iCloud~md~obsidian/Documents/TestBed/img"
    );

    exporter.copy_images();
    exporter.export_notes();
}
