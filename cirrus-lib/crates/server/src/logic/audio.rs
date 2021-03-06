use std::{
    fs::File,
    path::{Path, PathBuf}, collections::{HashMap, HashSet, hash_map::DefaultHasher}, hash::{Hash, Hasher}, ops::{Deref, DerefMut}, borrow::{BorrowMut, Borrow}, rc::Rc, sync::Arc,
};

use aiff::reader::AiffReader;
use bson::oid::ObjectId;
use chrono::{DateTime, NaiveDateTime, Utc, TimeZone};
use cirrus_grpc::api::{
    AudioDataRes, AudioMetaRes, AudioTagRes
};
// use futures::{TryStreamExt};
use mongodb::{bson::{Document, doc}, options::FindOptions, results::DeleteResult};
use tokio::sync::{Mutex, MutexGuard};
use walkdir::{DirEntry, WalkDir};

use crate::{
    util, 
    model::{self, document}
};

pub struct AudioFile {}

impl AudioFile {
    pub async fn read_meta(
        mongodb_client: mongodb::Client,
        audio_tag_id: &str
    ) -> Result<AudioMetaRes, String> {
        // let file = File::open(filepath)?;
        let audio_tag_id = ObjectId::parse_str(audio_tag_id).unwrap();
        let audio_file = model::AudioFile::find_by_audio_tag_id(mongodb_client.clone(), audio_tag_id).await.unwrap();

        let audio_file = match audio_file {
            Some(audio_file) => audio_file,
            None => return Err(String::from("failed to retrieve audio file information")),
        };

        // let audio_file_path = util::path::materialized_to_path(audio_file.get_path())

        let file = match File::open(audio_file.get_path()) {
            Ok(file) => file,
            Err(err) => return Err(String::from("failed to load file")),
        };

        let mut reader = AiffReader::new(file);
        // reader.read().unwrap();
        match reader.read() {
            Ok(_) => (),
            Err(err) => match err {
                aiff::chunks::ChunkError::InvalidID(id) => return Err(String::from("invalid id")),
                aiff::chunks::ChunkError::InvalidFormType(id) => return Err(String::from("invalid form type")),
                aiff::chunks::ChunkError::InvalidID3Version(ver) => return Err(String::from("invalid id3 version")),
                aiff::chunks::ChunkError::InvalidSize(exp, actual) => return Err(format!("invalid size, expected: {}, actual: {}", exp, actual)),
                aiff::chunks::ChunkError::InvalidData(msg) => return Err(msg.to_string()),
            },
        }

        let common = reader.form().as_ref().unwrap().common().as_ref().unwrap();
        let sound = reader.form().as_ref().unwrap().sound().as_ref().unwrap();

        Ok(AudioMetaRes {
            bit_rate: common.bit_rate as u32,
            block_size: sound.block_size,
            channels: common.num_channels as u32,
            offset: sound.offset,
            sample_frames: common.num_sample_frames,
            sample_rate: common.sample_rate as u32,
            size: sound.size as u32,
        })
    }

    pub async fn read_data(
        mongodb_client: mongodb::Client,
        audio_tag_id: &str,
        byte_start: usize, 
        byte_end: usize
    ) -> Result<AudioDataRes, String> {
        // let file = File::open(filepath)?;
        let audio_tag_id = ObjectId::parse_str(audio_tag_id).unwrap();
        let audio_file = model::AudioFile::find_by_audio_tag_id(mongodb_client.clone(), audio_tag_id).await.unwrap();

        let audio_file = match audio_file {
            Some(audio_file) => audio_file,
            None => return Err(String::from("failed to retrieve audio file information")),
        };

        let file = match File::open(audio_file.get_path()) {
            Ok(file) => file,
            Err(err) => return Err(String::from("failed to load file")),
        };
    
        let mut reader = AiffReader::new(file);
        // reader.read().unwrap();
        match reader.read() {
            Ok(_) => (),
            Err(err) => match err {
                aiff::chunks::ChunkError::InvalidID(id) => return Err(String::from("invalid id")),
                aiff::chunks::ChunkError::InvalidFormType(id) => return Err(String::from("invalid form type")),
                aiff::chunks::ChunkError::InvalidID3Version(ver) => return Err(String::from("invalid id3 version")),
                aiff::chunks::ChunkError::InvalidSize(exp, actual) => return Err(format!("invalid size, expected: {}, actual: {}", exp, actual)),
                aiff::chunks::ChunkError::InvalidData(msg) => return Err(msg.to_string()),
            },
        }
    
        let reader_form_ref = reader.form().as_ref().unwrap();
        let data = reader_form_ref.sound().as_ref().unwrap();
        let mut audio_data_part = Vec::<u8>::new();
        audio_data_part.extend_from_slice(&data.sound_data[4*byte_start..4*byte_end]);
    
        Ok(AudioDataRes {
            content: audio_data_part
        })
    }
}

