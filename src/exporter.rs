use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use gray_matter::Matter;
use gray_matter::engine::YAML;
use glob::glob;
use indextree::{Node};
use lazy_static::lazy_static;
use regex::Regex;
use crate::item::Item;
use crate::extractor::ExtractedNotes;
use crate::item::NoteFrontMatter;
use crate::oo3::OO3File;

// The main class of the app
pub struct Exporter {
    oo3: OO3File,

    pub note_dir: PathBuf,
    pub image_dir: PathBuf,

    notes: ExtractedNotes,

    previous_id_map: HashMap<String, PathBuf>,
    note_name_map: HashMap<String, String>,
}

impl Exporter {
    fn initial_note_name(item: &Item) -> String {
        lazy_static! {
            static ref PATH_SAFE_REGEX: Regex = Regex::new("/").unwrap();
        }

        let clean_name = if let Some(given_title) = &item.given_title {
            PATH_SAFE_REGEX.replace_all(given_title, "(or)").to_string()
        } else if let Some(margin_note_id) = &item.margin_note_id {
            margin_note_id.clone()
        } else {
            panic!("No way to determine note title for {:?}", item);
        };

        format!("(LIT) {}", clean_name)
    }

    fn create_node_name_map(notes: &ExtractedNotes) -> HashMap<String, String> {
        let mut out = HashMap::new();

        for node in notes.items_arena.iter() {
            let item = node.get();
            if let Some(id) = &item.margin_note_id {
                let mut name = Self::initial_note_name(item);

                while out.values().find(|v| **v == name).is_some() {
                    if let Some(parent) = node.parent() {
                        let parent = notes.items_arena.get(parent).unwrap().get();
                        name.push_str(&format!(" ({})", Self::initial_note_name(parent)));
                    } else {
                        println!("Unable to disambiguate note names for {:?}", item);
                    }
                }

                out.insert(id.clone(), name);
            } else {
                continue;
            }
        }

        out
    }
}

impl Exporter {
    pub(crate) fn new<P1: Into<PathBuf>, P2: Into<PathBuf>>(
        oo3: OO3File,
        notes: ExtractedNotes,
        note_dir: P1,
        image_dir: P2,
    ) -> Exporter {
        let note_dir = note_dir.into();
        let image_dir = image_dir.into();

        Exporter::validate_dir(&note_dir).unwrap();
        Exporter::validate_dir(&image_dir).unwrap();

        let previous_id_map = Exporter::create_previous_id_map(&note_dir);
        let note_name_map = Exporter::create_node_name_map(&notes);

        Exporter {
            oo3,
            notes,

            previous_id_map,
            note_name_map,

            note_dir,
            image_dir,
        }
    }

    fn validate_dir(dir: &Path) -> std::io::Result<()> {
        match fs::metadata(&dir) {
            Ok(x) if x.is_dir() => Ok(()),
            Ok(_) => panic!("Directory path exists but is not a file"),
            _ => fs::create_dir_all(&dir)
        }
    }

    fn create_previous_id_map(note_dir: &Path) -> HashMap<String, PathBuf> {
        let note_glob = note_dir.join("*.md");
        let note_glob = note_glob.to_str().expect("Invalid note_dir path");
        let contents = glob(note_glob).expect("Failed to read note dir");
        let matter = Matter::<YAML>::new();

        contents
            .filter_map(|entry| {
                let path = entry.unwrap().to_path_buf();
                let NoteFrontMatter { margin_note_id, .. } = matter.parse(&fs::read_to_string(&path)
                    .expect(&format!("Failed to read note, {}", path.to_string_lossy())))
                    .data.and_then(|data| data.deserialize().ok())?;

                margin_note_id.map(|margin_note_id| (margin_note_id, path))
            }).collect()
    }

    pub fn copy_images(&self) -> std::io::Result<()> {
        let images = self.oo3.images();

        for path in images {
            fs::copy(&path, self.image_dir.join(path.file_name().unwrap()))?;
        }

        Ok(())
    }

    fn previous_item_path(&self, item: &Item) -> Option<PathBuf> {
        item.margin_note_id.clone().and_then(|margin_note_id| self.previous_id_map.get(&margin_note_id).cloned())
    }

    fn note_name_of(&self, item: &Item) -> String {
        if let Some(margin_note_id) = &item.margin_note_id {
            self.note_name_map.get(margin_note_id).unwrap().clone()
        } else {
            Self::initial_note_name(item)
        }
    }

    fn path_of(&self, root: &PathBuf, item: &Item) -> PathBuf {
        root.join(self.note_name_of(item)).with_extension("md")
    }

    fn export_all_notes(&self) -> std::io::Result<()> {
        let root: &PathBuf = &self.note_dir;

        for item_node in self.notes.items_arena.iter() {
            let item = item_node.get();

            let previous_path = self.previous_item_path(item);
            let new_path = self.path_of(root, item);

            // Delete old file if we have renamed the note since
            // TODO Handle title overlaps or at least warn
            match previous_path {
                Some(previous_path) if previous_path == new_path => std::fs::remove_file(previous_path)?,
                _ => (),
            }

            std::fs::write(new_path.clone(), self.note_for(item_node)).expect(&format!("Failed to write file {}", new_path.clone().to_string_lossy()));
        }

        Ok(())
    }

    pub fn export_notes(&self) -> std::io::Result<()> {
        self.export_all_notes()
    }

    fn note_for(&self, node: &Node<Item>) -> String {
        let item = node.get();
        let id = self.notes.items_arena.get_node_id(node).unwrap();

        let parent = node.parent()
            .and_then(|parent| self.notes.items_arena.get(parent))
            .map(|x| x.get());

        let blocks = vec![
            // Metadata
            Some(format!("{}---", serde_yaml::to_string(&item.metadata(parent.map(|parent| self.note_name_of(parent)))).unwrap())),

            // Title
            item.given_title.as_ref().map(|given_title| format!("# {}", given_title)),

            // Source link
            item.margin_note_url().map(|url| format!("> [source]({})", url)),

            // Children
            Some(id.children(&self.notes.items_arena).map(|item|
                format!("- [[{}]]", self.note_name_of(self.notes.items_arena.get(item).unwrap().get()))
            ).collect::<Vec<String>>().join("\n")),

            // Comments
            item.comments.clone(),

            // Images
            Some(
                item.image_ids.iter()
                    .map(|image_id| format!("![[{}.png]]", image_id))
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
        ];

        blocks.iter()
            .filter_map(|v| v.clone())
            .collect::<Vec<String>>()
            .join("\n\n")
    }

    // fn extract_text(&self, image_id: &String) -> String {
    //     let image_path = self.image_dir.join(image_id).with_extension("png");
    //     tesseract::ocr(image_path.to_str().unwrap(), "en-GB").expect("Failed to OCR")
    // }
}
