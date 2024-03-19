use id3::{Tag, TagLike};
use serde_json::{json, Value};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
fn main() {
    println!("Hello Tiny Melody Server!");

    let music_path = PathBuf::from("D:/Music/Standard");

    search_main(&music_path);
}

fn search_main(music_path: &PathBuf) {

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

    println!("Start to Compress.");

    let tag_list = serde_json::to_string_pretty(&json!({
        "title":tag_list.title,
        "artist":tag_list.artist,
        "album":tag_list.album,
        "year":tag_list.year,

    })).unwrap();

    let folder_structure = serde_json::to_string_pretty(&folder_structure).unwrap();

    data_save(&song_path, &folder_structure,&tag_list);

    println!("Data Saved,Search Complete.")

}

fn folder_traveler(
    standard_path: &PathBuf,
    music_path: &PathBuf,
    count: &mut u32,
    song_path: &mut Vec<String>,
    folder_structure: &mut Vec<Value>,
    tag_list: &mut MusicTag,
    start: u32,
) {
    if let Ok(entries) = fs::read_dir(music_path) {
        let mut children = Vec::new();
        let mut current_index = start;

        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                let entry_name = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

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
                        current_index,
                    );

                    if sub_count > 0 {
                        let folder_entry = json!({
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
                                    .to_string(),
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

fn data_save(song_path: &Vec<String>, folder_structure: &String, tag_list: &String) {
    let song_list = "song.list";
    let folder_list = "folder.json";
    let tag = "tag.json";

    if let Ok(mut file) = File::create(song_list) {
        for path in song_path {
            if let Err(err) = writeln!(file, "{}", path) {
                eprintln!("Failed to write music path to file: {}", err);
            }
        }
    } else {
        eprintln!("Failed to create music path file");
    }

    if let Ok(mut file) = File::create(folder_list) {
        if let Err(err) = write!(file, "{}", folder_structure) {
            eprintln!("Failed to write folder_structure to file: {}", err);
        }
    } else {
        eprintln!("Failed to create folder_structure file");
    }

    if let Ok(mut file) = File::create(tag) {
        if let Err(err) = write!(file, "{}", tag_list) {
            eprintln!("Failed to write tag_list to file: {}", err);
        }
    } else {
        eprintln!("Failed to create tag file");
    }
}

struct MusicTag{
    title:Vec<String>,
    artist:Vec<String>,
    album:Vec<String>,
    year:Vec<u32>,
}

fn music_parse(path: &PathBuf, name: &str, tags: &mut MusicTag) {
    match Tag::read_from_path(&path) {
        Ok(tag) => {
            let title = tag.title().unwrap_or_else(|| name);
            let artist = tag.artist().unwrap_or_else(|| "anonymous");
            let album = tag.album().unwrap_or_else(|| path.get_folder_name().unwrap_or_default());
            let year = tag.year().unwrap_or(2077);

            tags.title.push(title.to_string());
            tags.artist.push(artist.to_string());
            tags.album.push(album.to_string());
            tags.year.push(year as u32);
        }
        Err(error) => {
            println!("Error reading MP3 tags: {}", error);

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
        if let Some(dot_index) = self.rfind('.') {
            &self[..dot_index]
        } else {
            self
        }
    }
}

trait FolderName {
    fn get_folder_name(&self) -> Option<&str>;
}

impl FolderName for PathBuf {
    fn get_folder_name(&self) -> Option<&str> {
        if let Some(parent) = self.parent() {
            parent
                .file_name()
                .and_then(|folder_name| folder_name.to_str())
        } else {
            None
        }
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
