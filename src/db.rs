// [[file:../zotero.note::*imports][imports:1]]
use gut::prelude::*;
use std::path::{Path, PathBuf};

use sqlx::prelude::*;
use sqlx::sqlite::SqlitePool;
// imports:1 ends here

// [[file:../zotero.note::*base][base:1]]
mod base {
    use super::*;

    pub struct ZoteroDb {
        // https://docs.rs/sqlx/0.5.7/sqlx/pool/struct.Pool.html#why-use-a-pool
        pool: SqlitePool,
    }

    impl ZoteroDb {
        pub async fn connect(uri: &str) -> Result<Self> {
            let pool = SqlitePool::connect(uri).await?;
            let db = Self { pool };
            Ok(db)
        }

        pub fn pool(&self) -> &SqlitePool {
            &self.pool
        }
    }
}
pub use base::ZoteroDb;
// base:1 ends here

// [[file:../zotero.note::*core][core:1]]
impl ZoteroDb {
    /// Return items.key from itemID when itemID is known
    async fn get_item_key(&self, id: i64) -> Result<String> {
        let rec = sqlx::query(
            r#"
SELECT key
FROM items
WHERE itemID = ?
"#,
        )
        .bind(id)
        .fetch_one(self.pool())
        .await?;

        Ok(rec.try_get("key")?)
    }
}
// core:1 ends here

// [[file:../zotero.note::*alignment str][alignment str:1]]
fn get_aligned_string(s: &str, max_width: usize) -> String {
    // replace special unicode chars for nice alignment
    let s = s.replace("–", "-").replace("×", "x").replace("−", "-");
    let title = format!("{:width$}", s, width = max_width);
    let s = title[..max_width].to_string();

    use unicode_width::*;
    let width = s.width_cjk();
    // dbg!(s.len(), width, s.width());
    if s.len() == width {
        format!("{:width$}", s, width = max_width)
    } else {
        format!("{:width$}", s, width = max_width - s.len() + width)
    }
}
// alignment str:1 ends here

// [[file:../zotero.note::*item][item:1]]
#[derive(sqlx::FromRow, Debug, Default)]
pub struct Item {
    key: String,
    extra: String,
    date: String,
    title: String,
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let title = get_aligned_string(&self.title, 100);
        write!(f, "{} => {} | {:^} | {}", self.key, self.date, &title, self.extra,)
    }
}

impl std::str::FromStr for Item {
    type Err = gut::prelude::Error;

    // only zotero item key matters
    // HUMF2AEA
    // HUMF2AEA =>
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() >= 8 {
            let key = s[..8].to_string();
            let x = Self {
                key,
                ..Default::default()
            };
            return Ok(x);
        } else {
            bail!("invalid record: {}", s);
        }
    }
}

impl Item {
    /// Construct a zotero item from item key.
    pub fn new<S: Into<String>>(key: S) -> Self {
        Self {
            key: key.into(),
            ..Default::default()
        }
    }

    /// Return a link in zotero protocol
    pub fn item_link(&self) -> String {
        format!("zotero://select/items/1_{}", self.key)
    }
}
// item:1 ends here

// [[file:../zotero.note::*link][link:1]]
impl ZoteroDb {
    /// Extract item key from link in zotero protocol
    fn get_item_key_from_link(link: &str) -> Result<String> {
        let p0 = "zotero://select/items/0_";
        let p1 = "zotero://select/items/1_";
        if link.starts_with(p1) || link.starts_with(p0) {
            let key = &link[p1.len()..];
            Ok(key.into())
        } else {
            bail!("invalid link: {}", link);
        }
    }
}
// link:1 ends here

// [[file:../zotero.note::*rec][rec:1]]
// For any key-value record
#[derive(sqlx::FromRow, Debug)]
struct KvRec {
    key: String,
    value: String,
}
// rec:1 ends here

// [[file:../zotero.note::*tags][tags:1]]
type Map = std::collections::HashMap<String, String>;

