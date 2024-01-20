use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum LibraryItem {
    Document(Document),
    Category(Category),
}

impl LibraryItem {
    pub fn set_enabled(&mut self, enabled: bool) -> bool {
        match self {
            LibraryItem::Document(doc) => {
                if doc.can_download() {
                    doc.enabled = enabled;
                } else {
                    doc.enabled = false;
                }
                doc.enabled
            },
            LibraryItem::Category(cat) => {
                if cat.can_download() {
                    cat.enabled = enabled;
                } else {
                    cat.enabled = false;
                }
                cat.enabled
            },
        }
    }

    pub fn can_download(&self) -> bool {
        match self {
            LibraryItem::Document(doc) => doc.can_download(),
            LibraryItem::Category(cat) => cat.can_download(),
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
        Document{name, url, size, download_type: d_type, enabled}
    }

    /// In cases such as a rsync Document on a windows system we cant download it
    pub fn can_download(&self) -> bool {
        self.download_type != DownloadType::Rsync || !crate::IS_WINDOWS
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
        if single_selection { // Only one option can be enabled at a time with single selection
            (1..items.len()).for_each(|i| {
                items[i].set_enabled(false);
            });
        }
        let enabled = items.iter().any(LibraryItem::can_download);
        Category{name, items, enabled, single_selection}
    }

    pub fn can_download(&self) -> bool {
        self.items.iter().any(|item| item.can_download())
    }

}
