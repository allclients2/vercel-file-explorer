// src/main.rs
use serde::{Deserialize, Serialize};
use tokio::{fs, io::AsyncWriteExt};
use warp::{http::Response, Filter, Rejection};
use base64::decode;
use warp::http::header::{HeaderMap, AUTHORIZATION};
use futures_util::{StreamExt, TryStreamExt};
use bytes::Buf;
use std::fs::OpenOptions;
use std::io::Write;

mod rejections;
use rejections::CustomRejection;

#[derive(Deserialize, Serialize)]
struct FileInfo {
    name: String,
}

#[tokio::main]
async fn main() {
    let list_files_route = warp::path("files")
        .and(warp::get())
        .and_then(list_files);

    let download_file_route = warp::path("files")
        .and(warp::path::param())
        .and(warp::get())
        .and_then(download_file);

    let upload_file_route = warp::path("upload")
        .and(warp::post())
        .and(with_basic_auth())
        .and(warp::multipart::form().max_length(1024 * 1024 * 10))
        .and_then(upload_file);

    warp::serve(list_files_route.or(download_file_route).or(upload_file_route))
        .run(([0, 0, 0, 0], 3030))
        .await;
}

async fn list_files() -> Result<impl warp::Reply, warp::Rejection> {
    let mut paths = fs::read_dir("./files").await.map_err(|e| warp::reject::custom(CustomRejection { message: e.to_string() }))?;
    let mut files = Vec::new();
    
    while let Some(entry) = paths.next_entry().await.map_err(|e| warp::reject::custom(CustomRejection { message: e.to_string() }))? {
        let file_name = entry.file_name().into_string().map_err(|e| warp::reject::custom(CustomRejection { message: format!("{:?}", e) }))?;
        files.push(FileInfo { name: file_name });
    }
    
    Ok(warp::reply::json(&files))
}

async fn download_file(name: String) -> Result<impl warp::Reply, warp::Rejection> {
    let file_path = format!("./files/{}", name);
    let file = fs::read(file_path).await.unwrap();
    Ok(Response::builder().body(file))
}

async fn upload_file(
    _auth: (),
    mut form: warp::multipart::FormData,
) -> Result<impl warp::Reply, warp::Rejection> {
    while let Ok(Some(part)) = form.try_next().await {
        // check if part represents a file
        if let Some(filename) = part.filename() {
            let filepath = format!("./files/{}", filename);
            let mut file = fs::File::create(filepath).await.map_err(|e| warp::reject::custom(CustomRejection { message: e.to_string() }))?;            // Process the stream of bytes for the file part
            let mut stream = part.stream();
            while let Ok(Some(chunk)) = stream.next().await.transpose() {
                let _ = file.write_all(chunk.chunk()).await;
            }
        }
    }
    Ok(warp::reply::json(&"Upload successful"))
}

// log the logins
async fn log_login(username: &str) -> std::io::Result<()> {
    use chrono::Utc;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logins.txt")?;
    writeln!(file, "{} logged in at {}", username, Utc::now())?;
    Ok(())
}

fn with_basic_auth() -> impl Filter<Extract = ((),), Error = Rejection> + Clone {
    warp::header::headers_cloned().and_then(|headers: HeaderMap| async move {
        match headers.get(AUTHORIZATION) {
            Some(header_value) => {
                if let Ok(auth_str) = header_value.to_str() {
                    if auth_str.starts_with("Basic ") {
                        let base64_encoded = &auth_str[6..];
                        if let Ok(decoded) = decode(base64_encoded) {
                            if let Ok(decoded_str) = String::from_utf8(decoded) {
                                let mut parts = decoded_str.split(':');
                                if let (Some(username), Some(password)) = (parts.next(), parts.next()) {
                                    if username == "admin" && password == "password" {
                                        let _ = log_login(username).await;
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            None => {}
        }
        Err(warp::reject::custom(Unauthorized))
    })
}

#[derive(Debug)]
struct Unauthorized;
impl warp::reject::Reject for Unauthorized {}
