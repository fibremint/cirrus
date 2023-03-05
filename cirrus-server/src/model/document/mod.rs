use bson::{Document, doc, oid::ObjectId};
use serde::Serialize;

pub mod path;
pub mod time;
pub mod audio;

pub fn query_single_id(id: &ObjectId) -> Document {
    doc! {
        "_id": id
    }
}

pub fn query_many_id(ids: &Vec<ObjectId>) -> Document {
    doc! {
        "_id": {
            "$in": ids
        }
    }
}

pub fn update_doc<T: Serialize>(doc: T) -> Result<Document, anyhow::Error> {
    let doc = mongodb::bson::to_document(&doc)?;

    Ok(doc! {
            "$set": doc
        }
    )
}