impl ZoteroDb {
    /// Create a new `report` item in zotero with a .note (org-mode) attachment, and
    /// returns zotero uri of the new item.
    async fn get_items_by_tag(&self, tag: &str) -> Result<Vec<Item>> {
        let items = sqlx::query_as::<_, KvRec>(
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
        .fetch_all(self.pool())
        .await?;

        // get other fields, such as title and date
        let mut all = vec![];
        for x in items {
            let d = get_item_data(self.pool(), &x.key).await?;
            let rec = Item {
                key: x.key.into(),
                extra: x.value.into(),
                title: d["title"].to_string(),
                date: d.get("date").unwrap_or(&"0000".to_string())[..4].to_string(),
                ..Default::default()
            };
            all.push(rec);
        }

        Ok(all)
    }
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

// [[file:../zotero.note::*relation][relation:1]]
impl ZoteroDb {
    // key: "2X4DGF8X",
    // value: "http://zotero.org/users/15074/items/9F6B5E9G",
    async fn get_related_items(&self, key: &str) -> Result<Vec<Item>> {
        let recs = sqlx::query_as::<_, KvRec>(
            r#"
SELECT items.key as key, itemRelations.object as value FROM items
JOIN itemRelations USING (itemID)
WHERE predicateID == 2 AND
      itemTypeID != 14 AND
      items.key = ?
"#,
        )
        .bind(key)
        .fetch_all(self.pool())
        .await?;

        let related_items = recs
            .into_iter()
            .filter_map(|rec| parse_zotero_key_from_object_url(&rec.value))
            .map(|key| Item::new(key))
            .collect_vec();
        Ok(related_items)
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

// [[file:../zotero.note::*attachment][attachment:1]]
#[derive(sqlx::FromRow, Debug)]
struct Attachment {
    id: i64,
    path: String,
}

impl ZoteroDb {
    /// Return the list of attachment for item in `key`
    async fn get_attachments(&self, key: &str) -> Result<Vec<Attachment>> {
        let attachments = sqlx::query_as::<_, Attachment>(
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
        .fetch_all(self.pool())
        .await?;

        Ok(attachments)
    }
}

/// Return .pdf/.note attachements associated with the item in `key`
async fn get_attachment_paths_from_key(key: &str) -> Result<Vec<String>> {
    let db = ZoteroDb::connect(Cached_Db_File).await?;

    let mut all = vec![];
    for attachment in db.get_attachments(key).await? {
        let p = attachment.path;
        let k = db.get_item_key(attachment.id).await?;
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
// attachment:1 ends here

// [[file:../zotero.note::*api][api:1]]
static Db_File: &str = "/home/ybyygu/Data/zotero/zotero.sqlite";
static Cached_Db_File: &str = "/home/ybyygu/.cache/zotero.sqlite";

/// Extract item key from link in zotero protocol
pub fn get_item_key_from_link(link: &str) -> Result<String> {
    ZoteroDb::get_item_key_from_link(link)
}

#[tokio::main(flavor = "current_thread")]
/// Search related items for item with `key`.
pub async fn get_related_items(key: &str) -> Result<Vec<Item>> {
    crate::profile::update_zotero_db_cache(Db_File.as_ref(), Cached_Db_File.as_ref())?;
    let db = ZoteroDb::connect(Cached_Db_File).await?;
    let items = db.get_related_items(key).await?;
    Ok(items)
}

#[tokio::main(flavor = "current_thread")]
/// Create a new `report` item in zotero with a .note (org-mode) attachment, and
/// returns zotero uri of the new item.
pub async fn get_items_by_tag(tag: &str) -> Result<Vec<Item>> {
    crate::profile::update_zotero_db_cache(Db_File.as_ref(), Cached_Db_File.as_ref())?;
    let db = ZoteroDb::connect(Cached_Db_File).await?;

    let items = db.get_items_by_tag(tag).await?;
    Ok(items)
}

impl Item {
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
// api:1 ends here

// [[file:../zotero.note::*test][test:1]]
#[tokio::test]
async fn test_db() -> Result<()> {
    let url = "/home/ybyygu/.cache/zotero.sqlite";
    let zotero = ZoteroDb::connect(url).await?;

    let x = get_attachment_paths_from_key("I9BXB5GH").await?;
    dbg!(x);

    let x = zotero.get_related_items("FU5SDYIA").await?;
    dbg!(x);

    Ok(())
}
// test:1 ends here
