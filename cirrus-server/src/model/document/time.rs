use bson::{Document, doc};

pub fn create_update_timestamp(timestamp: i64) -> Document {
    doc! {
        "$set": {"modified_timestamp": timestamp}
    }
}