use id3::{Tag, TagLike};
use serde_json::{json, Value};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
fn main() {
    println!("Hello Tiny Melody Server!");

    let music_path = PathBuf::from("/media/extension/Music/Standard");
    let cover_path = "";

    search_main(&music_path);
}

fn search_main(music_path: &PathBuf) {
    let mut count: u32 = 0;
    let mut song_path: Vec<String> = Vec::new();
    let mut folder_structure = json!({});
    let standard_path = music_path.clone();

    folder_traveler(
        &standard_path,
        &music_path,
        &mut count,
        &mut song_path,
        &mut folder_structure,
    );

    println!("Search Complete,{} Songs Found.", count);

    let folder_structure = serde_json::to_string_pretty(&folder_structure).unwrap();

    data_save(&song_path, &folder_structure);

    // println!("{:?}",song_path);
    // println!("{:?}",music_path);
    // println!("{:?}",song_path);
}

fn folder_traveler(
    standard_path: &PathBuf,
    music_path: &PathBuf,
    count: &mut u32,
    song_path: &mut Vec<String>,
    folder_structure: &mut Value,
) {
    if let Ok(entries) = fs::read_dir(music_path) {
        //start to loop
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                //this name is contend with ext
                let entry_name = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                //try to recognize thefolder and process
                if entry_path.is_dir() {
                    //create json
                    let mut nested_json = json!({});
                    folder_traveler(
                        standard_path,
                        &entry_path,
                        count,
                        song_path,
                        &mut nested_json,
                    );
                    folder_structure[entry_name] = nested_json;
                    //save to json
                } else if entry_path.is_file() {
                    if let Some(extension) = entry_path.extension() {
                        if extension == "mp3" {
                            //count +1
                            *count += 1;

                            //saving paths
                            println!("{:?}", entry_path);
                            // println!("{:?}",music_path);
                            song_path.push(
                                entry_path
                                    .strip_prefix(standard_path)
                                    .unwrap()
                                    .to_string_lossy()
                                    .to_string(),
                            );
                            //reading tags
                            music_parse(&entry_path, entry_name, folder_structure, &count);
                            //save to folder
                        }
                    }
                }
            }
        }
    }
}

fn data_save(song_path: &Vec<String>, folder_structure: &String) {
    let song_list = "song.list";
    let folder_list = "web/data/folder.json";

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
}

fn music_parse(path: &PathBuf, name: &str, tags: &mut Value, index: &u32) {
    match Tag::read_from_path(&path) {
        Ok(tag) => {
            // 读取标签成功
            tags[name] = json!({
                "index":index,
                "title": tag.title().unwrap_or_else(|| name.remove_extension()),
                "artist": tag.artist(),
                "album": tag.album().unwrap_or_else(|| path.get_folder_name().unwrap_or_default()),
                "disc": tag.disc(),
                "track": tag.track(),
                "year": tag.year(),
            });
        }
        Err(error) => {
            // 读取标签失败
            println!("Error reading MP3 tags: {}", error);
            // 继续执行其他逻辑
            tags[name] = json!({
                "index":index,
                "title": name.remove_extension(),
                "artist": path.get_folder_name(),
                "album": "",
                "disc": "",
                "track": "",
                "year": "",
            });
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
