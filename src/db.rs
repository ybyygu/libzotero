// [[file:../zotero.note::*imports][imports:1]]
use gut::prelude::*;
use std::path::{Path, PathBuf};

use sqlx::prelude::*;
use sqlx::sqlite::SqlitePool;
// imports:1 ends here

// [[file:../zotero.note::b64609c9][b64609c9]]
mod base {
    use super::*;

    pub struct ZoteroDb {
        // https://docs.rs/sqlx/0.5.7/sqlx/pool/struct.Pool.html#why-use-a-pool
        pool: SqlitePool,
    }

    impl ZoteroDb {
        pub async fn connect(uri: &str) -> Result<Self> {
            use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
            // read only access
            let options = SqliteConnectOptions::from_str(uri)?
                .immutable(true)
                .read_only(true);
            let pool = SqlitePool::connect_with(options).await?;
            let db = Self { pool };
            Ok(db)
        }

        pub fn pool(&self) -> &SqlitePool {
            &self.pool
        }
    }
}
pub use base::ZoteroDb;
// b64609c9 ends here

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
    use unicode_width::*;

    // if `s` is too long, truncate it to a short one by display width for nice alignment
    let title = format!("{:width$}", s, width = max_width);
    let s = if s.width() > max_width {
        format!("{}...", &title[..max_width - 3])
    } else {
        title[..max_width].to_string()
    };

    let str_width = s.width();
    let str_len = s.len();
    if str_len == str_width {
        format!("{:width$}", s, width = max_width)
    } else {
        format!("{}{:width$}", s, " ", width = max_width - str_width)
    }
}

#[test]
fn test_str_width() {
    use unicode_width::*;
    let s0 = "Brønsted acidic zeolites";
    let s1 = "Mo̸ller-Plesset 好好";
    let s2 = "101̅0) and (112̅0)";
    let s3 = "1010) and (1120)";
    let s4 = "Mo̸ller-Plesset 好好                    xx";
    let s5 = "Sites Converge?†";
    let s6 = "Selectivity—Modeled by QM/MM Calculations";
    assert_eq!(get_aligned_string(s0, 40).width(), 40);
    assert_eq!(get_aligned_string(s1, 40).width(), 40);
    assert_eq!(get_aligned_string(s2, 40).width(), 40);
    assert_eq!(get_aligned_string(s3, 40).width(), 40);
    assert_eq!(get_aligned_string(s4, 40).width(), 40);
    assert_eq!(get_aligned_string(s5, 40).width(), 40);
    assert_eq!(get_aligned_string(s6, 40).width(), 40);
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
        // make sure extra in one line
        let extra = self.extra.replace("\n", "; ");
        write!(f, "{} => {} | {:} | {}", self.key, self.date, &title, extra)
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

// 4VH9GANA => 2009 | Do Quantum Mechanical Energies Calculated for Small Models of Protein-Active Sites Converge?†        |
// FIIAZG4V => 2010 | P450 Enzymes: Their Structure, Reactivity, and Selectivity—Modeled by QM/MM Calculations             |
// JVGGKSCS => 2008 | A theoretical investigation into the thiophene-cracking mechanism over pure Brønsted acidic zeolite  |
// #[tokio::test]
// async fn test_item_key_format() {
//     let url = "/home/ybyygu/.cache/zotero.sqlite";
//     let zotero = ZoteroDb::connect(url).await.unwrap();
//     let item = zotero.get_item("TU57B9TH").await.unwrap();
//     println!("{}", item);
//     let item = zotero.get_item("4VH9GANA").await.unwrap();
//     println!("{}", item);
//     let item = zotero.get_item("FIIAZG4V").await.unwrap();
//     println!("{}", item);
//     let item = zotero.get_item("JVGGKSCS").await.unwrap();
//     println!("{}", item);
//     let item = zotero.get_item("N9GGY79G").await.unwrap();
//     println!("{}", item);
// }
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
    /// Search zotero items by `tag`
    async fn get_items_by_tag(&self, tag: &str) -> Result<Vec<Item>> {
        let items = sqlx::query_as::<_, KvRec>(
            r#"
SELECT items.key as key, tags.name as value FROM items
    JOIN itemTags USING (itemID)
    JOIN tags USING (tagID)
    WHERE LOWER(name) like ?
    -- exclude deleted items
    AND items.itemID NOT IN (select itemID from deletedItems)
"#,
        )
        .bind(format!("%{}%", tag.to_lowercase()))
        .fetch_all(self.pool())
        .await?;

        // get other fields, such as title and date
        let mut all = vec![];
        for x in items {
            let item = self.get_item(&x.key).await?;
            all.push(item);
        }
        Ok(all)
    }

    /// Get zotero item in `key` will interesting fields filled.
    async fn get_item(&self, key: &str) -> Result<Item> {
        let sql = r#"
SELECT fields.FieldName as key, itemDataValues.value as value
    FROM itemData
    LEFT JOIN items ON itemData.itemID = items.itemID
    LEFT JOIN fields ON itemData.fieldID = fields.fieldID
    LEFT JOIN itemDataValues ON itemData.valueID = itemDataValues.valueID
    WHERE items.key = ?
      AND fields.fieldName IN ("extra", "date", "publicationTitle", "title")
"#;

        let recs = sqlx::query_as::<_, KvRec>(sql).bind(key).fetch_all(self.pool()).await?;
        let d: Map = recs.into_iter().map(|x| (x.key, x.value)).collect();
        let item = Item {
            key: key.into(),
            extra: d.get("extra").unwrap_or(&String::new()).to_string(),
            title: d.get("title").unwrap_or(&String::new()).to_string(),
            date: d.get("date").unwrap_or(&"0000".to_string())[..4].to_string(),
            ..Default::default()
        };

        Ok(item)
    }
}
// tags:1 ends here

