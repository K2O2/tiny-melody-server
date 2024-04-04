use std::path::Path;
use ::config::Config;
use config::{ config_load, showlogo };
use server::ControlMessage;
use tokio::sync::mpsc;

mod config;
mod server;

const CONFIG_FILE: &str = "config.toml";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env::set_var("RUST_SERVER_LOG", "info");
    // pretty_env_logger::init_custom_env("RUST_SERVER_LOG");

    let config: Config = prepare_startup();

    server::data::prepare_data(&config);

    // 创建一个消息通道来控制服务器的启停
    let (tx, mut rx) = mpsc::channel(1);

    loop {
        // 启动服务器
        let server_tx = tx.clone();
        let server_config = config.clone();
        tokio::spawn(server::start_server(server_tx, server_config));

        // 等待服务器发送信号
        match rx.recv().await {
            Some(ControlMessage::Restart) => {
                // 处理重启信号
                println!("Server has been signaled to restart.");
                // 继续下一次循环以重启服务器
                continue;
            }
            Some(ControlMessage::Stop) => {
                // 如果发送端关闭，则退出循环
                println!("Server has been signaled to stop.");
                break;
            }
            Some(ControlMessage::Search) => {
                // 处理搜索信号
                println!("Server has been signaled to search.");
                println!("Start to search...");
                server::data::search_main(&config);
                println!("Search finished,Now Restarting...");
                // 继续下一次循环以重启服务器
                continue;
            }
            None => {
                println!("Server has been signaled to stop.");
                break;
            }
        }
    }

    Ok(())
}

fn prepare_startup() -> Config {
    let config_main = match config_load(Path::new(CONFIG_FILE)) {
        Ok(config) => {
            println!("Config loaded successfully.");
            config
        }
        Err(e) => {
            println!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    showlogo();

    config_main
}
