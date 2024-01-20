use std::{process::Command, collections::HashMap};

use once_cell::sync::Lazy;
use parsing::{parse_page, get_page_from_path};

mod types;
mod parsing;

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

fn main() {

    let mut map: HashMap<String, Vec<(String, u64, String, String)>> = HashMap::new();

    let items = parse_page(&get_page_from_path("https://wiki.kiwix.org/wiki/Content"));

    items.into_iter().for_each(|entry| {
        let name = entry.0.clone();
        map.entry(name).and_modify(|e| e.push(entry.clone())).or_insert(vec!(entry));
    });

    for category in map.iter() {
        println!("{}: {}", category.0, category.1.len());
    }
    
}
