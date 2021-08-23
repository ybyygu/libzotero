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
    fields(fieldID) {
        fieldID -> Nullable<Integer>,
        fieldName -> Nullable<Text>,
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
        itemID -> Integer,
        fieldID -> Nullable<Integer>,
        valueID -> Nullable<Integer>,
    }
}

table! {
    itemDataValues(valueID) {
        valueID -> Nullable<Integer>,
        value -> Nullable<Text>,
    }
}

table! {
    itemRelations(itemID, predicateID, object) {
        itemID -> Integer,
        predicateID -> Integer,
        object -> Text,
    }
}

joinable!(itemAttachments -> items(parentItemID));
joinable!(itemRelations -> items(itemID));
joinable!(itemData -> itemDataValues(valueID));
joinable!(itemData -> fields(fieldID));
joinable!(itemData -> items(itemID));
joinable!(itemTags -> items(itemID));
joinable!(itemTags -> tags(tagID));

allow_tables_to_appear_in_same_query! {
    items, itemTags, itemData, itemDataValues, fields, tags, itemAttachments, itemRelations,
}
