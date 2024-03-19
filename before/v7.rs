use id3::{Tag, TagLike};
use serde_json::{json, Value};
use std::{
    cmp::Ordering,
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
    let mut count: u32 = 0;
    let mut folder_index: u32 = 0;
    let mut level: i32 = 0;
    let mut song_path: Vec<String> = Vec::new();
    let mut folder_title: Vec<String> = Vec::new();
    let mut folder_level: Vec<i32> = Vec::new();
    let mut folder_dad: Vec<u32> = Vec::new();
    let mut folder_count: Vec<u32> = Vec::new();
    let mut tag_list = json!({});
    let standard_path = music_path.clone();

    folder_dad.push(0);
    folder_level.push(0);
    folder_count.push(0);
    folder_title.push(String::from("root"));

    folder_traveler(
        &standard_path,
        &music_path,
        &mut count,
        &mut song_path,
        &mut folder_title,
        &mut folder_level,
        &mut folder_index,
        &mut folder_dad,
        &mut folder_count,
        &mut tag_list,
        &mut level,
    );

    println!("Search Complete,{} Songs Found.", count);

    println!("Start to Compress.");

    // remove_empty_values(&mut folder_structure);

    let folder_structure: Value = json!({
        "title":serde_json::to_value(folder_title).unwrap(),
        "dad":serde_json::to_value(folder_dad).unwrap(),
        "level":serde_json::to_value(folder_level).unwrap(),
        "count":serde_json::to_value(folder_count).unwrap(),
    });

    let folder_structure = serde_json::to_string_pretty(&folder_structure).unwrap();
    let tag_list = serde_json::to_string_pretty(&tag_list).unwrap();

    data_save(&song_path, &folder_structure, &tag_list);

    println!("Data Saved,Search Complete.")

    // println!("{:?}",song_path);
    // println!("{:?}",music_path);
    // println!("{:?}",song_path);
}

fn remove_empty_values(json_obj: &mut Value) {
    if let Some(obj) = json_obj.as_object_mut() {
        let keys: Vec<String> = obj.keys().cloned().collect();
        for key in keys {
            if obj[&key].is_object() {
                remove_empty_values(&mut obj[&key]);
                if obj[&key].as_object().unwrap().is_empty() {
                    obj.remove(&key);
                }
            }
        }
    }
}

fn folder_traveler(
    standard_path: &PathBuf,
    music_path: &PathBuf,
    count: &mut u32,
    song_path: &mut Vec<String>,
    folder_title: &mut Vec<String>,
    folder_level: &mut Vec<i32>,
    folder_index: &mut u32,
    folder_dad: &mut Vec<u32>,
    folder_count: &mut Vec<u32>,
    tag_list: &mut Value,
    level: &mut i32,
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
                    *level += 1;
                    //create json
                    folder_title.push(entry_name.to_string());
                    folder_level.push(*level);
                    folder_count.push(*count);
                    // println!("{}", *folder_index as usize);
                    // println!("{}", folder_level[*folder_index as usize]);
                    match folder_level[*folder_index as usize].cmp(&(*level)) {
                        Ordering::Equal => {
                            println!("equal {}", folder_index);
                            folder_dad.push(folder_dad[*folder_index as usize]);
                            // *folder_index += 1;
                        }
                        Ordering::Greater => {
                            println!("greater {}", folder_index);
                            if *folder_index != 0 {
                                folder_dad.push(folder_dad[folder_dad[*folder_index as usize] as usize]);
                            }
                            // *folder_index += 1;
                        }
                        Ordering::Less => {
                            println!("less {}", folder_index);
                            folder_dad.push(*folder_index);
                        }
                    }
                    *folder_index += 1;
                    folder_traveler(
                        standard_path,
                        &entry_path,
                        count,
                        song_path,
                        folder_title,
                        folder_level,
                        folder_index,
                        folder_dad,
                        folder_count,
                        tag_list,
                        level,
                    );
                    *level += -1;
                } else if entry_path.is_file() {
                    if let Some(extension) = entry_path.extension() {
                        if extension == "mp3" {
                            //count +1
                            *count += 1;

                            //saving paths
                            // println!("{:?}", entry_path);
                            // println!("{:?}",music_path);
                            song_path.push(
                                entry_path
                                    .strip_prefix(standard_path)
                                    .unwrap()
                                    .to_string_lossy()
                                    .to_string(),
                            );
                            //reading tags
                            music_parse(&entry_path, entry_name, tag_list, &count);
                            //save to folder
                            // folder_structure["content"] = json!("index":count)
                        }
                    }
                }
            }
        }
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

fn music_parse(path: &PathBuf, name: &str, tags: &mut Value, index: &u32) {
    match Tag::read_from_path(&path) {
        Ok(tag) => {
            // 读取标签成功
            tags[index.to_string()] = json!({
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
            tags[index.to_string()] = json!({
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
