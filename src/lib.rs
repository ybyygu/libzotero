// [[file:~/Workspace/Programming/zotero/zotero.note::*imports][imports:1]]
#[macro_use]
extern crate diesel;
// imports:1 ends here

// [[file:~/Workspace/Programming/zotero/zotero.note::*mods][mods:1]]
pub mod models;
pub mod schema;

mod database;
mod zotxt;
// mods:1 ends here

// [[file:~/Workspace/Programming/zotero/zotero.note::*pub][pub:1]]
use gut::prelude::*;

/// Return PDF attachment path from zotero protocol link
///
/// # Parameters
/// ----------
/// * link: zotero item selection link, e.g: zotero://select/items/1_NIUYMGLJ
pub fn get_attachment_from_link(link: &str) -> Result<Option<String>> {
    use crate::database::*;
    use crate::zotxt::*;

    let url = "/home/ybyygu/Data/zotero/zotero.sqlite.bak";
    let zot_server = ZoteroServer::default();
    let zot_db = ZoteroDb::connect(url)?;

    zot_server.get_attachment(link).or(zot_db.get_attachment_from_link(link))
}
// pub:1 ends here

// [[file:~/Workspace/Programming/zotero/zotero.note::*test][test:1]]
#[test]
fn test_get_attachment() {
    let link = "zotero://select/items/1_U5MRLMBI";
    let attachment = get_attachment_from_link(link).expect("zotero attach");
    assert!(attachment.is_some());
    let path = std::path::PathBuf::from(attachment.unwrap());
    assert!(path.exists());
}
// test:1 ends here
