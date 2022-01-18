use std::path::PathBuf;
use std::fs;
use flate2::read::GzDecoder;
use sxd_document::Package;
use std::io::Read;
use crate::extractor::parse_xml;

/// Provide utilities to read OO3 files
pub struct OO3File(PathBuf);

impl OO3File {
    pub(crate) fn new<P: Into<PathBuf>>(p: P) -> OO3File {
        let p = p.into();
        assert!(fs::metadata(&p).unwrap().is_dir());
        OO3File(p)
    }
}

impl OO3File {
    pub fn images(&self) -> impl Iterator<Item=PathBuf> {
        glob::glob(self.0.join("*.png").to_str().unwrap())
            .expect("Failed to read glob pattern")
            .map(|ee| ee.unwrap())
    }

    fn xml_raw(&self) -> String {
        let gzip_xml = fs::read(self.0.join("contents.xml")).unwrap();
        let gzip_xml = gzip_xml.as_slice();
        let mut raw_xml = GzDecoder::new(gzip_xml);
        let mut xml = String::new();
        raw_xml.read_to_string(&mut xml).unwrap();
        xml
    }

    pub fn xml(&self) -> Package {
        let xml = self.xml_raw();
        parse_xml(&xml)
    }
}
