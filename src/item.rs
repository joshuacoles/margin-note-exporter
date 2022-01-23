use serde::{Deserialize, Serialize};

enum ItemContentBlock {
    Text { text: String },
    Image { image_id: String },
}

#[derive(Debug, PartialEq)]
pub struct Item {
    // A title if the user added one
    pub given_title: Option<String>,
    // Note we do not have a margin note id for groups
    pub margin_note_id: Option<String>,

    pub image_ids: Vec<String>,
    pub comments: Option<String>,
}

impl Item {
    pub fn metadata(&self, parent_note_name: Option<String>) -> NoteFrontMatter {
        NoteFrontMatter {
            title: self.given_title.clone(),
            tags: vec!["source-margin-note".to_string()],
            up: parent_note_name.map(|parent_note_name| vec![format!("[[{}]]", parent_note_name)]),
            margin_note_id: self.margin_note_id.clone(),
        }
    }

    pub fn margin_note_url(&self) -> Option<String> {
        self.margin_note_id.clone().map(|margin_note_id| format!("marginnote3app://note/{}", margin_note_id))
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
