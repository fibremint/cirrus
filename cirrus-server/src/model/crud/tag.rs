use crate::{
    model::{GetCollection, dto}
};

use super::{CrudMany, CrudSingle, PathOperation, Pagination};

pub struct AudioTag {
    pub single: CrudSingle<dto::AudioTag>,
    pub many: CrudMany<dto::AudioTag>,
    pub path: PathOperation<dto::AudioTag>,
    pub page: Pagination<dto::AudioTag>,
}

impl GetCollection<dto::AudioTag> for AudioTag {
    fn get_collection(db: mongodb::Client) -> mongodb::Collection<dto::AudioTag> {
        db.database("cirrus")
            .collection::<dto::AudioTag>("audio_tag")
    }
}

impl Default for AudioTag {
    fn default() -> Self {
        Self { 
            single: CrudSingle::new(Self::get_collection), 
            many: CrudMany::new(Self::get_collection), 
            path: PathOperation::new(Self::get_collection),
            page: Pagination::new(Self::get_collection),
        }
    }
}
