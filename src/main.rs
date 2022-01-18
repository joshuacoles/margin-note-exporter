mod extractor;
mod item;
mod exporter;
mod oo3;

extern crate serde;
extern crate gray_matter;
extern crate sxd_document;
extern crate sxd_xpath;
extern crate regex;
extern crate lazy_static;
extern crate flate2;
extern crate glob;
extern crate indextree;

use extractor::MarginNotesExtractor;
use crate::exporter::Exporter;
use crate::oo3::OO3File;

// Group in export by Book Title (ie by source)
// Check non-uniqueness of names and compensate (maybe append parent title in ()'s)
fn main() {
    let oo3: OO3File = OO3File::new("./ps1av2.oo3");

    let extractor = MarginNotesExtractor::new();
    let extracted_notes = extractor.read_items(oo3.xml());

    let exporter: Exporter = Exporter::new(
        oo3,
        extracted_notes,
        "/Users/joshuacoles/Library/Mobile Documents/iCloud~md~obsidian/Documents/TestBed/PS",
        "/Users/joshuacoles/Library/Mobile Documents/iCloud~md~obsidian/Documents/TestBed/img"
    );

    exporter.copy_images().expect("Failed to copy images");
    exporter.export_notes().expect("Failed to export notes");
}
