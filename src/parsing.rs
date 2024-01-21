use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::blocking::Client;

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
                size *= 1048576.0;
            } else if unit.eq_ignore_ascii_case("gb") {
                size *= 1073741824.0;
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

pub fn get_page_from_path(path: &str) -> String {
    static CLIENT: Lazy<Client> = Lazy::new(|| {
        reqwest::blocking::ClientBuilder::new()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/117.0")
            .build()
            .unwrap()
    });
    CLIENT.get(path).send().unwrap().text().unwrap()
}
