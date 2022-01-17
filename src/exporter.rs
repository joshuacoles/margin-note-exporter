use std::fs;
use std::path::PathBuf;
use crate::{Item, MarginNotes};
use glob::glob;

impl MarginNotes {
    pub fn export_notes_to<P: Into<PathBuf>>(&self, root: P) -> std::io::Result<()> {
        let root = root.into();

        match fs::metadata(&root) {
            Ok(x) if x.is_dir() => (),
            Ok(_) => panic!("Notes directory path exists but is not a file"),
            _ => fs::create_dir(&root)?
        };

        fn recurse(root: &PathBuf, items: &Vec<Item>) -> std::io::Result<()> {
            for item in items {
                std::fs::write(root.join(item.title.clone()).with_extension("md"), item.to_note())?;
                recurse(root, &item.children)?;
            }

            Ok(())
        }

        recurse(&root, &self.root_items)
    }

    pub fn copy_images_to<P: Into<PathBuf>>(&self, root: P) -> std::io::Result<()> {
        let root = root.into();

        match fs::metadata(&root) {
            Ok(x) if x.is_dir() => (),
            Ok(_) => panic!("Images directory path exists but is not a file"),
            _ => fs::create_dir(&root)?
        };

        let images = glob(self.oo3_path.join("*.png").to_str().unwrap()).expect("Failed to read glob pattern");

        for entry in images {
            match entry {
                Ok(path) => {
                    fs::copy(&path, root.join(path.file_name().unwrap()))?;
                }
                Err(e) => println!("Failed {:?}", e),
            };
        }

        Ok(())
    }
}

impl Item {
    pub fn toc(&self) -> String {
        fn toc_recur(item: &Item, indent: usize) -> String {
            let vec: Vec<String> = item.children.iter().map(|child| toc_recur(child, indent + 1)).collect();

            format!("{indent}- [[{title}]]{child_nl}{children}",
                    indent = "  ".repeat(indent),
                    title = item.title,
                    child_nl = if vec.is_empty() { "" } else { "\n" },
                    children = vec.join("\n")
            )
        }

        toc_recur(self, 0)
    }

    fn immediate_toc(&self) -> String {
        self.children.iter().map(|item|
            format!("- [[{}]]", item.title)
        ).collect::<Vec<String>>().join("\n")
    }

    pub fn to_note(&self) -> String {
        let blocks = vec![
            self.margin_note_id.clone().map(|id| format!("---\nmargin-note-id: {}\n---", id)),
            Some(format!("# {}", self.title)),
            self.margin_note_url.clone().map(|url| format!("> [source]({})", url)),
            Some(self.immediate_toc()),
            self.comments.clone(),
            Some(
                self.image_ids.iter()
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
