// [[file:../zotero.note::61a34b09][61a34b09]]
#![allow(non_snake_case)]

// #[macro_use]
// extern crate diesel;
// 61a34b09 ends here

// [[file:../zotero.note::*mods][mods:1]]
// pub mod models;
// pub mod schema;
// mod database;

mod db;
mod profile;
mod server;
// mods:1 ends here

// [[file:../zotero.note::*pub][pub:1]]
use gut::prelude::*;

// /// Return PDF attachment path from zotero protocol link
// ///
// /// # Parameters
// /// ----------
// /// * link: zotero item selection link, e.g: zotero://select/items/1_NIUYMGLJ
// pub fn get_attachment_from_link(link: &str) -> Result<Option<String>> {
//     use crate::database::*;
//     use crate::server::*;

//     let url = "/home/ybyygu/Data/zotero/zotero.sqlite.bak";
//     let zot_server = ZoteroServer::default();
//     let zot_db = ZoteroDb::connect(url)?;

//     zot_server
//         .get_attachment(link)
//         .or(zot_db.get_attachment_from_link(link))
// }

/// Create a new `report` item in zotero with a .note (org-mode) attachment, and
/// returns zotero uri of the new item.
pub fn create_new_note() -> Result<Option<String>> {
    use crate::server::*;

    let connector = ZoteroServer::default();
    connector.create_new_note("xx")
}

pub use crate::db::{get_item_key_from_link, get_items_by_collection, get_items_by_tag, Item};
// pub:1 ends here

// [[file:../zotero.note::*test][test:1]]
#[test]
fn test_get_attachment() -> Result<()> {
    let link = "zotero://select/items/1_U5MRLMBI";
    let key = get_item_key_from_link(link)?;
    // let item = Item::new(key);
    let item: Item = key.parse()?;
    let attachments = item.attachment_paths();
    assert_eq!(attachments.len(), 1);
    let path = std::path::PathBuf::from(&attachments[0]);
    assert!(path.exists());
    Ok(())
}
// test:1 ends here
