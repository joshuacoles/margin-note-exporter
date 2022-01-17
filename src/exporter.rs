use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use flate2::read::GzDecoder;
use gray_matter::Matter;
use gray_matter::engine::YAML;
use serde::Deserialize;
use glob::glob;
use sxd_document::Package;
use crate::extractor::parse_xml;
use crate::{item::Item, MarginNotesExtractor};

pub struct OO3(PathBuf);

impl OO3 {
    pub(crate) fn new<P: Into<PathBuf>>(p: P) -> OO3 {
        let p = p.into();
        fs::metadata(&p).unwrap();
        OO3(p)
    }
}

impl OO3 {
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

pub struct Exporter {
    pub note_dir: PathBuf,
    pub image_dir: PathBuf,

    oo3: OO3,
    id_map: HashMap<String, PathBuf>,
}

#[derive(Deserialize, Debug)]
struct NoteFrontMatter {
    margin_note_id: String,
}

impl Exporter {
    pub(crate) fn new<P1: Into<PathBuf>, P2: Into<PathBuf>>(
        oo3: OO3,
        note_dir: P1,
        image_dir: P2,
    ) -> Exporter {
        let note_dir = note_dir.into();
        let image_dir = image_dir.into();

        Exporter::validate_dir(&note_dir).unwrap();
        Exporter::validate_dir(&image_dir).unwrap();

        Exporter {
            oo3,
            id_map: Exporter::create_id_map(&note_dir),

            note_dir,
            image_dir,
        }
    }

    fn validate_dir(dir: &Path) -> std::io::Result<()> {
        match fs::metadata(&dir) {
            Ok(x) if x.is_dir() => Ok(()),
            Ok(_) => panic!("Images directory path exists but is not a file"),
            _ => fs::create_dir(&dir)
        }
    }

    fn create_id_map(note_dir: &Path) -> HashMap<String, PathBuf> {
        let note_glob = note_dir.join("*.md");
        let note_glob = note_glob.to_str().expect("Invalid note_dir path");
        let contents = glob(note_glob).expect("Failed to read note dir");
        let matter = Matter::<YAML>::new();

        contents
            .filter_map(|entry| {
                let path = entry.unwrap().to_path_buf();
                let NoteFrontMatter { margin_note_id } = matter.parse(&fs::read_to_string(&path)
                    .expect(&format!("Failed to read note, {}", path.to_string_lossy())))
                    .data.and_then(|data| data.deserialize().ok())?;

                Some((margin_note_id, path))
            }).collect()
    }

    pub fn copy_images(&self) -> std::io::Result<()> {
        let images = self.oo3.images();

        for path in images {
            fs::copy(&path, self.image_dir.join(path.file_name().unwrap()))?;
        }

        Ok(())
    }

    pub fn export_notes(&self) -> std::io::Result<()> {
        fn recurse(root: &PathBuf, items: &Vec<Item>) -> std::io::Result<()> {
            for item in items {
                std::fs::write(root.join(item.title.clone()).with_extension("md"), item.to_note())?;
                recurse(root, &item.children)?;
            }

            Ok(())
        }

        let extractor = MarginNotesExtractor::new();
        let root_items = extractor.root_items(self.oo3.xml());

        recurse(&self.note_dir, &root_items)
    }
}
