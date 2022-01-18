use indextree::{Arena, NodeId};
use lazy_static::lazy_static;
use sxd_document::{Package, parser};
use sxd_xpath::{Context, Factory, Value, XPath};
use sxd_xpath::nodeset::Node;
use regex::Regex;
use crate::item::Item;

pub struct MarginNotesExtractor {
    xpath_context: Context<'static>,

    root_items_path: XPath,
    title_path: XPath,
    margin_note_url_path: XPath,
    children_path: XPath,

    image_id_path_1: XPath,
    image_id_path_2: XPath,
    comments_path: XPath,

    arena: Arena<Item>
}

pub struct ExtractedNotes {
    pub items_arena: Arena<Item>,
    pub roots: Vec<NodeId>,
}

lazy_static! {
    static ref TITLE_CLEANUP_REGEX: Regex = Regex::new("\\s+").unwrap();
    static ref MARGIN_NOTE_ID_REGEX: Regex = Regex::new("marginnote3app://note/([A-F0-9-]+)").unwrap();
}

pub fn parse_xml(xml: &str) -> Package {
    lazy_static! {
        static ref DOCTYPE_REGEX: Regex = Regex::new(r#"<!DOCTYPE outline PUBLIC "-//omnigroup.com//DTD OUTLINE 3.0//EN" "http://www.omnigroup.com/namespace/OmniOutliner/xmloutline-v3.dtd">"#).unwrap();
    }

    let xml = DOCTYPE_REGEX.replace_all(xml, "");
    parser::parse(xml.as_ref()).expect("Failed to parse XML")
}

impl MarginNotesExtractor {
    fn create_context() -> Context<'static> {
        let mut ctx = sxd_xpath::context::Context::new();
        ctx.set_namespace("o", "http://www.omnigroup.com/namespace/OmniOutliner/v3");
        ctx
    }

    pub fn new() -> MarginNotesExtractor {
        let xpath_factory = Factory::new();

        let root_items_path = xpath_factory.build("/o:outline/o:root/o:item").unwrap().unwrap();
        let title_path = xpath_factory.build("string(./o:values/o:text[1]/o:p/o:run/o:lit/text())").unwrap().unwrap();
        let margin_note_url_path = xpath_factory.build("./o:values/o:text[4]/o:p[2]/o:run/o:lit/o:cell/@href").unwrap().unwrap();

        let image_id_path_1 = xpath_factory.build("./o:values//o:cell/@refid").unwrap().unwrap();
        let image_id_path_2 = xpath_factory.build("./o:note//o:cell/@refid").unwrap().unwrap();

        let comments_path = xpath_factory.build("./o:note//o:p//o:lit/text()").unwrap().unwrap();

        let children_path = xpath_factory.build("o:children/o:item").unwrap().unwrap();

        MarginNotesExtractor {
            xpath_context: MarginNotesExtractor::create_context(),

            root_items_path,
            title_path,
            margin_note_url_path,
            image_id_path_1,
            image_id_path_2,
            children_path,
            comments_path,

            arena: Arena::new()
        }
    }

    pub fn read_items(mut self, package: Package) -> ExtractedNotes {
        ExtractedNotes {
            roots: self.root_items(package),
            items_arena: self.arena,
        }
    }

    pub fn root_items(&mut self, package: Package) -> Vec<NodeId> {
        let document = package.as_document();

        match self.root_items_path.evaluate(&self.xpath_context, document.root()).unwrap() {
            Value::Nodeset(ns) => ns.iter().map(|node| self.create_item(node)).collect(),
            _ => panic!("Unable to read root items")
        }
    }

    fn create_item(&mut self, node: Node) -> NodeId {
        let (margin_note_url, margin_note_id) = self.extract_margin_note_url_and_id(node);
        let (given_title, title) = self.extract_title(node, &margin_note_id);
        let image_ids: Vec<String> = self.extract_image_ids(node);

        let comments: Option<String> = self.extract_comments(node);

        let item = Item {
            given_title,
            title,
            margin_note_id,
            image_ids,
            margin_note_url,
            comments,
        };

        let node_id = self.arena.new_node(item);

        match self.children_path.evaluate(&self.xpath_context, node).unwrap() {
            Value::Nodeset(ns) => {
                for node in ns.document_order() {
                    let child_id = self.create_item(node);
                    node_id.append(child_id, &mut self.arena)
                }
            },

            v => panic!("Unexpect XML when extracting children, got {:#?}", v)
        };

        node_id
    }

    fn extract_margin_note_url_and_id(&self, item: Node) -> (Option<String>, Option<String>) {
        let url = match self.margin_note_url_path.evaluate(&self.xpath_context, item).unwrap() {
            Value::Nodeset(mnu) => mnu.document_order_first().map(|n| n.string_value()),
            v => panic!("Unexpect XML when extracting url, got {:#?}", v)
        };

        let id = url.clone().map(|url| url.clone().chars().skip(22).collect());
        (url, id)
    }

    fn extract_comments(&self, item: Node) -> Option<String> {
        match self.comments_path.evaluate(&self.xpath_context, item).unwrap() {
            Value::Nodeset(ns) if ns.size() > 0 => Some(ns.document_order().iter().map(|node| node.string_value()).collect::<Vec<String>>().join("")),
            _ => None
        }
    }

    fn extract_image_ids(&self, item: Node) -> Vec<String> {
        let mut image_ids: Vec<String> = match self.image_id_path_1.evaluate(&self.xpath_context, item).unwrap() {
            Value::Nodeset(mnu) => mnu.document_order().iter().map(|n| n.string_value()).collect(),
            v => panic!("Unexpect XML when extracting images, got {:#?}", v)
        };

        let mut image_id_additional: Vec<String> = match self.image_id_path_2.evaluate(&self.xpath_context, item).unwrap() {
            Value::Nodeset(mnu) => mnu.document_order().iter().map(|n| n.string_value()).collect(),
            v => panic!("Unexpect XML when extracting images, got {:#?}", v)
        };

        image_ids.append(&mut image_id_additional);
        image_ids
    }

    fn extract_title(&self, item: Node, margin_note_id: &Option<String>) -> (Option<String>, String) {
        match self.title_path.evaluate(&self.xpath_context, item).unwrap() {
            Value::String(given_title) => {
                let given_title = given_title.trim();
                let given_title = TITLE_CLEANUP_REGEX.replace_all(given_title, " ");
                let given_title = given_title.trim();

                if !given_title.is_empty() {
                    let string = given_title.to_string();
                    (Some(string.clone()), string)
                } else if let Some(id) = margin_note_id {
                    eprintln!("Warning using margin note id as title for {}", id.clone());
                    (None, id.clone())
                } else {
                    panic!("Cannot determine title for {:#?}", item)
                }
            }

            v => panic!("Unexpect XML when extracting title, got {:#?}", v)
        }
    }
}
