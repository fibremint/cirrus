use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub fn get_hashed_value(target: &impl Hash) -> i64 {
    let mut hasher = DefaultHasher::new();
    
    target.hash(&mut hasher);
    hasher.finish() as i64
}