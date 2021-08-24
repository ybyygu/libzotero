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

// [[file:../zotero.note::*tags][tags:1]]
use sqlx::prelude::*;
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;

type Map = HashMap<String, String>;

// For any key-value record
#[derive(sqlx::FromRow, Debug)]
struct KvRec {
    key: String,
    value: String,
}

#[derive(sqlx::FromRow, Debug, Default)]
pub struct Rec {
    key: String,
    value: String,
    date: String,
    title: String,
}

impl std::fmt::Display for Rec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let title = format!("{:^50}", self.title);
        let title = get_aligned_string(&title[..50], 100);
        write!(f, "{} => {} | {:^} | {}", self.key, self.date, &title, self.value,)
    }
}

impl std::str::FromStr for Rec {
    type Err = gut::prelude::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("=>") {
            match s.splitn(2, "=>").collect_vec().as_slice() {
                [a, b] => {
                    let x = Self {
                        key: a.trim().into(),
                        value: b.trim().into(),
                        ..Default::default()
                    };
                    return Ok(x);
                }
                _ => bail!("invalid {}", s),
            }
        } else {
            bail!("invalid record: {}", s);
        }
    }
}

impl Rec {
    /// Return a link in zotero protocol
    pub fn item_link(&self) -> String {
        format!("zotero://select/items/1_{}", self.key)
    }
}

#[tokio::main(flavor = "current_thread")]
/// Create a new `report` item in zotero with a .note (org-mode) attachment, and
/// returns zotero uri of the new item.
pub async fn get_items_by_tag(tag: &str) -> Result<Vec<String>> {
    let dbfile = "/home/ybyygu/Data/zotero/zotero.sqlite.bak";
    let pool = SqlitePool::connect(dbfile).await?;

    // get matched items
    let recs = sqlx::query_as::<_, KvRec>(
        r#"
SELECT key, value FROM items
       JOIN itemData USING (itemID)
       JOIN itemDataValues USING (valueID)
       JOIN fields USING (fieldID)
       JOIN itemTags USING (itemID)
       JOIN tags USING (tagID)
       WHERE name = "todo" AND fieldName = "extra"
      "#,
    )
    .fetch_all(&pool)
    .await?;

    // get other fields, such as title and date
    let mut all = vec![];
    for x in recs {
        let d = get_item_data(&pool, &x.key).await?;
        let rec = Rec {
            key: x.key.into(),
            value: x.value.into(),
            title: d["title"].to_string(),
            date: d.get("date").unwrap_or(&"0000".to_string())[..4].to_string(),
            ..Default::default()
        };
        all.push(rec.to_string());
    }
    Ok(all)
}

/// search item fields with key
async fn get_item_data(pool: &SqlitePool, key: &str) -> Result<Map> {
    let sql = r#"
SELECT fields.FieldName as key, itemDataValues.value as value
       FROM itemData
       LEFT JOIN items ON itemData.itemID = items.itemID
       LEFT JOIN fields ON itemData.fieldID = fields.fieldID
       LEFT JOIN itemDataValues ON itemData.valueID = itemDataValues.valueID
       WHERE items.key = ? 
         AND fields.fieldName IN ("extra", "date", "publicationTitle", "title")
"#;

    let recs = sqlx::query_as::<_, KvRec>(sql).bind(key).fetch_all(pool).await?;
    let d = recs.into_iter().map(|x| (x.key, x.value)).collect();
    Ok(d)
}
// tags:1 ends here

// [[file:../zotero.note::*attachment][attachment:1]]
impl Rec {
    #[tokio::main(flavor = "current_thread")]
    /// Return full paths of zotero item attachments
    pub async fn attachment_paths(&self) -> Vec<String> {
        match get_attachment_paths_from_key(&self.key).await {
            Ok(paths) => paths,
            Err(err) => {
                eprintln!("no attachment found for key: {}", self.key);
                vec![]
            }
        }
    }
}

