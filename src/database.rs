// [[file:../zotero.note::*base][base:1]]
use gut::prelude::*;

use diesel::prelude::*;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::models::*;
use crate::*;

#[derive(Clone)]
pub struct ZoteroDb {
    database_url: String,
    connection: Arc<Mutex<SqliteConnection>>,
}

impl ZoteroDb {
    /// Eastablish connection to database specified using env var
    /// `ZOTERO_DATABASE_URL`.
    pub fn establish() -> Result<ZoteroDb> {
        let db_var = "ZOTERO_DATABASE_URL";

        // read vars from .env file
        dotenv::dotenv().ok();
        let database_url =
            std::env::var(db_var).with_context(|| format!("{} var not set", db_var))?;
        debug!("DATABASE URL: {}", database_url);

        Self::connect(&database_url)
    }

    /// Connect to database specified using `database_url`.
    pub fn connect(database_url: &str) -> Result<ZoteroDb> {
        // diesel accept &str, not Path
        let conn = SqliteConnection::establish(database_url)?;
        let conn = Arc::new(Mutex::new(conn));

        let db = ZoteroDb {
            database_url: database_url.into(),
            connection: conn.clone(),
        };

        Ok(db)
    }

    /// Show database url.
    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    pub(crate) fn get(&self) -> MutexGuard<'_, SqliteConnection> {
        self.connection.lock().expect("cannot lock db connection!")
    }
}
// base:1 ends here

// [[file:../zotero.note::*core][core:1]]
use crate::models::*;

impl ZoteroDb {
    pub fn get_attachment_paths_from_key(&self, k: &str) -> Result<Vec<(String, String)>> {
        let con = self.get();

        let parent_item: Item = {
            use crate::schema::items::dsl::*;
            items
                .filter(key.eq(k))
                .first(&*con)
                .context("find parent itemID")?
        };

        let attachment_items: Vec<(i32, Option<String>)> = {
            use crate::schema::itemAttachments::dsl::*;
            itemAttachments
                .select((itemID, path))
                .filter(parentItemID.eq(parent_item.id))
                .filter(path.is_not_null())
                .filter(
                    contentType
                        .eq("application/pdf")
                        .or(contentType.eq("application/x-note")),
                )
                .load(&*con)
                .context("find attachment itemID")?
        };

        let attachments = {
            use crate::schema::items::dsl::*;
            attachment_items
                .into_iter()
                .map(|(item_id, path)| {
                    let parent_key: String = items
                        .find(item_id)
                        .select(key)
                        .first(&*con)
                        .expect("attachment item key");
                    (parent_key, path.unwrap())
                })
                .collect()
        };

        Ok(dbg!(attachments))
    }

    pub fn get_attachment_from_link(&self, link: &str) -> Result<Option<String>> {
        // FIXME: auto detect from zotero config
        use std::path::PathBuf;

        // FIXME: dirty hack
        let zotero_storage_root = "/home/ybyygu/Data/zotero/storage";
        let p0 = "zotero://select/items/0_";
        let p1 = "zotero://select/items/1_";
        if link.starts_with(p1) || link.starts_with(p0) {
            let key = &link[p1.len()..];
            let attachments = self.get_attachment_paths_from_key(key)?;
            if attachments.len() > 0 {
                let (key, path) = &attachments[0];
                let attach_path = if path.starts_with("storage:") {
                    &path[8..]
                } else {
                    path
                };
                let path = format!("{}/{}/{}", zotero_storage_root, key, attach_path);
                return Ok(Some(path));
            } else {
                return Ok(None);
            }
        }
        Ok(None)
    }
}
// core:1 ends here

// [[file:../zotero.note::*relation][relation:1]]
impl ZoteroDb {
    fn get_related_items(&self) -> Result<Vec<(String, String)>> {
        let con = self.get();

        // key: "2X4DGF8X",
        // object: "http://zotero.org/users/15074/items/9F6B5E9G",
        let x: Vec<(String, String)> = {
            use crate::schema::itemRelations::dsl::*;
            use crate::schema::items::dsl::*;
            itemRelations
                .inner_join(items)
                .select((key, object))
                .filter(predicateID.eq(2))
                .filter(itemTypeID.ne(14))
                .limit(10)
                .load(&*con)
                .context("find item relations")?
        };

        Ok(x)
    }
}

// key  object
// | 2787B283 | http://zotero.org/users/15074/items/M2S2HTNN |
fn parse_zotero_key_from_object_url(url: &str) -> Option<String> {
    if url.starts_with("http://zotero.org") {
        url.rsplit("items/").next().map(|x| x.to_string())
    } else {
        None
    }
}

#[test]
fn test_key_from_object_url() {
    let url = "http://zotero.org/users/15074/items/M2S2HTNN";
    let parsed_key = parse_zotero_key_from_object_url(url);
    assert_eq!(parsed_key, Some("M2S2HTNN".to_string()))
}
// relation:1 ends here

// [[file:../zotero.note::*test][test:1]]
#[test]
fn test_diesel() {
    let url = "/home/ybyygu/Data/zotero/zotero.sqlite.bak";
    let zotero = ZoteroDb::connect(url).unwrap();

    zotero.get_attachment_paths_from_key("NIUYMGLJ").unwrap();
    let x = zotero.get_attachment_from_link("zotero://select/items/1_RXBNJTNY");
    dbg!(x);

    let x = zotero.get_related_items();
    dbg!(x);
}
// test:1 ends here
