use std::{collections::HashMap, ops::Add};

use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::blocking::Client;

use crate::types::{Category, Document, DownloadType, LibraryItem};

pub fn parse_page(input: &str) -> Vec<(String, u64, String, String)> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new("<td>(.+?)</td>\n<td>en</td>\n<td>(.+?)</td>\n<td>.+?</td>\n<td>(.*?)</td>\n<td><a rel=\"nofollow\".+?href=\"(.+?)\">. Download</a>").unwrap()
    });

    RE.captures_iter(input)
        .map(|e| e.extract())
        .map(|(_, [category, size, doc_name, url])| {
            let category = category.strip_suffix(" (English)").unwrap();

            let (size, unit) = size.split_once(' ').unwrap();
            let mut size: f64 = size.parse().unwrap();
            if unit.eq_ignore_ascii_case("kb") {
                size *= 1024.0;
            } else if unit.eq_ignore_ascii_case("mb") {
                size *= 1_048_576.0;
            } else if unit.eq_ignore_ascii_case("gb") {
                size *= 1_073_741_824.0;
            }
            let size = size as u64;
            (
                category.to_string(),
                size,
                doc_name.to_string(),
                url.to_string(),
            )
        })
        .collect()
}

fn to_document(data: (String, u64, String, String)) -> LibraryItem {
    let (_catname, size, name, url) = data;
    LibraryItem::Document(Document::new(name, url, size, DownloadType::Http))
}

pub fn parse_items_into_categories(items: Vec<(String, u64, String, String)>) -> Vec<LibraryItem> {
    let mut map: HashMap<String, Vec<(String, u64, String, String)>> = HashMap::new();

    for entry in items {
        let name = entry.0.clone();
        map.entry(name)
            .and_modify(|e| e.push(entry.clone()))
            .or_insert_with(|| vec![entry]);
    }

    let mut root_kiwix = Category::new("Wiki".into(), vec![], false);
    let mut wikipedia = Category::new("Wikipedia".into(), vec![], false);
    let mut stack_exchange = Category::new("Stack Exchange".into(), vec![], false);
    let mut avanti = Category::new("Avanti".into(), vec![], false);
    let mut root_linux = Category::new("Linux".into(), vec![], false);

    for (key, value) in map {
        match key.as_str() {
            "wikipedia" | "wiktionary" | "wikivoyage" | "wikiversity" | "wikibooks"
            | "wikisource" | "wikiquote" | "wikinews" | "wikispecies" => {
                let cat = Category::new(key, value.into_iter().map(to_document).collect(), true);
                wikipedia.add(LibraryItem::Category(cat));
            }
            "ted" | "keylearning" | "scienceinthebath" | "aimhi" => {
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
            "alpinelinux" => {
                let cat = Category::new(
                    key.add(" (wiki)"),
                    value.into_iter().map(to_document).collect(),
                    true,
                );
                let arch = Category::new("Alpine".into(), vec![LibraryItem::Category(cat)], false);
                root_linux.add(LibraryItem::Category(arch));
            }
            _ => {
                if value.len() == 1 {
                    let (cat_name, size, _name, url) = &value[0];
                    let doc = Document::new(key, url.to_owned(), *size, DownloadType::Http);
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

    vec![root_kiwix, root_linux]
}

pub fn get_page_from_path(path: &str) -> String {
    static CLIENT: Lazy<Client> = Lazy::new(|| {
        reqwest::blocking::ClientBuilder::new()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/117.0")
            .build()
            .unwrap()
    });
    CLIENT.get(path).send().unwrap().text().unwrap()
}