// [[file:../zotero.note::*collection][collection:1]]
impl ZoteroDb {
    /// Search zotero items by `collection`
    async fn get_items_by_collection(&self, collection: &str) -> Result<Vec<Item>> {
        let items = sqlx::query_as::<_, KvRec>(
            r#"
SELECT items.key as key, collectionName as value from collections
    JOIN collectionItems USING (collectionID)
    JOIN items using (itemID)
    WHERE LOWER(collections.collectionName) like ?
"#,
        )
        .bind(format!("%{}%", collection.to_lowercase()))
        .fetch_all(self.pool())
        .await?;

        // get other fields, such as title and date
        let mut all = vec![];
        for x in items {
            let item = self.get_item(&x.key).await?;
            all.push(item);
        }
        Ok(all)
    }
}
// collection:1 ends here

// [[file:../zotero.note::8ae891d8][8ae891d8]]
impl ZoteroDb {
    // key: "2X4DGF8X",
    // value: "http://zotero.org/users/15074/items/9F6B5E9G",
    async fn get_related_items(&self, key: &str) -> Result<Vec<Item>> {
        let recs = sqlx::query_as::<_, KvRec>(
            r#"
SELECT items.key as key, itemRelations.object as value FROM items
JOIN itemRelations USING (itemID)
WHERE predicateID == 2
      -- only matching parent item, not attachment
      AND itemTypeID != 14
      -- exclude deleted items
      AND items.itemID NOT IN (select itemID from deletedItems)
      AND items.key = ?
"#,
        )
        .bind(key)
        .fetch_all(self.pool())
        .await?;

        let mut related = vec![];
        for rec in recs {
            if let Some(key) = parse_zotero_key_from_object_url(&rec.value) {
                let item = self.get_item(&key).await?;
                related.push(item);
            }
        }
        Ok(related)
    }
}

/// Return related items with item in `key`
async fn get_related_items_from_key(key: &str) -> Result<Vec<Item>> {
    let db = ZoteroDb::connect(DB_FILE).await?;
    let items = db.get_related_items(key).await?;
    Ok(items)
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
// 8ae891d8 ends here

// [[file:../zotero.note::c91d3b45][c91d3b45]]
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
  -- exclude deleted items
  AND itemAttachments.itemID NOT IN (select itemID from deletedItems)
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
    let db = ZoteroDb::connect(DB_FILE).await?;

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
    let zotero_storage_root = "/home/ybyygu/Documents/Data/zotero/storage";
    let attach_path = if path.starts_with("storage:") { &path[8..] } else { path };
    format!("{}/{}/{}", zotero_storage_root, key, attach_path)
}
// c91d3b45 ends here

// [[file:../zotero.note::cdcbd2e6][cdcbd2e6]]
static DB_FILE: &str = "/home/ybyygu/Documents/Data/zotero/zotero.sqlite";

#[tokio::main(flavor = "current_thread")]
/// Quick search zotero items
pub async fn get_items_dwim(keyword: &str) -> Result<Vec<Item>> {
    let db = ZoteroDb::connect(DB_FILE).await?;

    // let items = db.get_items_dwim(keyword).await?;
    // Ok(items)
    todo!();
}

/// Extract item key from link in zotero protocol
pub fn get_item_key_from_link(link: &str) -> Result<String> {
    ZoteroDb::get_item_key_from_link(link)
}

#[tokio::main(flavor = "current_thread")]
/// Create a new `report` item in zotero with a .note (org-mode) attachment, and
/// returns zotero uri of the new item.
pub async fn get_items_by_tag(tag: &str) -> Result<Vec<Item>> {
    let db = ZoteroDb::connect(DB_FILE).await?;

    let items = db.get_items_by_tag(tag).await?;
    Ok(items)
}

#[tokio::main(flavor = "current_thread")]
/// Search zotero items by collection name
pub async fn get_items_by_collection(name: &str) -> Result<Vec<Item>> {
    let db = ZoteroDb::connect(DB_FILE).await?;

    let items = db.get_items_by_collection(name).await?;
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

    #[tokio::main(flavor = "current_thread")]
    /// Return a list of related items
    pub async fn get_related_items(&self) -> Result<Vec<Item>> {
        let related = get_related_items_from_key(&self.key).await?;
        Ok(related)
    }
}
// cdcbd2e6 ends here

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