pub struct AudioLibrary {}

impl AudioLibrary {
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

    pub async fn add_audio_library(
        mongodb_client: mongodb::Client,
        library_root: &Path
    ) -> Result<String, String> {
        if !library_root.exists() {
            return Err(String::from("not exists"))
        }

        if model::AudioLibraryRoot::check_exists_by_path(mongodb_client.clone(), library_root).await {
            return Err(format!("path '{:?}' already exists", library_root))
        }

        let audio_types = vec!["aiff"];

        let audio_library_entries = Self::get_audio_library_entries(library_root, &audio_types);
        let audio_file_docs: Vec<_> = audio_library_entries
            .iter()
            .map(|item| Self::get_audio_file_paths(item.path(), &audio_types))
            .flat_map(|item| item)
            .map(|item| document::AudioFile::create_from_path(&item))
            .collect();
        
        let library_docs: Vec<_> = audio_library_entries
            .iter()
            .map(|item| document::AudioLibrary::create_from_path(&item.path()))
            .collect();

        let audio_library_root_doc = document::AudioLibrary::create_from_path(&library_root);

        let library_create_res = model::AudioLibraryRoot::create(mongodb_client.clone(), audio_library_root_doc).await;

        if !library_docs.is_empty() {
            model::AudioLibrary::create_many(mongodb_client.clone(), library_docs).await.unwrap();
        }
        
        if !audio_file_docs.is_empty() {
            model::AudioFile::create_many(mongodb_client.clone(), &audio_file_docs).await.unwrap();
        }

        match library_create_res {
            Ok(res) => return Ok(format!("{:?}", res.inserted_id)),
            Err(err) => return Err(format!("failed to create library {:?}", library_root)),
        }
    }

    pub async fn remove_audio_library(
        mongodb_client: mongodb::Client,
        path: &Path
    ) -> Result<String, String> {
        if !model::AudioLibraryRoot::check_exists_by_path(mongodb_client.clone(), path).await {
            return Err(format!("path '{:?}' not exists", path))
        }

        let mut delete_tag_count = 0;
        let mut delete_file_count = 0;
        let mut delete_library_count = 0;

        let delete_audio_libraries = model::AudioLibrary::get_by_path(mongodb_client.clone(), path).await.unwrap();
        for delete_audio_library in delete_audio_libraries.iter() {
            let delete_audio_library_path = util::path::materialized_to_path(&delete_audio_library.path.as_ref().unwrap());
            let delete_audio_library_path = Path::new(&delete_audio_library_path);
            let audio_files = model::AudioFile::get_self_by_library_path(mongodb_client.clone(), delete_audio_library_path, false).await.unwrap();
            let delete_audio_tag_ids: Vec<_> = audio_files.iter()
                .filter_map(|item| item.audio_tag_refer)
                .collect();
    
            let audio_tag_delete_res = model::AudioTag::delete_by_ids(mongodb_client.clone(), &delete_audio_tag_ids).await.unwrap();
            delete_tag_count += audio_tag_delete_res.deleted_count;
    
            let audio_file_delete_res = model::AudioFile::delete_by_selfs(mongodb_client.clone(), &audio_files).await.unwrap();
            delete_file_count += audio_file_delete_res.deleted_count;
    
            let library_delete_res = model::AudioLibrary::delete_by_path(mongodb_client.clone(), delete_audio_library_path).await.unwrap();
            delete_library_count += library_delete_res.deleted_count;
        }

        model::AudioLibraryRoot::delete_by_path(mongodb_client.clone(), path).await;

        Ok(format!("deleted tag count: {}, deleted file count: {}, deleted library count: {}", delete_tag_count, delete_file_count, delete_library_count))
    }

