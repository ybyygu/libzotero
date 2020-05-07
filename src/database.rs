// [[file:~/Workspace/Programming/zotero/zotero.note::*base][base:1]]
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

// [[file:~/Workspace/Programming/zotero/zotero.note::*core][core:1]]
impl ZoteroDb {
    pub fn get_attachment_paths_from_key(&self, k: &str) -> Result<Vec<Option<String>>> {
        use crate::schema::*;

        let con = self.get();
        let results: Vec<Option<String>> = itemAttachments::table
            .filter(
                itemAttachments::columns::contentType
                    .eq("application/pdf")
                    .or(itemAttachments::columns::contentType.eq("application/x-note")),
            )
            .inner_join(
                items::table.on(itemAttachments::columns::parentItemID.eq(items::columns::itemID)),
            )
            .filter(items::columns::key.eq(k))
            .select(itemAttachments::columns::path)
            .limit(5)
            .load(&*con)
            .context("Error loading keys")?;

        // remove item `None`
        // let results: Vec<_> = results.into_iter().filter_map(|x| x).collect();
        println!("Found {} attachments for item {}", results.len(), k);

        Ok(dbg!(results))
    }

    pub fn get_attachment(&self, link: &str) -> Result<Option<String>> {
        // FIXME: auto detect from zotero config
        use std::path::PathBuf;

        let zotero_storage_root = "/home/ybyygu/Data/zotero/storage";
        let p = "zotero://select/items/1_";
        if link.starts_with(p) {
            let key = &link[p.len()..];
            let paths = self.get_attachment_paths_from_key(key)?;
            if paths.len() > 0 {
                let path = &paths[0].as_ref().unwrap();
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

#[test]
fn test_diesel() {
    // use crate::schema::itemAttachments::dsl::*;
    // use crate::schema::items::dsl::*;
    use crate::schema::*;

    let url = "/home/ybyygu/Data/zotero/zotero.sqlite.bak";
    let zotero = ZoteroDb::connect(url).unwrap();

    zotero.get_attachment_paths_from_key("NIUYMGLJ").unwrap();
    let x = zotero.get_attachment("zotero://select/items/1_RXBNJTNY");
    dbg!(x);
}
// core:1 ends here