/// Return .pdf/.note attachements associated with the item in `key`
pub async fn get_attachment_paths_from_key(key: &str) -> Result<Vec<String>> {
    let dbfile = "/home/ybyygu/Data/zotero/zotero.sqlite.bak";
    let pool = SqlitePool::connect(dbfile).await?;

    // get matched items
    let recs = sqlx::query(
        r#"
SELECT itemAttachments.itemID as id, itemAttachments.path as path
FROM items, itemAttachments
WHERE itemAttachments.path is not null
  AND itemAttachments.parentItemID = items.itemID
  AND (itemAttachments.contentType = "application/pdf" OR itemAttachments.contentType = "application/x-note")
  AND items.key = ?
"#,
    )
    .bind(key)
    .fetch_all(&pool)
    .await?;

    let mut all = vec![];
    for rec in recs {
        let p: String = rec.try_get("path")?;
        let i: i64 = rec.try_get("id")?;
        let k = get_item_key_from_item_id(i).await?;
        all.push(full_attachment_path(&k, &p));
    }
    Ok(all)
}

// zotero's attachment path may have a "storage:" prefix
fn full_attachment_path(key: &str, path: &str) -> String {
    // FIXME: auto detect from zotero config
    // FIXME: dirty hack
    let zotero_storage_root = "/home/ybyygu/Data/zotero/storage";
    let attach_path = if path.starts_with("storage:") { &path[8..] } else { path };
    format!("{}/{}/{}", zotero_storage_root, key, attach_path)
}

/// Return items.key from itemID
async fn get_item_key_from_item_id(id: i64) -> Result<String> {
    let dbfile = "/home/ybyygu/Data/zotero/zotero.sqlite.bak";
    let pool = SqlitePool::connect(dbfile).await?;

    let rec = sqlx::query(
        r#"
SELECT key
FROM items
WHERE itemID = ?
"#,
    )
    .bind(id)
    .fetch_one(&pool)
    .await?;

    let x: String = rec.try_get("key")?;
    Ok(x)
}
// attachment:1 ends here

// [[file:../zotero.note::*alignment str][alignment str:1]]
fn get_aligned_string(s: &str, max_width: usize) -> String {
    use unicode_width::*;

    // replace special unicode chars for nice alignment
    let s = s.replace("–", "-").replace("×", "x").replace("−", "-");

    let width = s.width_cjk();
    assert!(max_width > width, "invalid {}/{}", max_width, width);
    // dbg!(s.len(), width, s.width());
    if s.len() == width {
        format!("{:width$}", s, width = max_width)
    } else {
        format!("{:width$}", s, width = max_width - s.len() + width)
    }
}
// alignment str:1 ends here

// [[file:../zotero.note::*test][test:1]]
#[tokio::test]
async fn test_diesel() -> Result<()> {
    let url = "/home/ybyygu/Data/zotero/zotero.sqlite.bak";
    let zotero = ZoteroDb::connect(url).unwrap();

    zotero.get_attachment_paths_from_key("NIUYMGLJ").unwrap();
    let x = zotero.get_attachment_from_link("zotero://select/items/1_RXBNJTNY");
    dbg!(x);

    let x = zotero.get_related_items();
    dbg!(x);

    let dbfile = "/home/ybyygu/Data/zotero/zotero.sqlite.bak";
    let pool = SqlitePool::connect(dbfile).await?;

    let s1 = "Global Optimization of Adsorbate–Surface Structu";
    let s2 = "Minima hopping guided path search: An efficient me";
    let s3 = "中 hopping guided path search: An efficient me";
    let s4 = "Structure of the SnO2(110)−(4×1) Surface ";
    println!("{} | xx", get_aligned_string(s1, 100));
    println!("{} | xx", get_aligned_string(s2, 100));
    println!("{} | xx", get_aligned_string(s3, 100));
    println!("{} | xx", get_aligned_string(s4, 100));

    let x = get_attachment_paths_from_key("AMC4WS9I").await?;
    dbg!(x);

    Ok(())
}
// test:1 ends here
