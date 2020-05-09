table! {
    itemAttachments(itemID) {
        itemID -> Integer,
        parentItemID -> Nullable<Integer>,
        contentType -> Nullable<Text>,
        path -> Nullable<Text>,
    }
}

table! {
    items(itemID) {
        itemID -> Integer,
        itemTypeID -> Integer,
        key -> Text,
    }
}

table! {
    itemTags(itemID, tagID) {
        itemID -> Integer,
        tagID -> Integer,
    }
}

table! {
    tags(tagID) {
        tagID -> Integer,
        name -> Text,
    }
}

table! {
    creators(creatorID) {
        creatorID -> Nullable<Integer>,
        firstName -> Nullable<Text>,
        lastName -> Nullable<Text>,
    }
}

table! {
    itemCreator(itemID, creatorID, creatorTypeID, orderIndex) {
        itemID -> Integer,
        creatorID -> Integer,
        creatorTypeID -> Integer,
        orderIndex -> Integer,
    }
}

table! {
    itemData(itemID, fieldID) {
        itemID -> Nullable<Integer>,
        fieldID -> Nullable<Integer>,
        valueID -> Nullable<Integer>,
    }
}

joinable!(itemAttachments -> items(parentItemID));

allow_tables_to_appear_in_same_query! {
    items, itemAttachments
}
