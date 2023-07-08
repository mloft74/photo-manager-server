use std::io;

use axum::{
    extract::{DefaultBodyLimit, Multipart, State},
    routing::post,
    BoxError, Router,
};
use futures::{Stream, TryStreamExt};
use hyper::{body::Bytes, StatusCode};
use serde::Serialize;
use serde_json::json;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
use tower_http::limit::RequestBodyLimitLayer;

use crate::{
    api::IMAGES_DIR,
    domain::{actions::images::ImageSaver, models::Image},
};

pub fn make_upload_router<T: ImageSaver + 'static>(image_saver: T) -> Router {
    Router::new()
        .route("/upload", post(upload_image::<T>))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            250 * 1024 * 1024, /* 250mb */
        ))
        .with_state(image_saver)
}

#[derive(Serialize)]
enum UploadImageError {
    FileNameNotPresent,
    GeneralError(String),
}

impl UploadImageError {
    fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|e| {
            json!({
                "error": "jsonConverionFailed",
                "message": e.to_string(),
            })
            .to_string()
        })
    }
}

// Handler that accepts a multipart form upload and streams each field to a file.
async fn upload_image<T: ImageSaver>(
    saver: State<T>,
    mut multipart: Multipart,
) -> Result<(), (StatusCode, String)> {
    let mut outer_file_name = None;
    let mut count = 0;
    while let Ok(Some(field)) = multipart.next_field().await {
        tracing::debug!("iteration: {}", count);
        count += 1;
        let file_name = if let Some(file_name) = field.file_name() {
            file_name.to_owned()
        } else {
            continue;
        };

        stream_to_file(&file_name, field)
            .await
            .map_err(|(s, e)| (s, e.to_json_string()))?;
        if outer_file_name.is_some() {
            continue;
        }
        outer_file_name = Some(file_name)
    }

    let file_name = outer_file_name.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            UploadImageError::FileNameNotPresent.to_json_string(),
        )
    })?;

    saver.save_image(&Image { file_name }).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            UploadImageError::GeneralError(e.to_string()).to_json_string(),
        )
    })?;

    Ok(())
}

// Save a `Stream` to a file
async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), (StatusCode, UploadImageError)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    if !path_is_valid(path) {
        return Err((
            StatusCode::BAD_REQUEST,
            UploadImageError::GeneralError("Invalid path".to_string()),
        ));
    }

    async {
        // Convert the stream into an `AsyncRead`.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the file. `File` implements `AsyncWrite`.
        let path = std::path::Path::new(IMAGES_DIR).join(path);
        let mut file = BufWriter::new(File::create(path).await?);

        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            UploadImageError::GeneralError(err.to_string()),
        )
    })
}

// to prevent directory traversal attacks we ensure the path consists of exactly one normal
// component
fn path_is_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}
