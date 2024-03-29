use std::{
    path::{Path, PathBuf}, collections::{HashMap, HashSet},
};

use bson::oid::ObjectId;

use itertools::Itertools;
use mongodb::bson;
use walkdir::{DirEntry, WalkDir};

use crate::{
    util, 
    model::{crud, dto::{self, GetPathValue}}
};

// * path not exist -> return not found
// * path is added already -> return added already
fn get_audio_library_entries(path: &Path, audio_types: &Vec<&str>) -> Vec<DirEntry> {
    let audio_library_entries: Vec<_> = WalkDir::new(path)
        .into_iter()
        .filter_map(|item| item.ok()
            .and_then(|entry| entry.path().is_dir().then(|| entry))
            .and_then(|entry2| {
                let mut current_dir = std::fs::read_dir(entry2.path()).unwrap();
                current_dir.any(|content_entry| {
                    match content_entry.unwrap().path().extension() {
                        Some(extension) => return audio_types.contains(&extension.to_str().unwrap()),
                        None => false,
                    }
                }).then(|| entry2)
            })
        )
        .collect();

    audio_library_entries
}

fn get_audio_file_paths(current_path: &Path, audio_types: &Vec<&str>) -> Vec<PathBuf> {
    let audio_file_dir = std::fs::read_dir(current_path).unwrap();
    let audio_file_paths: Vec<_> = audio_file_dir
        .into_iter()
        .filter_map(|item| item.ok()
            .and_then(|entry| entry.path().is_file().then(|| entry.path()))
            .and_then(|pathbuf| {
                match pathbuf.extension() {
                    Some(file_extension) => {
                        let file_extension = file_extension.to_str().unwrap();
                        return audio_types.contains(&file_extension).then(|| pathbuf)
                    },
                    None => None,
                }
            })
        )
        .collect();
    
    audio_file_paths
}

pub struct AudioLibrary {
    crud_audio_lib: crud::AudioLibrary,
    crud_audio_lib_root: crud::AudioLibraryRoot,
    crud_audio_file: crud::AudioFile,
    crud_audio_tag: crud::AudioTag,
}

impl Default for AudioLibrary {
    fn default() -> Self {
        Self { 
            crud_audio_lib: Default::default(),
            crud_audio_lib_root: Default::default(),
            crud_audio_file: Default::default(),
            crud_audio_tag: Default::default(),
        }
    }
}

impl AudioLibrary {
    pub async fn add_audio_library(
        &self,
        db: mongodb::Client,
        library_root: &Path
    ) -> Result<String, anyhow::Error> {
        if !library_root.exists() {
            return Err(anyhow::anyhow!("library {} does not exists", library_root.to_str().unwrap()))
        }

        if self.crud_audio_lib_root.path.check_exists_by_path(db.clone(), library_root).await? {
            return Err(anyhow::anyhow!("path '{:?}' already exists", library_root))
        }

        let audio_types = vec!["aiff"];

        let audio_library_entries = get_audio_library_entries(library_root, &audio_types);
        let audio_file_docs = audio_library_entries
            .iter()
            .map(|item| get_audio_file_paths(item.path(), &audio_types))
            .flat_map(|item| item)
            .map(|item| dto::AudioFile::new(&item))
            .collect::<Vec<_>>();
        
        let library_docs = audio_library_entries
            .iter()
            .map(|item| dto::AudioLibrary::new(&item.path()))
            .collect::<Vec<_>>();

        let create_lib_root_res = match self.crud_audio_lib_root.single.create(
                db.clone(), 
                &dto::AudioLibrary::new(&library_root)
            ).await {
                Ok(res) => res,
                Err(err) => return Err(anyhow::anyhow!(err)),
        };

        if !library_docs.is_empty() {
            self.crud_audio_lib.many.create_many(db.clone(), library_docs).await?;
        }
        
        if !audio_file_docs.is_empty() {
            self.crud_audio_file.many.create_many(db.clone(), audio_file_docs).await?;
        }

        Ok(format!("{:?}", create_lib_root_res.inserted_id))
    }

