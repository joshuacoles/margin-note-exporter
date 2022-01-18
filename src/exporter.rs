use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use gray_matter::Matter;
use gray_matter::engine::YAML;
use glob::glob;
use indextree::{Arena, Node, NodeId};
use crate::{item::Item, MarginNotesExtractor};
use crate::item::NoteFrontMatter;
use crate::oo3::OO3File;

// The main class of the app
pub struct Exporter {
    oo3: OO3File,

    pub note_dir: PathBuf,
    pub image_dir: PathBuf,

    items_arena: Arena<Item>,

    previous_id_map: HashMap<String, PathBuf>,
}

impl Exporter {
    pub(crate) fn new<P1: Into<PathBuf>, P2: Into<PathBuf>>(
        oo3: OO3File,
        note_dir: P1,
        image_dir: P2,
    ) -> Exporter {
        let note_dir = note_dir.into();
        let image_dir = image_dir.into();

        Exporter::validate_dir(&note_dir).unwrap();
        Exporter::validate_dir(&image_dir).unwrap();

        let mut items_arena = Arena::new();

        Exporter {
            oo3,
            items_arena,

            previous_id_map: Exporter::create_id_map(&note_dir),

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

    fn path_of(&self, root: &PathBuf, item: &Item) -> PathBuf {
        root.join(item.title.clone()).with_extension("md")
    }

    fn recurse_export_notes(&self, root: &PathBuf, item_ids: &Vec<NodeId>) -> std::io::Result<()> {
        for item_id in item_ids {
            let node = self.items_arena.get(*item_id).unwrap();
            let item = node.get();

            let previous_path = self.previous_item_path(item);
            let new_path = root.join(item.title.clone()).with_extension("md");

            // Delete old file if we have renamed the note since
            match previous_path {
                Some(previous_path) if previous_path == new_path => std::fs::remove_file(previous_path)?,
                _ => (),
            }

            std::fs::write(new_path, self.note_for(node))?;
            let vec = item_id.children(&self.items_arena).collect();
            self.recurse_export_notes(root, &vec)?;
        }

        Ok(())
    }

    pub fn export_notes(&mut self) -> std::io::Result<()> {
        let mut extractor = MarginNotesExtractor::new(&mut self.items_arena);
        let root_items = extractor.root_items(self.oo3.xml());

        self.recurse_export_notes(&self.note_dir, &root_items)
    }

    fn note_for(&self, node: &Node<Item>) -> String {
        let item = node.get();
        let id = self.items_arena.get_node_id(node).unwrap();

        let blocks = vec![
            // Metadata
            Some(format!("{}---", serde_yaml::to_string(&item.metadata()).unwrap())),

            // Title
            Some(format!("# {}", item.title)),

            // Source link
            item.margin_note_url.clone().map(|url| format!("> [source]({})", url)),

            // Children
            Some(id.children(&self.items_arena).map(|item|
                format!("- [[{}]]", self.items_arena.get(item).unwrap().get().title)
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
}
