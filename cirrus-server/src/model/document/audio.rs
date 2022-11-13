use bson::{Document, doc, oid::ObjectId};

pub fn update_audio_tag_referer(tag_id: &ObjectId) -> Document {
    doc! {
        "$set": {"audio_tag_refer": tag_id}
    }
}

pub fn query_audio_tag_referer(ref_id: &ObjectId) -> Document {
    doc! {
        "audio_tag_refer": ref_id
    }
}