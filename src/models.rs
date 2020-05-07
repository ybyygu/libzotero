#[derive(Queryable, Debug, Clone)]
pub struct ItemAttachment {
    pub id: Option<i32>,
    pub parent_item_id: Option<i32>,
    pub content_type: Option<String>,
    pub path: Option<String>,
}

#[derive(Queryable, Debug, Clone)]
pub struct Item {
    pub id: Option<i32>,
    pub type_id: i32,
    pub key: String,
}
