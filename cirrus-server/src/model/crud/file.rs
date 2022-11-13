use crate::{
    model::{GetCollection, dto}
};

use super::{CrudMany, PathOperation, CrudSingle};

pub struct AudioFile {
    pub single: CrudSingle<dto::AudioFile>,
    pub many: CrudMany<dto::AudioFile>,
    pub path: PathOperation<dto::AudioFile>,
}

impl GetCollection<dto::AudioFile> for AudioFile {
    fn get_collection(db: mongodb::Client) -> mongodb::Collection<dto::AudioFile> {
        db.database("cirrus").collection::<dto::AudioFile>("audio")
    }
}

impl Default for AudioFile {
    fn default() -> Self {
        Self { 
            single: CrudSingle::new(Self::get_collection), 
            many: CrudMany::new(Self::get_collection), 
            path: PathOperation::new(Self::get_collection)
        }
    }
}
