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

pub fn get_zotero_attachment(link: &str) -> Result<Option<String>> {
    let zot_server = crate::zotxt::ZoteroServer::default();

    let zot_db = crate::database::ZoteroDb::establish()?;
    // zot_server.get_attachment(link).or(zot_db.get_attachment(link))
    zot_server.get_attachment(link)
}
// pub:1 ends here
