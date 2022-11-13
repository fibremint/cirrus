use bson::{Document, doc};

pub fn query_path(key: &str, path: &str) -> Document {
    doc! {
        key: { "$regex": format!("^{}", path) }
    }
}