    pub async fn analyze_audio_library(
        mongodb_client: mongodb::Client,
    ) -> Result<(), String> {
        let audio_libraries = model::AudioLibraryRoot::get_all(mongodb_client.clone()).await;

        for audio_library in audio_libraries.into_iter() {
            let audio_files = model::AudioFile::get_self_by_library_path(mongodb_client.clone(), Path::new(&audio_library.id), true).await.unwrap();

            for audio_file in audio_files.iter() {
                let parent_path = util::path::materialized_to_path(&audio_file.parent_path);
                let audio_tag = Self::create_audio_tag(None, &parent_path, &audio_file.filename);
                let audio_tag_id = audio_tag.id.clone();
                
                match model::AudioTag::create(mongodb_client.clone(), audio_tag).await {
                    Ok(_) => (),
                    Err(err) => return Err(format!("{}", err)),
                }

                let update_res = model::AudioFile::set_audio_tag_refer(mongodb_client.clone(), &audio_file.id.unwrap(), &audio_tag_id.unwrap()).await.unwrap();
                println!("ur: {:?}", update_res);
            }
        }

        Ok(())
    }

    pub async fn refresh_audio_library(
        mongodb_client: mongodb::Client,
    ) -> Result<(), String> {
        let audio_library_roots = model::AudioLibraryRoot::get_all(mongodb_client.clone()).await;
        let audio_types = vec!["aiff"];

        for audio_library_root in audio_library_roots.iter() {
            let audio_libraries = model::AudioLibrary::get_by_path(mongodb_client.clone(), Path::new(&audio_library_root.id)).await.unwrap();
            let audio_libraries: HashMap<_, _> = audio_libraries.iter()
                .map(|item| (item.id.as_str(), item))
                .collect();
            let local_audio_library_entreis = Self::get_audio_library_entries(Path::new(&audio_library_root.id), &audio_types);

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
                    .map(|item| Self::get_audio_file_paths(Path::new(item), &audio_types))
                    .flat_map(|item| item)
                    .map(|item| document::AudioFile::create_from_path(&item))
                    .collect();

                let new_library_docs: Vec<_> = new_library_pathstrs
                    .iter()
                    .map(|item| document::AudioLibrary::create_from_path(Path::new(&item)))
                    .collect();

                model::AudioLibrary::create_many(mongodb_client.clone(), new_library_docs).await.unwrap();

                model::AudioFile::create_many(mongodb_client.clone(), &new_audio_file_docs).await.unwrap();
    
            }

            if !deleted_library_pathstrs.is_empty() {
                for deleted_library_pathstr in deleted_library_pathstrs.iter() {
                    println!("sync delete audio library: {:?}", deleted_library_pathstr);
                    let delted_library_path = Path::new(deleted_library_pathstr);

                    let audio_files = model::AudioFile::get_self_by_library_path(mongodb_client.clone(), delted_library_path, false).await.unwrap();
                    let delete_audio_tag_ids: Vec<_> = audio_files.iter()
                        .filter_map(|item| item.audio_tag_refer)
                        .collect();

                    let audio_tag_delete_res = model::AudioTag::delete_by_ids(mongodb_client.clone(), &delete_audio_tag_ids).await.unwrap();

                    let audio_file_delete_res = model::AudioFile::delete_by_selfs(mongodb_client.clone(), &audio_files).await.unwrap();

                    let library_delete_res = model::AudioLibrary::delete_by_path(mongodb_client.clone(), delted_library_path).await.unwrap();
                }
            }

            if !updated_local_libraries.is_empty() {
                println!("sync updated local libraries: {:?}", updated_local_libraries);
                
                for updated_local_library in updated_local_libraries.iter() {
                    let local_library_path = Path::new(&updated_local_library.id);
                    let audio_files = model::AudioFile::get_self_by_library_path(mongodb_client.clone(), local_library_path.clone(), false).await.unwrap();
                    let audio_filenames: HashSet<_> = audio_files
                        .iter()
                        .map(|item| item.filename.to_owned())
                        .collect();
                    let mut audio_files: HashMap<String, document::AudioFile> = audio_files
                        .into_iter()
                        .map(|item| (item.filename.to_owned(), item))
                        .collect();

                    let local_audio_file_paths = Self::get_audio_file_paths(Path::new(&updated_local_library.id), &audio_types);

                    let local_audio_filenames: HashSet<_> = local_audio_file_paths
                        .iter()
                        .filter_map(|item| item.file_name()
                            .and_then(|item2| item2.to_str()))
                        .map(|item| item.to_owned())
                        .collect();
                    
                    let new_audio_filenames: HashSet<_> = local_audio_filenames.difference(&audio_filenames).cloned().collect();
                    let deleted_audio_filenames: HashSet<_> = audio_filenames.difference(&local_audio_filenames).cloned().collect();
                    let managed_audio_filenames: HashSet<_> = audio_filenames.difference(&deleted_audio_filenames).cloned().collect();

                    let mut updated_audio_files: Vec<document::AudioFile> = vec![];
                    let mut updated_audio_tags: Vec<document::AudioTag> = vec![];

                    for managed_audio_filename in managed_audio_filenames.iter() {
                        let mut audio_file = audio_files.remove(managed_audio_filename).unwrap();
                        if audio_file.check_modified() {
                            match audio_file.audio_tag_refer {
                                Some(audio_tag_id) => {
                                    let parent_path = util::path::materialized_to_path(&audio_file.parent_path);
                                    let updated_audio_tag = Self::create_audio_tag(Some(audio_tag_id), &parent_path, &audio_file.filename);
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

                            document::AudioFile::create_from_path(&target_path)
                        })
                        .collect();

                    let delete_audio_file_docs: Vec<_> = deleted_audio_filenames
                        .iter()
                        .filter_map(|item| audio_files.remove(item))
                        .collect();

                    if !new_audio_file_docs.is_empty() {
                        model::AudioFile::create_many(mongodb_client.clone(), &new_audio_file_docs).await.unwrap();
                    }

                    if !delete_audio_file_docs.is_empty() {
                        let deleted_audio_tag_ids: Vec<_> = delete_audio_file_docs
                            .iter()
                            .filter_map(|item| item.audio_tag_refer)
                            .collect();

                        model::AudioTag::delete_by_ids(mongodb_client.clone(), &deleted_audio_tag_ids).await.unwrap();

                        model::AudioFile::delete_by_selfs(mongodb_client.clone(), &delete_audio_file_docs).await.unwrap();
                    }

                    if !updated_audio_files.is_empty() {
                        model::AudioFile::update_self(mongodb_client.clone(), &updated_audio_files).await;
                    }

                    if !updated_audio_tags.is_empty() {
                        model::AudioTag::update_self(mongodb_client.clone(), &updated_audio_tags).await;
                    }

                    let local_library_modified_timestamp = util::path::get_timestamp(&local_library_path);
                    let update_local_library_res = model::AudioLibrary::update_modified_timestamp(mongodb_client.clone(), &updated_local_library.id, local_library_modified_timestamp).await;
                }
            }

        }
        // collection; libraries - audio library root
        //             libraries-detail - actual contents (sub_dirs, audio_files)



        // filter updated path (by paths' modified datetime)

        Ok(())
    }

    fn create_audio_tag(
        id: Option<ObjectId>,
        parent_path: &str,
        filename: &str,
    ) -> document::AudioTag {
        let mut audio_file_path = Path::new(parent_path).to_path_buf();
        audio_file_path.push(filename);

        let audio_file = File::open(audio_file_path).unwrap();
        let mut aiff = AiffReader::new(audio_file);
        aiff.read().unwrap();

        let id = match id {
            Some(id) => Some(id),
            None => Some(ObjectId::new())
        };

        let audio_metadata = if let Some(id3v2_tag) = aiff.id3v2_tag {
            let date_recorded = match id3v2_tag.date_recorded() {
                Some(datetime) => {
                    let month = datetime.month.unwrap_or_else(|| 1u8);
                    let day = datetime.day.unwrap_or_else(|| 1u8);
                    let hour = datetime.hour.unwrap_or_else(|| 0u8);
                    let minute = datetime.minute.unwrap_or_else(|| 0u8);
                    let second = datetime.second.unwrap_or_else(|| 0u8);

                    Some(Utc.ymd(datetime.year, month.into(), day.into()).and_hms(hour.into(), minute.into(), second.into()))
                },
                None => None,
            };

            let date_released = match id3v2_tag.date_released() {
                Some(datetime) => {
                    let month = datetime.month.unwrap_or_else(|| 1u8);
                    let day = datetime.day.unwrap_or_else(|| 1u8);
                    let hour = datetime.hour.unwrap_or_else(|| 0u8);
                    let minute = datetime.minute.unwrap_or_else(|| 0u8);
                    let second = datetime.second.unwrap_or_else(|| 0u8);

                    Some(Utc.ymd(datetime.year, month.into(), day.into()).and_hms(hour.into(), minute.into(), second.into()))
                },
                None => None,
            };

            let pictures: Vec<_> = id3v2_tag.pictures()
                .into_iter()
                .map(|item| document::AudioFileMetadataPicture {
                    description: item.description.clone(),
                    mime_type: item.mime_type.clone(),
                    picture_type: item.picture_type.to_string(),
                    data: item.data.to_owned(),
                })
                .collect();

            let artist = match id3v2_tag.artist() {
                Some(item) => Some(item.to_owned()),
                None => None,
            };

            let album = match id3v2_tag.album() {
                Some(item) => Some(item.to_owned()),
                None => None,
            };

            let album_artist = match id3v2_tag.album_artist() {
                Some(item) => Some(item.to_owned()),
                None => None,
            };

            let genre = match id3v2_tag.genre() {
                Some(item) => Some(item.to_owned()),
                None => None,
            };

            let title = match id3v2_tag.title() {
                Some(item) => Some(item.to_owned()),
                None => None,
            };

            let mut audio_tag = document::AudioTag {
                id,
                property_hash: None,
                artist: artist,
                album: album,
                album_artist: album_artist,
                date_recorded,
                date_released,
                disc: id3v2_tag.disc(),
                duration: id3v2_tag.duration(),
                genre: genre,
                pictures: pictures,
                title: title,
                total_discs: id3v2_tag.total_discs(),
                total_tracks: id3v2_tag.total_tracks(),
                track: id3v2_tag.track(),
                year: id3v2_tag.year(),
            };

            audio_tag.property_hash = Some(util::hash::get_hashed_value(&audio_tag));

            return audio_tag

        } else {
            return document::AudioTag {
                id,
                property_hash: None,
                title: Some(filename.clone().to_owned()),
                ..Default::default()
            };
        };
    }
}

pub struct AudioTag {}

impl AudioTag {
    pub async fn list_audio_tags(
        mongodb_client: mongodb::Client,
        max_item_num: u64,
        page: u64,
    ) -> Result<Vec<AudioTagRes>, String> {
        let get_all_res = model::AudioTag::get_all(mongodb_client.clone(), max_item_num as i64, page).await;

        let res: Vec<_> = get_all_res
            .iter()
            .map(|item| AudioTagRes {
                id: item.id.as_ref().unwrap().to_string(),
                artist: item.artist.as_ref().unwrap().to_string(),
                genre: item.genre.as_ref().unwrap().to_string(),
                title: item.title.as_ref().unwrap().to_string(),
            })
            .collect();

        Ok(res)
    }
}