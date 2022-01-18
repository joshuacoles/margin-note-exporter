use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub struct Item {
    pub title: String,
    pub margin_note_id: Option<String>,
    pub margin_note_url: Option<String>,
    pub image_ids: Vec<String>,
    // pub children: Vec<Item>,
    pub comments: Option<String>,
}

impl Item {
    pub fn metadata(&self) -> NoteFrontMatter {
        NoteFrontMatter {
            tags: vec!["source-margin-note".to_string()],
            margin_note_id: self.margin_note_id.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NoteFrontMatter {
    pub tags: Vec<String>,
    pub margin_note_id: Option<String>,
}
