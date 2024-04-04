use std::ffi::OsStr;
use std::path::Path;
use std::{ fs::{ self }, path::PathBuf };

use config::Config;
use id3::{ Tag, TagLike };
use serde_json::{ json, Value };

type SongPaths = Vec<String>;
type FolderStructure = String;
type TagList = String;

const DATA_DIR: &str = "./data";
const SONG_LIST_PATH: &str = "./data/song.list";
const FOLDER_LIST_PATH: &str = "./data/folder.json";
const TAG_PATH: &str = "./data/tag.json";

fn read_file_to_string<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}

pub fn prepare_data(config: &Config) {
    match config.get_string("autosearch").unwrap().as_str() {
        "true" => {
            println!("Starting Auto Search...");

            search_main(&config)
        }
        _ => println!("Auto Search Disabled"),
    }
}

pub fn read_data() -> (SongPaths, FolderStructure, TagList) {
    println!("Starting Read Data...");

    let song_paths = read_file_to_string(SONG_LIST_PATH)
        .map(|content| content.lines().map(ToString::to_string).collect())
        .unwrap_or_default();

    let folder_structure = read_file_to_string(FOLDER_LIST_PATH).unwrap_or_default();
    let tag_list = read_file_to_string(TAG_PATH).unwrap_or_default();

    println!("Read Data Complete");

    (song_paths, folder_structure, tag_list)
}

pub fn search_main(config: &Config) {
    let music_path = PathBuf::from(config.get_string("datapath").unwrap());
    let mut tag_list = MusicTag::default();
    let mut count: u32 = 0;
    let start: u32 = 0;
    let mut song_path: Vec<String> = Vec::new();
    let mut folder_structure = Vec::new();
    let standard_path = music_path.clone();

    folder_traveler(
        &standard_path,
        &music_path,
        &mut count,
        &mut song_path,
        &mut folder_structure,
        &mut tag_list,
        start
    );

    println!("Search Complete,{} Songs Found.", count);

    let tag_list = serde_json
        ::to_string_pretty(
            &json!({
        "title": tag_list.title,
        "artist": tag_list.artist,
        "album": tag_list.album,
        "year": tag_list.year,
    })
        )
        .map_err(|e| {
            eprintln!("Error serializing tag list: {}", e);
        })
        .ok()
        .unwrap_or_default();

    let folder_structure = serde_json
        ::to_string_pretty(&folder_structure)
        .map_err(|e| {
            eprintln!("Error serializing folder structure: {}", e);
        })
        .ok()
        .unwrap_or_default();

    data_save(&song_path, &folder_structure, &tag_list).unwrap_or_else(|e| {
        eprintln!("Error saving data: {}", e);
    });

    println!("Data Saved,Search Complete");
}

fn data_save(
    song_path: &[String],
    folder_structure: &str,
    tag_list: &str
) -> Result<(), std::io::Error> {
    fs::create_dir_all(DATA_DIR)?;

    fs::write(SONG_LIST_PATH, song_path.join("\n"))?;

    fs::write(FOLDER_LIST_PATH, folder_structure)?;

    fs::write(TAG_PATH, tag_list)?;

    Ok(())
}

fn folder_traveler(
    standard_path: &PathBuf,
    music_path: &PathBuf,
    count: &mut u32,
    song_path: &mut Vec<String>,
    folder_structure: &mut Vec<Value>,
    tag_list: &mut MusicTag,
    start: u32
) {
    if let Ok(entries) = fs::read_dir(music_path) {
        let mut children = Vec::new();
        let mut current_index = start;

        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                let entry_name = entry_path.file_name().and_then(OsStr::to_str).unwrap_or("");

                if entry_path.is_dir() {
                    let mut sub_count = 0;
                    let mut sub_song_path = Vec::new();
                    let mut sub_folder_structure = Vec::new();

                    folder_traveler(
                        standard_path,
                        &entry_path,
                        &mut sub_count,
                        &mut sub_song_path,
                        &mut sub_folder_structure,
                        tag_list,
                        current_index
                    );

                    if sub_count > 0 {
                        let folder_entry =
                            json!({
                            "label": entry_name,
                            "count": sub_count,
                            "start": current_index,
                            "children": sub_folder_structure,
                        });

                        children.push(folder_entry);
                    }

                    *count += sub_count;
                    song_path.append(&mut sub_song_path);
                    current_index += sub_count;
                } else if entry_path.is_file() {
                    if let Some(extension) = entry_path.extension() {
                        if extension == "mp3" {
                            *count += 1;
                            song_path.push(
                                entry_path
                                    .strip_prefix(standard_path)
                                    .unwrap()
                                    .to_string_lossy()
                                    .to_string()
                            );
                            music_parse(&entry_path, entry_name, tag_list);
                        }
                    }
                }
            }
        }

        *folder_structure = children;
    }
}

struct MusicTag {
    title: Vec<String>,
    artist: Vec<String>,
    album: Vec<String>,
    year: Vec<u32>,
}

fn music_parse(path: &PathBuf, name: &str, tags: &mut MusicTag) {
    match Tag::read_from_path(&path) {
        Ok(tag) => {
            let title = tag.title().unwrap_or(name);
            let artist = tag.artist().unwrap_or("anonymous");
            let album = tag.album().unwrap_or_else(|| path.get_folder_name().unwrap_or_default());
            let year = tag.year().unwrap_or(2077);

            tags.title.push(title.to_string());
            tags.artist.push(artist.to_string());
            tags.album.push(album.to_string());
            tags.year.push(year as u32);
        }
        Err(error) => {
            println!("Error reading MP3 tags: {}, File: {}", error, path.to_str().unwrap());

            let title = name.to_string();
            let album = path.get_folder_name().unwrap_or_else(|| "unknown");

            tags.title.push(title);
            tags.artist.push("anonymous".to_string());
            tags.album.push(album.to_string());
            tags.year.push(2077);
        }
    }
}

trait RemoveExtension {
    fn remove_extension(&self) -> &str;
}

impl RemoveExtension for str {
    fn remove_extension(&self) -> &str {
        self.rsplit_once('.')
            .map(|(s, _)| s)
            .unwrap_or(self)
    }
}

trait FolderName {
    fn get_folder_name(&self) -> Option<&str>;
}

impl FolderName for PathBuf {
    fn get_folder_name(&self) -> Option<&str> {
        self.parent().and_then(|p| p.file_name().and_then(OsStr::to_str))
    }
}

impl Default for MusicTag {
    fn default() -> Self {
        Self {
            title: Vec::new(),
            artist: Vec::new(),
            album: Vec::new(),
            year: Vec::new(),
        }
    }
}
