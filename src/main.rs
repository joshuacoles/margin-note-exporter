mod extractor;
mod item;
mod exporter;
mod oo3;
mod image_operations;

extern crate serde;
extern crate gray_matter;
extern crate sxd_document;
extern crate sxd_xpath;
extern crate regex;
extern crate lazy_static;
extern crate flate2;
extern crate glob;
extern crate indextree;
// extern crate tesseract;

use extractor::MarginNotesExtractor;
use crate::exporter::Exporter;
use crate::oo3::OO3File;

// Tags
// Values[0] and note seem to be the same concept split over two areas
// Extract header from Values[0] then combine other content
// Look for links (I think based on background colour)
    // text-background-color
// Also images can be interspersed with comments / links

// Group in export by Book Title (ie by source)
// Check non-uniqueness of names and compensate (maybe append parent title in ()'s)
fn main() {
    let oo3: OO3File = OO3File::new("/Users/joshuacoles/Downloads/PS1A23.oo3");

    let extractor = MarginNotesExtractor::new();
    let extracted_notes = extractor.read_items(oo3.xml());

    let exporter: Exporter = Exporter::new(
        oo3,
        extracted_notes,
        "/Users/joshuacoles/Library/Mobile Documents/iCloud~md~obsidian/Documents/STEM/Probability & Stats 1A (Literature)",
        "/Users/joshuacoles/Library/Mobile Documents/iCloud~md~obsidian/Documents/STEM/Probability & Stats 1A (Images)"
    );

    exporter.copy_images().expect("Failed to copy images");
    exporter.export_notes().expect("Failed to export notes");
}
