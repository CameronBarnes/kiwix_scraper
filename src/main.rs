use std::process::Command;

use once_cell::sync::Lazy;
use parsing::{get_page_from_path, parse_items_into_categories, parse_page};

mod parsing;
mod types;

static IS_WINDOWS: bool = cfg!(windows);
static HAS_RSYNC: Lazy<bool> = Lazy::new(check_for_rsync);

#[must_use]
pub fn check_for_rsync() -> bool {
    let result = Command::new("which").arg("rsync").output();

    if let Ok(output) = result {
        output.status.success()
    } else {
        false
    }
}

fn main() {
    let items = parse_page(&get_page_from_path("https://wiki.kiwix.org/wiki/Content"));

    let categories = parse_items_into_categories(items);

    for cat in categories {
        println!("{}", serde_json::to_string(&cat).unwrap());
    }
}
