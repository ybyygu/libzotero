// [[file:../zotero.note::*imports][imports:1]]
use gut::prelude::*;
use serde::*;

pub(crate) struct ZoteroServer {
    base_url: String,
}

impl Default for ZoteroServer {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:23119".into(),
        }
    }
}
// imports:1 ends here

// [[file:../zotero.note::*get attachment][get attachment:1]]
impl ZoteroServer {
    /// Get attachment of item `link` in zotero
    ///
    /// link: zotero://select/items/1_BHDGEJJP
    pub fn get_attachment(&self, link: &str) -> Result<Option<String>> {
        let p = "zotero://select/items/";
        if link.starts_with(p) {
            let key = &link[p.len()..];
            let url = format!("{}/zotxt/itesm?key={}&format=paths", self.base_url, key);
            let resp = zotxt_client_call(&url)?;

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

    /// Get attachment of current selected item in zotero
    pub fn get_attachment_of_selected_item(&self) -> Result<Option<String>> {
        let url = format!(
            "{}/zotxt/items?selected=selected&format=paths",
            self.base_url
        );
        let resp = zotxt_client_call(&url)?;

        if resp.len() == 1 {
            let path = resp[0].paths[0].clone();
            Ok(Some(path))
        } else {
            Ok(None)
        }
    }

    /// Get zotero url of current selected item in zotero
    pub fn get_uri_of_selected_item(&self) -> Result<Option<String>> {
        let url = format!("{}/zotxt/items?selected=selected&format=key", self.base_url);
        let resp = zotxt_client_call(&url)?;

        if resp.len() == 1 {
            let key = resp[0].key.clone();
            let uri = format!("zotero://select/items/{}", key);
            Ok(Some(uri))
        } else {
            Ok(None)
        }
    }
}

fn zotxt_client_call(url: &str) -> Result<Vec<ResponseItem>> {
    let x = reqwest::blocking::get(url)?.text()?;
    let items = serde_json::from_str(&x)?;
    Ok(items)
}

#[derive(Debug, Deserialize)]
struct ResponseItem {
    key: String,
    paths: Vec<String>,
}
// get attachment:1 ends here

// [[file:../zotero.note::*save item][save item:1]]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Creator {
    first_name: String,
    last_name: String,
}

impl Default for Creator {
    fn default() -> Self {
        Self {
            first_name: "Wenping".into(),
            last_name: "Guo".into(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Attachment {
    title: String,
    url: String,
    mime_type: String,
    snapshot: bool,
    proxy: bool,
}

impl Default for Attachment {
    fn default() -> Self {
        Self {
            title: "research note".into(),
            url: "http://localhost:8000/life.note".into(),
            mime_type: "application/x-note".into(),
            snapshot: true,
            proxy: false,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConnectorItem {
    item_type: String,
    title: String,
    extra: String,
    place: String,
    date: String,
    creators: Vec<Creator>,
    attachments: Vec<Attachment>,
}

impl Default for ConnectorItem {
    fn default() -> Self {
        Self {
            item_type: "report".into(),
            title: "research.note".into(),
            extra: "this is a test".into(),
            place: "Beijing".into(),
            // FIXME: using current date
            date: "2020/08/01".into(),
            creators: vec![Creator::default()],
            attachments: vec![Attachment::default()],
        }
    }
}

impl ZoteroServer {
    /// Create a new report item with an attached .note file
    ///
    /// Return the file path to the attached file
    pub fn create_new_note(&self, f: &str) -> Result<Option<String>> {
        let url = format!("{}/connector/saveItems", self.base_url);
        let mut call = std::collections::HashMap::new();
        let items = vec![ConnectorItem::default()];
        call.insert("items", items);
        // let json = serde_json::to_string_pretty(&call).unwrap();
        // println!("{}", json);
        let new = reqwest::blocking::Client::new()
            .post(&url)
            .json(&call)
            .send()?;

        let resp = new.text().context("client requests to create item")?;
        debug!("server response: {}", resp);

        self.get_attachment_of_selected_item()
    }
}

#[test]
fn test_connector_json() {
    let connector = ZoteroServer::default();
    let x = connector.create_new_note("xx").unwrap();
    dbg!(x);
}
// save item:1 ends here
