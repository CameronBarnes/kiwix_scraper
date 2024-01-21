use std::{collections::HashMap, ops::Add, process::Command};

use once_cell::sync::Lazy;
use parsing::{get_page_from_path, parse_page};
use types::{Category, Document, LibraryItem};

mod parsing;
mod types;

static IS_WINDOWS: bool = cfg!(windows);
static HAS_RSYNC: Lazy<bool> = Lazy::new(check_for_rsync);

pub fn check_for_rsync() -> bool {
    Command::new("which")
        .arg("rsync")
        .output()
        .unwrap()
        .status
        .success()
}

fn to_document(data: (String, u64, String, String)) -> LibraryItem {
    let (_catname, size, name, url) = data;
    LibraryItem::Document(Document::new(name, url, size, types::DownloadType::Http))
}

fn main() {
    let mut map: HashMap<String, Vec<(String, u64, String, String)>> = HashMap::new();

    let items = parse_page(&get_page_from_path("https://wiki.kiwix.org/wiki/Content"));

    items.into_iter().for_each(|entry| {
        let name = entry.0.clone();
        map.entry(name)
            .and_modify(|e| e.push(entry.clone()))
            .or_insert(vec![entry]);
    });

    let mut root_kiwix = Category::new("Wiki".into(), vec![], false);
    let mut wikipedia = Category::new("Wikipedia".into(), vec![], false);
    let mut stack_exchange = Category::new("Stack Exchange".into(), vec![], false);
    let mut avanti = Category::new("Avanti".into(), vec![], false);
    let mut root_linux = Category::new("Linux".into(), vec![], false);

    for (key, value) in map.into_iter() {
        match key.as_str() {
            "wikipedia" | "wiktionary" | "wikivoyage" | "wikiversity" | "wikibooks"
            | "wikisource" | "wikiquote" | "wikinews" | "wikispecies" => {
                let cat = Category::new(key, value.into_iter().map(to_document).collect(), true);
                wikipedia.add(LibraryItem::Category(cat));
            }
            "ted" | "keylearning" | "scienceinthebath" => {
                let cat = Category::new(key, value.into_iter().map(to_document).collect(), false);
                root_kiwix.add(LibraryItem::Category(cat));
            }
            "installgentoo" | "gentoo" => {
                let cat = Category::new(
                    key.add(" (wiki)"),
                    value.into_iter().map(to_document).collect(),
                    true,
                );
                let gentoo =
                    Category::new("Gentoo".into(), vec![LibraryItem::Category(cat)], false);
                root_linux.add(LibraryItem::Category(gentoo));
            }
            "archlinux" => {
                let cat = Category::new(
                    key.add(" (wiki)"),
                    value.into_iter().map(to_document).collect(),
                    true,
                );
                let arch = Category::new("Arch".into(), vec![LibraryItem::Category(cat)], false);
                root_linux.add(LibraryItem::Category(arch));
            }
            _ => {
                if value.len() == 1 {
                    let (cat_name, size, _name, url) = &value[0];
                    let doc = Document::new(key, url.to_owned(), *size, types::DownloadType::Http);
                    let doc = LibraryItem::Document(doc);
                    if cat_name.ends_with("stackexchange.com") {
                        stack_exchange.add(doc);
                    } else if cat_name.starts_with("avanti") {
                        avanti.add(doc);
                    } else if cat_name.eq_ignore_ascii_case("teded")
                        || cat_name.eq_ignore_ascii_case("tedmed")
                    {
                        let cat = Category::new("ted".into(), vec![doc], false);
                        root_kiwix.add(LibraryItem::Category(cat));
                    } else {
                        root_kiwix.add(doc);
                    }
                } else {
                    let cat = Category::new(
                        key.clone(),
                        value.into_iter().map(to_document).collect(),
                        true,
                    );
                    let cat = LibraryItem::Category(cat);
                    if key.ends_with("stackexchange.com") {
                        stack_exchange.add(cat);
                    } else if key.starts_with("avanti") {
                        avanti.add(cat);
                    } else if key.eq_ignore_ascii_case("teded")
                        || key.eq_ignore_ascii_case("tedmed")
                    {
                        let cat = Category::new("ted".into(), vec![cat], false);
                        root_kiwix.add(LibraryItem::Category(cat));
                    } else {
                        root_kiwix.add(cat);
                    }
                }
            }
        }
    }

    root_kiwix.add(LibraryItem::Category(wikipedia));
    root_kiwix.add(LibraryItem::Category(stack_exchange));
    root_kiwix.add(LibraryItem::Category(avanti));

    let root_kiwix = LibraryItem::Category(root_kiwix);
    let root_linux = LibraryItem::Category(root_linux);

    println!(
        "{}\n{}",
        serde_json::to_string(&root_kiwix).unwrap(),
        serde_json::to_string(&root_linux).unwrap()
    );
}
