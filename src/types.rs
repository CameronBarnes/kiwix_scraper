use std::cmp::Reverse;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum LibraryItem {
    Document(Document),
    Category(Category),
}

impl LibraryItem {
    pub fn set_enabled(&mut self, enabled: bool) -> bool {
        match self {
            Self::Document(doc) => {
                if doc.can_download() {
                    doc.enabled = enabled;
                } else {
                    doc.enabled = false;
                }
                doc.enabled
            }
            Self::Category(cat) => {
                if cat.can_download() {
                    cat.enabled = enabled;
                } else {
                    cat.enabled = false;
                }
                cat.enabled
            }
        }
    }

    pub fn can_download(&self) -> bool {
        match self {
            Self::Document(doc) => doc.can_download(),
            Self::Category(cat) => cat.can_download(),
        }
    }

    pub fn size(&self, enabled_only: bool) -> u64 {
        match self {
            Self::Document(doc) => {
                if enabled_only {
                    if doc.enabled {
                        doc.size
                    } else {
                        0
                    }
                } else {
                    doc.size
                }
            }
            Self::Category(cat) => {
                if enabled_only {
                    cat.size(true)
                } else {
                    cat.size(false)
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum DownloadType {
    Http,
    Rsync,
    #[allow(unused)]
    Either,
}

#[derive(Debug, Serialize)]
pub struct Document {
    name: String,
    url: String,
    size: u64,
    download_type: DownloadType,
    pub enabled: bool,
}

impl Document {
    pub fn new(name: String, url: String, size: u64, d_type: DownloadType) -> Self {
        let enabled = d_type != DownloadType::Rsync || !crate::IS_WINDOWS;
        Self {
            name,
            url,
            size,
            download_type: d_type,
            enabled,
        }
    }

    /// In cases such as a rsync Document on a windows system we cant download it
    pub fn can_download(&self) -> bool {
        self.download_type != DownloadType::Rsync || (!crate::IS_WINDOWS && *crate::HAS_RSYNC)
    }
}

#[derive(Debug, Serialize)]
pub struct Category {
    name: String,
    pub items: Vec<LibraryItem>,
    single_selection: bool,
    pub enabled: bool,
}

impl Category {
    pub fn new(name: String, mut items: Vec<LibraryItem>, single_selection: bool) -> Self {
        items.sort_unstable_by_key(|item| {
            let size = match item {
                LibraryItem::Document(doc) => doc.size,
                LibraryItem::Category(cat) => cat.size(true),
            };
            Reverse(size)
        });
        if single_selection {
            // Only one option can be enabled at a time with single selection
            (1..items.len()).for_each(|i| {
                items[i].set_enabled(false);
            });
        }
        let enabled = items.iter().any(LibraryItem::can_download);
        Self {
            name,
            items,
            single_selection,
            enabled,
        }
    }

    pub fn can_download(&self) -> bool {
        self.items.iter().any(LibraryItem::can_download)
    }

    pub fn add(&mut self, item: LibraryItem) {
        match item {
            LibraryItem::Document(_) => self.items.push(item),
            LibraryItem::Category(category) => {
                if category.items.is_empty() {
                    return;
                }
                if let Some(merge) = self.items.iter_mut().find_map(|e| match e {
                    LibraryItem::Document(_) => None,
                    LibraryItem::Category(cat) => {
                        if cat.name.eq_ignore_ascii_case(&category.name) {
                            Some(cat)
                        } else {
                            None
                        }
                    }
                }) {
                    // End of condition, merge the two categories if their names match
                    for item in category.items {
                        merge.add(item);
                    }
                } else {
                    self.items.push(LibraryItem::Category(category));
                }
            }
        }
    }

    pub fn size(&self, enabled_only: bool) -> u64 {
        self.items.iter().map(|item| item.size(enabled_only)).sum()
    }
}
