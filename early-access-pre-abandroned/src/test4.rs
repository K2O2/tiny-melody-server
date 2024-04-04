use id3::{Tag, TagLike};
use image::imageops::FilterType;
use image::ImageFormat;
use serde_json::{json, Value};
use std::fs::{self};
use std::io::{self, prelude::*};
use std::{fs::File, path::PathBuf};

fn main() {
    println!("Hello,Tiny Melody Server!");

    let mut input = String::new();
    print!("是否重建数据库? (y/n): ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();

    // 处理用户输入
    let input = input.trim().to_lowercase();
    if input == "y" || input == "yes" {
        // 用户选择调用函数
        start_search_directory();
    } else {
        // 用户选择不调用函数
        println!("用户放弃建立.");
    }

    
}

fn start_search_directory() {
    let folder_path = "D:/Music/LOCAL";
    let (music_paths, folder_data) = scan_directory(&folder_path);

    if let Err(err) = fs::create_dir_all("web/data") {
        eprintln!("Failed to create web directory: {}", err);
        return;
    }
    if let Err(err) = fs::create_dir_all("web/image/folder") {
        eprintln!("Failed to create web directory: {}", err);
        return;
    }

    save_paths(&music_paths);
    save_folder_structure(&folder_data);
}

fn save_paths(music_paths: &[String]) {
    let music_file = "web/data/music.list";

    if let Ok(mut file) = File::create(music_file) {
        for path in music_paths {
            if let Err(err) = writeln!(file, "{}", path) {
                eprintln!("Failed to write music path to file: {}", err);
            }
        }
    } else {
        eprintln!("Failed to create music path file");
    }
}

fn save_folder_structure(json_string: &str) {
    let folder_file = "web/data/folder.json";

    if let Ok(mut file) = File::create(folder_file) {
        if let Err(err) = write!(file, "{}", json_string) {
            eprintln!("Failed to write folder structure to file: {}", err);
        }
    } else {
        eprintln!("Failed to create folder structure file");
    }
}

fn scan_directory(path: &str) -> (Vec<String>, String) {
    let mut music_paths: Vec<String> = Vec::new();
    let mut json_structure = json!({});
    let mut index: u32 = 0;

    // 递归遍历目录
    scan_recursive(path, &mut music_paths, &mut json_structure, &mut index);

    let json_string = serde_json::to_string_pretty(&json_structure).unwrap();

    (music_paths, json_string)
}

fn scan_recursive(
    path: &str,
    music_paths: &mut Vec<String>,
    json_structure: &mut Value,
    index: &mut u32,
) {
    if let Ok(entries) = fs::read_dir(path) {
        let mut cover_check = true;
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                let entry_name = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                if entry_path.is_dir() {
                    let mut nested_json = json!({});
                    scan_recursive(
                        entry_path.to_str().unwrap(),
                        music_paths,
                        &mut nested_json,
                        index,
                    );
                    json_structure[entry_name] = nested_json;
                } else if entry_path.is_file() {
                    if let Some(extension) = entry_path.extension() {
                        if extension == "mp3" {
                            //Only Save Music Files Path
                            music_paths.push(entry_path.to_string_lossy().into());
                            //Resign the type
                            json_structure[entry_name] = json!({
                                "type": "mp3"
                            });
                        } else if extension == "jpg" && cover_check {
                            cover_check = false;
                            // println!("{},{}",&entry_path.display(),&index);
                            if let Some(file_name) = entry_path.file_name() {
                                // println!("{:?}",&file_name);
                                if file_name == "cover.jpg" || file_name == "Cover.jpg" {
                                    if let Err(err) = compress_and_copy_jpg(&entry_path, &index) {
                                        eprintln!(
                                            "Something went wrong when saving covers. {} ",
                                            err
                                        );
                                    }
                                    *index += 1;
                                    //Resign the cover
                                    json_structure[&index.to_string()] = json!({
                                        "type": "jpg"
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn compress_and_copy_jpg(
    input_path: &PathBuf,
    index: &u32,
) -> Result<(), Box<dyn std::error::Error>> {
    // 读取输入路径的图像文件
    let image = image::open(input_path)?;

    // 压缩图像为512x512像素
    let resized_image = image.resize_to_fill(512, 512, FilterType::Nearest);

    // 构建输出路径
    let output_filename = format!("{}.jpg", index);
    let output_path = PathBuf::from("web/image/folder").join(output_filename);

    // 保存压缩后的图像到输出路径
    resized_image.save_with_format(&output_path, ImageFormat::Jpeg)?;

    Ok(())
}

#[derive(Debug)]
struct MusicInfo {
    artist: Option<String>,
    title: Option<String>,
    album: Option<String>,
    disc: Option<u32>,
    track: Option<u32>,
    year: Option<i32>,
}

fn music_parse(path: &PathBuf) -> Result<MusicInfo, Box<dyn std::error::Error>> {
    let tag = Tag::read_from_path(path)?;

    let music_info = MusicInfo {
        artist: tag.artist().map(|s| s.to_owned()),
        title: tag.title().map(|s| s.to_owned()),
        album: tag.album().map(|s| s.to_owned()),
        disc: tag.disc(),
        track: tag.track(),
        year: tag.year(),
    };
    Ok(music_info)
}