    pub async fn remove_audio_library(
        &self,
        db: mongodb::Client,
        path: &Path
    ) -> Result<String, anyhow::Error> {
        if !self.crud_audio_lib_root.path.check_exists_by_path(db.clone(), path).await? {
            return Err(anyhow::anyhow!("path '{:?}' not exists", path))
        }

        let mut delete_tag_count = 0;
        let mut delete_file_count = 0;
        let mut delete_library_count = 0;

        let delete_audio_libs = self.crud_audio_lib.path.get_by_path(
                db.clone(), 
                path
            ).await?;
        
        for audio_lib in delete_audio_libs.iter() {
            let audio_files = self.crud_audio_file.path.get_by_materialized_path(
                    db.clone(), 
                    &audio_lib.get_mat_path_val()
                ).await?;

            let (delete_file_ids, delete_tag_ids): (Vec<ObjectId>, Vec<Option<ObjectId>>)  = audio_files
                .into_iter()
                .map(|item| (
                    item.id.unwrap(), 
                    item.audio_tag_refer
                ))
                .unzip();

            let delete_tag_ids = delete_tag_ids
                .into_iter()
                .filter_map(|item| item)
                .collect_vec();

            let delete_audio_tag_res = self.crud_audio_tag
                .many
                .delete_many(
                    db.clone(), 
                    &delete_tag_ids
                ).await?;

            delete_tag_count += delete_audio_tag_res.deleted_count;

            let delete_audio_file_res = self.crud_audio_file
                .many
                .delete_many(
                    db.clone(), 
                    &delete_file_ids
                ).await?;

            delete_file_count += delete_audio_file_res.deleted_count;

            let delete_lib_res = self.crud_audio_lib
                .single
                .delete(
                    db.clone(), 
                    &audio_lib.id.unwrap()
                ).await?;

            delete_library_count += delete_lib_res.deleted_count;
        }

        self.crud_audio_lib_root
            .path
            .delete_by_path(
                db.clone(), 
                path
            ).await?;

        Ok(format!("deleted tag count: {}, deleted file count: {}, deleted library count: {}", delete_tag_count, delete_file_count, delete_library_count))
    }

    pub async fn analyze_audio_library(
        &self,
        db: mongodb::Client,
    ) -> Result<(), anyhow::Error> {
        let audio_libs = self.crud_audio_lib_root
            .many
            .get_all(db.clone())
            .await?;

        for audio_lib in audio_libs.into_iter() {
            let audio_files = self.crud_audio_file
                .path
                .get_by_materialized_path(
                    db.clone(), 
                    &audio_lib.get_mat_path_val()
                ).await?;

            let mut audio_files = audio_files
                .into_iter()
                .filter(|item| item.audio_tag_refer.is_none())
                .collect_vec();

            for audio_file in audio_files.iter_mut() {
                let audio_tag = dto::AudioTag::new(
                        None,
                        &util::path::materialized_to_path(&audio_file.parent_path), 
                        &audio_file.filename
                    )?;

                self.crud_audio_tag.single.create(db.clone(), &audio_tag).await?;

                audio_file.audio_tag_refer = audio_tag.id.clone();

                let update_res = self.crud_audio_file
                    .single
                    .update(
                        db.clone(), 
                        &audio_file.id.unwrap(), 
                        audio_file
                    ).await?;

                println!("ur: {:?}", update_res);
            }
        }

        Ok(())
    }

