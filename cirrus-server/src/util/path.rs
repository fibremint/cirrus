use std::path::Path;

use chrono::DateTime;

pub fn replace_with_common_separator(path: &str) -> String {
    path.replace(std::path::MAIN_SEPARATOR, "/")
}

// ref: https://docs.mongodb.com/manual/tutorial/model-tree-structures-with-materialized-paths/
pub fn path_to_materialized(path: &Path) -> String {
    let path = path.to_str().unwrap();
    let path = replace_with_common_separator(path);
    let path = path.replace("/", ",");

    format!(",{},", path)
}

pub fn materialized_to_path(materialized_path: &str) -> String {
    let (path_slice_start, path_slice_end) = (1 as usize, materialized_path.len() - 1);
    let path = &materialized_path[path_slice_start..path_slice_end];

    path.replace(",", "/")
}

pub fn get_timestamp(path: &Path) -> i64 {
    let path_modified_time = path.metadata().unwrap().modified().unwrap();
    let path_modified_time = DateTime::<chrono::Utc>::from(path_modified_time);

    path_modified_time.timestamp()
}