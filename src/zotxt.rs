// [[file:~/Workspace/Programming/zotero/zotero.note::*zotxt.rs][zotxt.rs:1]]
use gut::prelude::*;
use serde::*;


pub(crate) struct ZoteroServer {
    base_url: String,
}

impl Default for ZoteroServer {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:23119/zotxt".into(),
        }
    }
}

impl ZoteroServer {
    /// Request server to list current jobs in queue.
    /// link: zotero://select/items/1_BHDGEJJP
    pub fn get_attachment(&self, link: &str) -> Result<Option<String>> {
        let p = "zotero://select/items/";
        if link.starts_with(p) {
            let key = &link[p.len()..];
            let url = format!("{}/items?key={}&format=paths", self.base_url, key);
            let x = reqwest::blocking::get(&url)?.text()?;
            let resp: Vec<ZotxtResponseItem> = serde_json::from_str(&x)?;

            let path = if resp.len() > 0 && resp[0].paths.len() > 0 {
                let path = resp[0].paths[0].clone();
                Some(path)
            } else {
                None
            };
            return Ok(path);
        }
        eprintln!("invalid link: {}", link);
        Ok(None)
    }
}

#[derive(Debug, Deserialize)]
struct ZotxtResponseItem {
    key: String,
    paths: Vec<String>,
}
// zotxt.rs:1 ends here
