use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub struct Item {
    pub title: String,
    pub given_title: Option<String>,
    pub margin_note_id: Option<String>,
    pub margin_note_url: Option<String>,
    pub image_ids: Vec<String>,
    // pub children: Vec<Item>,
    pub comments: Option<String>,
}

impl Item {
    pub fn metadata(&self, parent: Option<&Item>) -> NoteFrontMatter {
        NoteFrontMatter {
            tags: vec!["source-margin-note".to_string()],
            up: parent.map(|parent| vec![format!("[[{}]]", parent.title)]),
            margin_note_id: self.margin_note_id.clone(),
            title: self.given_title.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NoteFrontMatter {
    pub tags: Vec<String>,
    pub margin_note_id: Option<String>,

    // Breadcrumbs
    pub up: Option<Vec<String>>,
    pub title: Option<String>,
}
