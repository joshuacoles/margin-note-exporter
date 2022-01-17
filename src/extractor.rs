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
        }
    }

    fn create_context() -> Context<'static> {
        let mut ctx = sxd_xpath::context::Context::new();
        ctx.set_namespace("o", "http://www.omnigroup.com/namespace/OmniOutliner/v3");
        ctx
    }

    pub fn root_items(&self, package: Package) -> Vec<Item> {
        let document = package.as_document();

        match self.root_items_path.evaluate(&self.xpath_context, document.root()).unwrap() {
            Value::Nodeset(ns) => ns.iter().map(|node| self.create_item(node)).collect(),
            _ => panic!("Unable to read root items")
        }
    }

    fn create_item(&self, item: Node) -> Item {
        let margin_note_url = match self.margin_note_url_path.evaluate(&self.xpath_context, item).unwrap() {
            Value::Nodeset(mnu) => mnu.document_order_first().map(|n| n.string_value()),
            v => panic!("Unexpect XML when extracting url, got {:#?}", v)
        };

        let margin_note_id = margin_note_url.clone().map(|url| url.chars().skip(22).collect());

        let title = self.title_of(item, &margin_note_id);

        let mut image_ids: Vec<String> = match self.image_id_path_1.evaluate(&self.xpath_context, item).unwrap() {
            Value::Nodeset(mnu) => mnu.document_order().iter().map(|n| n.string_value()).collect(),
            v => panic!("Unexpect XML when extracting images, got {:#?}", v)
        };

        let mut image_id_additional: Vec<String> = match self.image_id_path_2.evaluate(&self.xpath_context, item).unwrap() {
            Value::Nodeset(mnu) => mnu.document_order().iter().map(|n| n.string_value()).collect(),
            v => panic!("Unexpect XML when extracting images, got {:#?}", v)
        };

        image_ids.append(&mut image_id_additional);

        let children = match self.children_path.evaluate(&self.xpath_context, item).unwrap() {
            Value::Nodeset(ns) => ns.document_order().iter().map(|node| self.create_item(*node)).collect(),
            v => panic!("Unexpect XML when extracting children, got {:#?}", v)
        };

        let comments: Option<String> = match self.comments_path.evaluate(&self.xpath_context, item).unwrap() {
            Value::Nodeset(ns) if ns.size() > 0 => Some(ns.document_order().iter().map(|node| node.string_value()).collect::<Vec<String>>().join("")),
            _ => None
        };

        Item {
            title,
            margin_note_id,
            image_ids,
            margin_note_url,
            comments,
            children,
        }
    }

    fn title_of(&self, item: Node, margin_note_id: &Option<String>) -> String {
        match self.title_path.evaluate(&self.xpath_context, item).unwrap() {
            Value::String(given_title) => {
                let given_title = given_title.trim();
                let given_title = TITLE_CLEANUP_REGEX.replace_all(given_title, " ");
                let given_title = given_title.trim();

                if !given_title.is_empty() {
                    given_title.to_string()
                } else if let Some(id) = margin_note_id {
                    eprintln!("Warning using margin note id as title for {}", id.clone());
                    id.clone()
                } else {
                    panic!("Cannot determine title for {:#?}", item)
                }
            }

            v => panic!("Unexpect XML when extracting title, got {:#?}", v)
        }
    }
}
