use crate::{
    model::{GetCollection, dto}
};

use super::{CrudMany, PathOperation, CrudSingle};

pub struct AudioLibraryRoot {
    pub single: CrudSingle<dto::AudioLibrary>,
    pub many: CrudMany<dto::AudioLibrary>,
    pub path: PathOperation<dto::AudioLibrary>
}

impl GetCollection<dto::AudioLibrary> for AudioLibraryRoot {
    fn get_collection(db: mongodb::Client) -> mongodb::Collection<dto::AudioLibrary> {
        db.database("cirrus").collection::<dto::AudioLibrary>("library_roots")
    }
}

impl Default for AudioLibraryRoot {
    fn default() -> Self {
        Self { 
            single: CrudSingle::new(Self::get_collection), 
            many: CrudMany::new(Self::get_collection), 
            path: PathOperation::new(Self::get_collection),
        }
    }
}

pub struct AudioLibrary {
    pub single: CrudSingle<dto::AudioLibrary>,
    pub many: CrudMany<dto::AudioLibrary>,
    pub path: PathOperation<dto::AudioLibrary>
}

impl GetCollection<dto::AudioLibrary> for AudioLibrary {
    fn get_collection(db: mongodb::Client) -> mongodb::Collection<dto::AudioLibrary> {
        db.database("cirrus").collection::<dto::AudioLibrary>("libraries")
    }
}

impl Default for AudioLibrary {
    fn default() -> Self {
        Self { 
            single: CrudSingle::new(Self::get_collection),
            many: CrudMany::new(Self::get_collection),
            path: PathOperation::new(Self::get_collection),
        }
    }
}
