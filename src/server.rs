use std::{ collections::HashMap, path::Path, usize };

use ::config::Config;
use axum::{
    extract::Query,
    http::{ HeaderMap, StatusCode },
    response::IntoResponse,
    routing::get,
    Router,
};
use axum::extract::Path as AxumPath;
use tokio::sync::mpsc;
use tower_http::services::ServeDir;

pub enum ControlMessage {
    Restart,
    Stop,
    Search,
}

pub mod data;

pub async fn start_server(tx: mpsc::Sender<ControlMessage>, config: Config) {
    let (song_paths, folder_structure, tag_list) = data::read_data();

    let (tx_stop, mut rx_stop) = mpsc::channel(1);

    let addr = config.get_string("host").unwrap();
    let port = config.get_int("port").unwrap();
    let addr = format!("{}:{}", addr, port);

    let api_router = Router::new()
        .route(
            "/data/tag",
            get(move || async { tag_list })
        )
        .route(
            "/data/folder",
            get(move || async { folder_structure })
        )
        .route(
            "/index/:index",
            get(move |AxumPath(index): AxumPath<usize>| async move {
                get_music_file(&config, index, &song_paths)
            })
        )
        .route(
            "/control",
            get(|Query(params): Query<HashMap<String, String>>| async move {
                if let Some(control) = params.get("control") {
                    if control == "restart" {
                        tx.send(ControlMessage::Restart).await.unwrap();
                        tx_stop.send(()).await.unwrap();
                    } else if control == "stop" {
                        tx.send(ControlMessage::Stop).await.unwrap();
                        tx_stop.send(()).await.unwrap();
                    } else if control == "search" {
                        tx.send(ControlMessage::Search).await.unwrap();
                        tx_stop.send(()).await.unwrap();
                    }
                    (StatusCode::NOT_FOUND, "Unknown control command")
                } else {
                    (StatusCode::BAD_REQUEST, "Missing control parameter")
                }
            })
        );

    let app = Router::new().nest("/api", api_router).nest_service("/", ServeDir::new("web"));

    println!("Listening on http://{}", addr);


    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(async move {
            rx_stop.recv().await;
        }).await
        .unwrap();
}

fn get_music_file(
    config: &Config,
    index: usize,
    song_paths: &[String]
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if index < song_paths.len() {
        let song_path = &song_paths[index];
        let full_path = format!("{}/{}", config.get_string("datapath").unwrap(), song_path);
        let path = Path::new(&full_path);

        if path.exists() {
            let mut headers = HeaderMap::new();
            headers.insert(
                "content-type",
                (
                    match path.extension().and_then(|ext| ext.to_str()) {
                        Some("mp3") => "audio/mpeg",
                        // Some("wav") => "audio/wav",
                        _ => "application/octet-stream",
                    }
                )
                    .parse()
                    .unwrap()
            );

            let file_content = std::fs
                ::read(path)
                .map_err(|_| (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to read file".to_string(),
                ))?;
            Ok((headers, file_content))
        } else {
            Err((StatusCode::NOT_FOUND, "File not found".to_string()))
        }
    } else {
        Err((StatusCode::BAD_REQUEST, "Index out of bounds".to_string()))
    }
}
