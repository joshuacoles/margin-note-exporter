use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Item {
    pub title: String,
    pub margin_note_id: Option<String>,
    pub margin_note_url: Option<String>,
    pub image_ids: Vec<String>,
    pub children: Vec<Item>,
    pub comments: Option<String>,
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
            Some(format!("{}---", serde_yaml::to_string(&self.metadata()).unwrap())),
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