    pub async fn refresh_audio_library(
        &self,
        db: mongodb::Client,
    ) -> Result<(), anyhow::Error> {
        let audio_lib_roots = self.crud_audio_lib_root.many.get_all(db.clone()).await?;
        let audio_types = vec!["aiff"];

        for audio_lib_root in audio_lib_roots.iter() {
            let audio_libs = self.crud_audio_lib
                .path
                .get_by_materialized_path(
                    db.clone(), 
                    &audio_lib_root.get_mat_path_val()
                ).await?;

            let audio_libraries: HashMap<_, _> = audio_libs.iter()
                .map(|item| (item.os_path.clone(), item))
                .collect();

            let local_audio_library_entreis = get_audio_library_entries(
                Path::new(&audio_lib_root.os_path), 
                &audio_types
            );

            let audio_libraries_keys: HashSet<_> = audio_libraries
                .iter()
                .map(|(k, _)| util::path::replace_with_common_separator(k))
                .collect();
            let local_audio_libraries_keys: HashSet<_> = local_audio_library_entreis
                .iter()
                .map(|item| {
                    util::path::replace_with_common_separator(item.path().to_str().unwrap())
                })
                .collect();

            let new_library_pathstrs: HashSet<_> = local_audio_libraries_keys.difference(&audio_libraries_keys).cloned().collect();
            let deleted_library_pathstrs: HashSet<_> = audio_libraries_keys.difference(&local_audio_libraries_keys).cloned().collect();
            let managed_library_pathstrs: HashSet<_> = audio_libraries_keys.difference(&deleted_library_pathstrs).collect();
            let updated_local_libraries: Vec<_> = managed_library_pathstrs.into_iter()
                .filter_map(|item| audio_libraries.get(item.as_str())
                    .and_then(|audio_library| audio_library.check_modified().then(|| audio_library)))
                .collect();

            println!("nl: {:?}, dl: {:?}, ull: {:?}", new_library_pathstrs, deleted_library_pathstrs, updated_local_libraries);

            if !new_library_pathstrs.is_empty() {
                let new_audio_file_docs: Vec<_> = new_library_pathstrs
                    .iter()
                    .map(|item| get_audio_file_paths(Path::new(item), &audio_types))
                    .flat_map(|item| item)
                    .map(|item| dto::AudioFile::new(&item))
                    .collect();

                let new_library_docs: Vec<_> = new_library_pathstrs
                    .iter()
                    .map(|item| dto::AudioLibrary::new(Path::new(&item)))
                    .collect();

                self.crud_audio_lib.many.create_many(db.clone(), new_library_docs).await?;

                self.crud_audio_file.many.create_many(db.clone(), new_audio_file_docs).await?;
    
            }

            if !deleted_library_pathstrs.is_empty() {
                for deleted_library_pathstr in deleted_library_pathstrs.iter() {
                    println!("sync delete audio library: {:?}", deleted_library_pathstr);
                    let deleted_audio_lib_path = Path::new(deleted_library_pathstr);

                    let audio_files = self.crud_audio_file.path.get_by_path(db.clone(), deleted_audio_lib_path).await?;
                    let delete_audio_tag_ids: Vec<_> = audio_files.iter()
                        .filter_map(|item| item.audio_tag_refer)
                        .collect();

                    let _delete_audio_tag_res = self.crud_audio_tag
                        .many
                        .delete_many(
                            db.clone(), 
                            &delete_audio_tag_ids
                        ).await?;

                    let _delete_audio_file_res = self.crud_audio_file
                        .many
                        .delete_many(
                            db.clone(), 
                            &audio_files.iter().map(|item| item.id.unwrap()).collect_vec()
                        ).await?;
            
                    let _delete_audio_lib_res = self.crud_audio_lib
                        .path
                        .delete_by_path(
                            db.clone(), 
                            deleted_audio_lib_path,
                        ).await?;
                }
            }

            if !updated_local_libraries.is_empty() {
                println!("sync updated local libraries: {:?}", updated_local_libraries);
                
                for updated_local_library in updated_local_libraries.into_iter() {
                    let local_library_path = Path::new(&updated_local_library.os_path);

                    let audio_files = self.crud_audio_file
                        .path
                        .get_by_path(
                            db.clone(), 
                            local_library_path
                        ).await?;

                    let audio_filenames: HashSet<_> = audio_files
                        .iter()
                        .map(|item| item.filename.to_owned())
                        .collect();
                    let mut audio_files: HashMap<String, dto::AudioFile> = audio_files
                        .into_iter()
                        .map(|item| (item.filename.to_owned(), item))
                        .collect();

                    let local_audio_file_paths = get_audio_file_paths(
                        local_library_path, 
                        &audio_types
                    );

                    let local_audio_filenames: HashSet<_> = local_audio_file_paths
                        .iter()
                        .filter_map(|item| item.file_name()
                            .and_then(|item2| item2.to_str()))
                        .map(|item| item.to_owned())
                        .collect();
                    
                    let new_audio_filenames: HashSet<_> = local_audio_filenames.difference(&audio_filenames).cloned().collect();
                    let deleted_audio_filenames: HashSet<_> = audio_filenames.difference(&local_audio_filenames).cloned().collect();
                    let managed_audio_filenames: HashSet<_> = audio_filenames.difference(&deleted_audio_filenames).cloned().collect();

                    let mut updated_audio_files: Vec<dto::AudioFile> = vec![];
                    let mut updated_audio_tags: Vec<dto::AudioTag> = vec![];

                    for managed_audio_filename in managed_audio_filenames.iter() {
                        let mut audio_file = audio_files.remove(managed_audio_filename).unwrap();
                        if audio_file.check_modified() {
                            match audio_file.audio_tag_refer {
                                Some(audio_tag_id) => {
                                    let parent_path = util::path::materialized_to_path(&audio_file.get_mat_path_val());
                                    let updated_audio_tag = dto::AudioTag::new(Some(audio_tag_id), &parent_path, &audio_file.filename)?;
                                    updated_audio_tags.push(updated_audio_tag);
                                },
                                None => (),
                            }

                            audio_file.update_modified_timestamp();

                            updated_audio_files.push(audio_file);
                        }
                    }

                    let new_audio_file_docs: Vec<_> = new_audio_filenames
                        .iter()
                        .map(|item| {
                            let mut target_path = local_library_path.clone().to_path_buf();
                            target_path.push(item);

                            dto::AudioFile::new(&target_path)
                        })
                        .collect();

                    let delete_audio_file_docs: Vec<_> = deleted_audio_filenames
                        .iter()
                        .filter_map(|item| audio_files.remove(item))
                        .collect();

                    if !new_audio_file_docs.is_empty() {
                        self.crud_audio_file
                            .many
                            .create_many(
                                db.clone(), 
                                new_audio_file_docs
                            ).await?;
                    }

                    if !delete_audio_file_docs.is_empty() {
                        let deleted_audio_tag_ids: Vec<_> = delete_audio_file_docs
                            .iter()
                            .filter_map(|item| item.audio_tag_refer)
                            .collect();

                        self.crud_audio_tag
                            .many
                            .delete_many(
                                db.clone(), 
                                &deleted_audio_tag_ids
                            ).await?;

                        self.crud_audio_file
                            .many
                            .delete_many(
                                db.clone(), 
                                &delete_audio_file_docs.iter().map(|item| item.id.unwrap()).collect_vec()
                            ).await?;
                    }

                    if !updated_audio_files.is_empty() {
                        self.crud_audio_file
                            .many
                            .update_many(
                                db.clone(), 
                                &updated_audio_files
                            ).await;
                    }

                    if !updated_audio_tags.is_empty() {
                        self.crud_audio_tag
                            .many
                            .update_many(
                                db.clone(), 
                                &updated_audio_tags
                            ).await;
                    }

                    let modified_ts = util::path::get_timestamp(&local_library_path);
                    let _update_local_library_res = self.crud_audio_lib
                        .path
                        .update_modified_timestamp(
                            db.clone(), 
                            &updated_local_library, 
                            modified_ts
                        ).await?;
                }
            }

        }

        Ok(())
    }
}
